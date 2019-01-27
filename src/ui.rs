use crate::cli_opt::*;
use crate::template_cfg::Variable;
use crate::{Action, Ctx, Variables};
use console::Style;
use console::Term;
use dialoguer::Confirmation;
use dialoguer::Input;
use failure::Error;
use lazy_static::lazy_static;
use slog::debug;

lazy_static! {
    static ref term: Term = Term::stdout();
    static ref title_style: Style = Style::new().bold();
}

fn write_title(s: &str) -> Result<(), Error> {
    term.write_line(&format!("\n\n{}\n", title_style.apply_to(s)))?;
    Ok(())
}

pub fn ask_variables(
    ctx: &Ctx,
    list_variables: &Vec<Variable>,
    mut init: Variables,
) -> Result<Variables, Error> {
    let mut variables = Variables::new();
    variables.append(&mut init);
    if !ctx.cmd_opt.x_always_default_value {
        write_title("Configure variables")?;
        // TODO optimize to reduce clones
        for variable in list_variables.iter().cloned() {
            let name = variable.name;
            let value: String = {
                let mut input = Input::new();
                if let Some(default_value) = variable.default_value {
                    input.default(default_value);
                }
                let prompt = if variable.ask.is_some() {
                    variable.ask.unwrap()
                } else {
                    name.clone()
                };
                // TODO manage error
                input.with_prompt(&prompt).interact()?
            };
            variables.insert(name, value);
        }
    } else {
        for variable in list_variables.iter() {
            let name = variable.name.clone();
            let value = (variable.default_value)
                .clone()
                .unwrap_or_else(|| "".into());
            variables.insert(name, value);
        }
    }
    Ok(variables)
}

//TODO add flag to filter display: all, changes, none
pub fn confirm_plan(ctx: &Ctx, actions: &[Action]) -> Result<bool, Error> {
    write_title("Plan to execute")?;
    debug!(ctx.logger, "plan"; "actions" => format!("{:?}", actions));
    for a in actions {
        let s = format!(
            "   - {} {:?}",
            format!("{:?}", a.operation).to_lowercase(),
            a.dst_path.base.join(&a.dst_path.relative)
        );
        term.write_line(&s)?;
    }
    let r = if ctx.cmd_opt.confirm == AskConfirmation::Always {
        Confirmation::new()
            .with_text("Do you want to apply plan ?")
            .interact()?
    } else {
        //TODO implement a algo for auto, like if no change then no ask.
        true
    };
    Ok(r)
}
