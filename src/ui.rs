use crate::cli_opt::*;
use crate::template_cfg::ValuesForSelection;
use crate::template_cfg::Variable;
use crate::Result;
use crate::{Action, Ctx, Variables};
use console::Style;
use console::Term;
use dialoguer::Confirmation;
use dialoguer::Input;
use dialoguer::Select;
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

pub struct VariableResponse {
    value: String,
    idx: Option<usize>,
}

pub struct VariableRequest {
    prompt: String,
    default_value: Option<VariableResponse>,
    values: Vec<String>,
}

pub fn ask_variables(
    ctx: &Ctx,
    list_variables: &[Variable],
    mut init: Variables,
) -> Result<Variables> {
    let mut variables = Variables::new();
    variables.append(&mut init);
    let handlebars = new_hbs();
    write_title("Configure variables")?;
    // TODO optimize to reduce clones
    for variable in list_variables.iter().cloned() {
        let name = variable.name;
        let request = {
            let prompt = if variable.ask.is_some() {
                let ask = variable.ask.expect("variable ask should defined");
                handlebars
                    .render_template(&ask, &variables)
                    .context(crate::Handlebars {
                        when: format!("define prompt for '{}'", &name),
                        template: ask.clone(),
                    })?
            } else {
                name.clone()
            };
            let values: Vec<String> = match variable.select_in_values {
                ValuesForSelection::Empty => vec![],
                ValuesForSelection::Sequence(v) => v.clone(),
                ValuesForSelection::String(s) => {
                    let s_evaluated = handlebars
                        .render_template(&s, &variables)
                        .context(crate::Handlebars {
                            when: format!("define values for '{}'", &name),
                            template: s.clone(),
                        })?;
                    let s_values: Vec<String> =
                        serde_yaml::from_str(&s_evaluated).context(crate::SerdeYaml {})?;
                    //dbg!(&s_values);
                    s_values
                }
            };
            let default_value = variable
                .default_value
                .and_then(|default_value| {
                    handlebars
                        .render_template(&default_value, &variables)
                        //TODO better manage error
                        .context(crate::Handlebars {
                            when: format!("define default_value for '{}'", &name),
                            template: default_value.clone(),
                        })
                        .ok()
                })
                .map(|value| {
                    let idx = values
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| if v == &value { Some(i) } else { None })
                        .nth(0);
                    VariableResponse { value, idx }
                });
            VariableRequest {
                prompt,
                values,
                default_value,
            }
        };
        let resp = if variable.hidden || ctx.cmd_opt.x_always_default_value {
            request.default_value.unwrap_or(VariableResponse {
                value: "".to_owned(),
                idx: None,
            })
        } else {
            ask_variable_value(request)?
        };
        if let Some(idx) = resp.idx {
            variables.insert(format!("{}__idx", name), idx.to_string());
        }
        variables.insert(name, resp.value);
    }
    Ok(variables)
}

pub fn ask_variable_value(req: VariableRequest) -> Result<VariableResponse> {
    if req.values.is_empty() {
        let mut input = Input::new();
        if let Some(default_value) = req.default_value {
            input.default(default_value.value);
        }
        let value = input
            .with_prompt(&req.prompt)
            .interact()
            .context(crate::Io {})?;
        Ok(VariableResponse { value, idx: None })
    } else {
        let mut input = Select::new();
        input
            .with_prompt(&req.prompt)
            .items(&req.values)
            .paged(true);
        if let Some(default_value) = req.default_value.and_then(|v| v.idx) {
            input.default(default_value);
        }
        let idx = input.interact().context(crate::Io {})?;
        Ok(VariableResponse {
            value: req.values[idx].clone(),
            idx: Some(idx),
        })
    }
}

//TODO add flag to filter display: all, changes, none
pub fn confirm_plan(ctx: &Ctx, actions: &[Action]) -> Result<bool> {
    write_title("Plan to execute")?;
    debug!(ctx.logger, "plan"; "actions" => ?actions);
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
