mod tree;

use crate::cli_opt::*;
use crate::error::*;
use crate::variable_def::VariableDef;
use crate::FileOperation;
use crate::{Action, Ctx, Variables};
use console::Style;
use console::Term;
use dialoguer::Confirm;
use dialoguer::Input;
use dialoguer::Select;
use handlebars_misc_helpers::new_hbs;
use lazy_static::lazy_static;
use serde_yaml::Value;
use std::borrow::Cow;
use tracing::debug;

lazy_static! {
    static ref TERM: Term = Term::stdout();
    static ref TITLE_STYLE: Style = Style::new().bold();
}

fn write_title(s: &str) -> Result<()> {
    TERM.write_line(&format!("\n\n{}\n", TITLE_STYLE.apply_to(s)))?;
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
    list_variables: &[VariableDef],
    mut init: Variables,
) -> Result<Variables> {
    let mut variables = Variables::default();
    variables.append(&mut init);
    let handlebars = new_hbs();
    write_title("Configure variables")?;
    // TODO optimize to reduce clones
    for variable in list_variables.iter().cloned() {
        let name = variable.name;
        if variables.contains_key(&name) {
            continue;
        }
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
                .map(|v| serde_yaml::from_value(v.clone()).map_err(Error::from))
                .collect::<Result<Vec<String>>>()?;
            //TODO ValuesForSelection::Empty => vec![],
            // ValuesForSelection::Sequence(v) => v.clone(),
            // ValuesForSelection::String(s) => {
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
            // }
            // };
            let default_value = variable
                .default_value
                .and_then(|default_value| match default_value {
                    Value::String(ref v) => Some(format!("\"{}\"", v)),
                    Value::Bool(ref v) => Some(format!("{}", v)),
                    Value::Number(ref v) => Some(format!("{}", v)),
                    _ => None,
                })
                .and_then(|tmpl| {
                    handlebars
                        .render_template(&tmpl, &variables)
                        //TODO better manage error
                        // .context(crate::Handlebars {
                        //     when: format!("define default_value for '{}'", &name),
                        //     template: tmpl,
                        // })
                        .ok()
                })
                .map(|value| {
                    let idx = values
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| if v == &value { Some(i) } else { None })
                        .next();
                    VariableResponse { value, idx }
                });
            VariableRequest {
                prompt,
                values,
                default_value,
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
        }
        variables.insert(
            name.clone(),
            Variables::value_from_str(&resp.value).map_err(|_source| Error::ReadVariable {
                name,
                value: resp.value.clone(),
            })?,
        )?;
    }
    Ok(variables)
}

pub fn ask_variable_value(req: VariableRequest) -> Result<VariableResponse> {
    if req.values.is_empty() {
        let mut input = Input::new();
        if let Some(default_value) = req.default_value {
            input.default(default_value.value);
        }
        let value = input.with_prompt(&req.prompt).interact()?;
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
        Confirm::new()
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
    let mut input = Select::new();
    input
        .with_prompt(&format!(
            "Modification of {:?} (use arrow + return to select option)",
            local.as_ref()
        ))
        .items(
            &values
                .iter()
                .map(|v| format!("{} - {}", v.1.to_string(), v.0))
                .collect::<Vec<_>>(),
        )
        .default(0)
        .paged(false);
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
        Confirm::new()
            .with_prompt("Do you want to run the commands ?")
            .interact()
            .map_err(Error::from)
    }
}
