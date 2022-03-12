mod tree;

use crate::cfg::TransformsValues;
use crate::cfg::VariableCfg;
use crate::cli_opt::*;
use crate::error::*;
use crate::variable_def::LabelValue;
use crate::variable_def::VariableDef;
use crate::FileOperation;
use crate::{Action, Ctx, Variables};
use console::Style;
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use dialoguer::Input;
use dialoguer::Select;
use handlebars_misc_helpers::new_hbs;
use lazy_static::lazy_static;
use std::borrow::Cow;
use tracing::{debug, instrument, span, warn, Level};

lazy_static! {
    static ref TERM: Term = Term::stdout();
    static ref TITLE_STYLE: Style = Style::new().bold();
    static ref PROMPT_THEME: ColorfulTheme = ColorfulTheme::default();
}

fn write_title(s: &str) -> Result<()> {
    TERM.write_line(&format!("\n\n{}\n", TITLE_STYLE.apply_to(s)))?;
    Ok(())
}

#[derive(Debug)]
pub struct VariableResponse {
    value: String,
    idx: Option<usize>,
}

#[derive(Debug)]
pub struct VariableRequest {
    prompt: String,
    default_value: Option<VariableResponse>,
    values: Vec<String>,
}

#[instrument]
fn to_variabledef(v: &VariableCfg) -> Result<VariableDef> {
    let hidden: bool = match v.hidden {
        None => false,
        Some(ref v) => serde_yaml::from_str(v)?,
    };
    let select_in_values: Vec<LabelValue> = match &v.select_in_values {
        None => vec![],
        Some(v) => v.into(),
    };

    //     let s_evaluated =
    //         handlebars
    //             .render_template(&s, &variables)
    //             .context(crate::Handlebars {
    //                 when: format!("define values for '{}'", &name),
    //                 template: s.clone(),
    //             })?;
    //     let s_values: Vec<String> =
    //         serde_yaml::from_str(&s_evaluated).context(crate::SerdeYaml {})?;
    //     //dbg!(&s_values);
    //     s_values

    Ok(VariableDef {
        name: v.name.clone(),
        default_value: v.default_value.as_ref().map(|v| v.0.clone()),
        ask: v.ask.clone(),
        hidden,
        select_in_values,
    })
}

pub(crate) fn ask_variables(
    ctx: &Ctx,
    list_variables: &[VariableCfg],
    mut init: Variables,
) -> Result<Variables> {
    let mut variables = Variables::default();
    variables.append(&mut init);
    let handlebars = new_hbs();

    write_title("Configure variables")?;
    // TODO optimize to reduce clones
    for variable_cfg in list_variables.iter().cloned() {
        let _span_ = span!(Level::DEBUG, "ask_variables", ?variable_cfg).entered();
        if variables.contains_key(&variable_cfg.name) {
            continue;
        }
        let render = |v: &str| {
            let r = handlebars.render_template(v, &variables);
            match r {
                Ok(s) => s,
                Err(e) => {
                    warn!(input = ?v, error = ?e, "failed to convert");
                    v.into()
                }
            }
        };
        let variable_cfg = variable_cfg.transforms_values(&render)?;
        let variable = to_variabledef(&variable_cfg)?;
        let name = variable.name;
        let request = {
            let prompt = if variable.ask.is_some() {
                let ask = variable.ask.expect("variable ask should defined");
                handlebars
                    .render_template(&ask, &variables)
                    .map_err(|source| Error::Handlebars {
                        when: format!("define prompt for '{}'", &name),
                        template: ask.clone(),
                        source,
                    })?
            } else {
                name.clone()
            };
            let values: Vec<String> = variable
                .select_in_values
                .iter()
                .map(|v| v.label.to_string())
                .collect::<Vec<String>>();
            let default_value = variable
                .default_value
                .and_then(|default_value| Variables::value_as_str(&default_value).ok())
                .map(|value| {
                    let idx = variable
                        .select_in_values
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| {
                            if v.label == value
                                || Variables::value_as_str(&v.value)
                                    .map(|v| v == value)
                                    .unwrap_or(false)
                            {
                                Some(i)
                            } else {
                                None
                            }
                        })
                        .next();
                    VariableResponse { value, idx }
                });
            VariableRequest {
                prompt,
                default_value,
                values,
            }
        };
        let resp = if variable.hidden || ctx.cmd_opt.no_interaction {
            request.default_value.unwrap_or(VariableResponse {
                value: "".to_owned(),
                idx: None,
            })
        } else {
            ask_variable_value(request)?
        };
        if let Some(idx) = resp.idx {
            variables.insert(format!("{}__idx", name), idx)?;
            variables.insert(format!("{}__label", name), resp.value)?;
            variables.insert(
                name.clone(),
                variable
                    .select_in_values
                    .get(idx)
                    .expect("selected should be in the list")
                    .value
                    .clone(),
            )?;
        } else {
            variables.insert(name.clone(), Variables::value_from_str(&resp.value)?)?;
        }
    }
    Ok(variables)
}

