use crate::cli_opt::*;
use crate::template_cfg::Variable;
use crate::Result;
use crate::{Action, Ctx, Variables};
use console::Style;
use console::Term;
use dialoguer::Confirmation;
use dialoguer::Input;
use handlebars_misc_helpers::new_hbs;
use lazy_static::lazy_static;
use slog::debug;
use snafu::ResultExt;

lazy_static! {
    static ref TERM: Term = Term::stdout();
    static ref TITLE_STYLE: Style = Style::new().bold();
}

fn write_title(s: &str) -> Result<()> {
    TERM.write_line(&format!("\n\n{}\n", TITLE_STYLE.apply_to(s)))
        .context(crate::Io {})?;
    Ok(())
}

pub fn ask_variables(
    ctx: &Ctx,
    list_variables: &[Variable],
    mut init: Variables,
) -> Result<Variables> {
    let mut variables = Variables::new();
    variables.append(&mut init);
    let handlebars = new_hbs();
    if !ctx.cmd_opt.x_always_default_value {
        write_title("Configure variables")?;
        // TODO optimize to reduce clones
        for variable in list_variables.iter().cloned() {
            let name = variable.name;
            let value: String = {
                let mut input = Input::new();
                if let Some(default_value) = variable.default_value {
                    let default = handlebars
                        .render_template(&default_value, &variables)
                        .context(crate::Handlebars {})?;
                    input.default(default);
                }
                let prompt = if variable.ask.is_some() {
                    let ask = variable.ask.expect("variable ask should defined");
                    handlebars
                        .render_template(&ask, &variables)
                        .context(crate::Handlebars {})?
                } else {
                    name.clone()
                };
                // TODO manage error
                input
                    .with_prompt(&prompt)
                    .interact()
                    .context(crate::Io {})?
            };
            variables.insert(name, value);
        }
    } else {
        for variable in list_variables {
            let name = variable.name.clone();
            let value = if let Some(default_value) = &variable.default_value {
                handlebars
                    .render_template(&default_value, &variables)
                    .context(crate::Handlebars {})?
            } else {
                "".to_owned()
            };
            variables.insert(name, value);
        }
    }
    Ok(variables)
}

//TODO add flag to filter display: all, changes, none
pub fn confirm_plan(ctx: &Ctx, actions: &[Action]) -> Result<bool> {
    write_title("Plan to execute")?;
    debug!(ctx.logger, "plan"; "actions" => format!("{:?}", actions));
    for a in actions {
        let s = format!(
            "   - {} {:?}",
            format!("{:?}", a.operation).to_lowercase(),
            a.dst_path.base.join(&a.dst_path.relative)
        );
        TERM.write_line(&s).context(crate::Io {})?;
    }
    let r = if ctx.cmd_opt.confirm == AskConfirmation::Always {
        Confirmation::new()
            .with_text("Do you want to apply plan ?")
            .interact()
            .context(crate::Io {})?
    } else {
        //TODO implement a algo for auto, like if no change then no ask.
        true
    };
    Ok(r)
}