pub fn ask_variable_value(req: VariableRequest) -> Result<VariableResponse> {
    if req.values.is_empty() {
        let value = match req.default_value {
            Some(v) if v.value == "true" || v.value == "false" => {
                Confirm::with_theme(&(*PROMPT_THEME))
                    .default(v.value == "true")
                    .with_prompt(&req.prompt)
                    .interact()
                    .map(|r| r.to_string())?
            }
            _ => {
                let mut input = Input::with_theme(&(*PROMPT_THEME));
                if let Some(default_value) = req.default_value {
                    input.default(default_value.value);
                }
                input.with_prompt(&req.prompt).interact()?
            }
        };
        Ok(VariableResponse { value, idx: None })
    } else {
        let mut input = Select::with_theme(&(*PROMPT_THEME));
        input.with_prompt(&req.prompt).items(&req.values);
        if let Some(default_value) = req.default_value.and_then(|v| v.idx) {
            input.default(default_value);
        }
        let idx = input.interact()?;
        Ok(VariableResponse {
            value: req.values[idx].clone(),
            idx: Some(idx),
        })
    }
}

fn format_operation(op: &FileOperation) -> Cow<'static, str> {
    let s = match op {
        FileOperation::Nothing => "do nothing",
        FileOperation::Ignore => "ignore",
        FileOperation::MkDir => "make dir",
        FileOperation::AddFile => "add file",
        FileOperation::UpdateFile => "update file",
    };
    console::pad_str(s, 15, console::Alignment::Left, Some("..."))
}

//TODO add flag to filter display: all, changes, none
pub fn confirm_plan(ctx: &Ctx, actions: &[Action]) -> Result<bool> {
    write_title("Plan to execute")?;
    debug!(?actions, "plan");
    let prefixes = tree::provide_prefix(actions, |parent, item| {
        Some(parent.dst_path.relative.as_path()) == item.dst_path.relative.parent()
    });
    for (a, prefix) in actions.iter().zip(prefixes.iter()) {
        let p = a.dst_path.base.join(&a.dst_path.relative);
        let s = format!(
            "   - {} \x1B[38;2;{};{};{}m{}\x1B[0m{}",
            format_operation(&a.operation),
            80,
            80,
            80,
            prefix,
            p.file_name().and_then(|v| v.to_str()).unwrap_or("???"),
        );
        TERM.write_line(&s)?;
    }
    let r = if ctx.cmd_opt.confirm == AskConfirmation::Always {
        Confirm::with_theme(&(*PROMPT_THEME))
            .with_prompt("Do you want to apply plan ?")
            .interact()?
    } else {
        //TODO implement a algo for auto, like if no change then no ask.
        true
    };
    Ok(r)
}

pub fn show_difference<P>(local: P, remote: P) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    use difference::Changeset;
    use std::fs;
    let local_str = fs::read_to_string(&local)?;
    let remote_str = fs::read_to_string(&remote)?;
    let changeset = Changeset::new(&local_str, &remote_str, "\n");
    println!("{}", changeset);
    Ok(())
}

pub fn ask_update_mode<P>(local: P) -> Result<UpdateMode>
where
    P: AsRef<std::path::Path>,
{
    // let values = UpdateMode::variants();
    let values = vec![
                //("ask what to do", UpdateMode::Ask),
                ("show diff then ask", UpdateMode::ShowDiff),
                ("keep existing local file (ignore template)", UpdateMode::Keep),
                ("override local file with file from template", UpdateMode::Override),
                ("keep existing local file, add template with extension .REMOTE", UpdateMode::UpdateAsRemote),
                ("rename existing local file with extension .LOCAL, add template file", UpdateMode::CurrentAsLocal),
                ("try to merge existing local with remote template via merge tool (defined in the git's configuration)", UpdateMode::Merge),
    ];
    let mut input = Select::with_theme(&(*PROMPT_THEME));
    input
        .with_prompt(&format!(
            "Modification of {:?} (use arrow + return to select option)",
            local.as_ref()
        ))
        .items(
            &values
                .iter()
                .map(|v| format!("{} - {}", v.1, v.0))
                .collect::<Vec<_>>(),
        )
        .default(0);
    let idx = input.interact()?;

    Ok(values[idx].1.clone())
}

pub fn show_message(
    _ctx: &Ctx,
    template_name: impl std::fmt::Display,
    message: impl std::fmt::Display,
) -> Result<()> {
    println!("\n message from template: {}\n\t{}", template_name, message);
    Ok(())
}

pub fn confirm_run_script(
    ctx: &Ctx,
    template_name: impl std::fmt::Display,
    script: impl std::fmt::Display,
) -> Result<bool> {
    // let s = format!(
    //     "   - {} \x1B[38;2;{};{};{}m{}\x1B[0m{}",
    //     format_operation(&a.operation),
    //     80,
    //     80,
    //     80,
    //     prefix,
    //     p.file_name().and_then(|v| v.to_str()).unwrap_or("???"),
    // );
    // TERM.write_line(&s).context(crate::Io {})?;

    println!(
        "\n command to run:\n\t from template: {}\n\t commands:\n{}",
        template_name, script
    );
    if ctx.cmd_opt.no_interaction {
        Ok(true)
    } else {
        Confirm::with_theme(&(*PROMPT_THEME))
            .with_prompt("Do you want to run the commands ?")
            .interact()
            .map_err(Error::from)
    }
}
