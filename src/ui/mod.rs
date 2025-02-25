mod tree;

use crate::FileOperation;
use crate::cfg::TransformsValues;
use crate::cfg::VariableCfg;
use crate::cli_opt::*;
use crate::error::*;
use crate::variable_def::LabelValue;
use crate::variable_def::VariableDef;
use crate::{Action, Ctx, Variables};
use cliclack::confirm;
use cliclack::input;
use cliclack::note;
use cliclack::select;
use console::Style;
use handlebars_misc_helpers::new_hbs;
use std::borrow::Cow;
use tracing::{Level, debug, instrument, span, warn};

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
        Option::None => false,
        Some(ref v) => serde_yaml::from_str(v)?,
    };
    let select_in_values: Vec<LabelValue> = match &v.select_in_values {
        Option::None => vec![],
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

pub(crate) fn intro(title: &str) -> Result<()> {
    cliclack::intro(title).map_err(Error::from)
}

pub(crate) fn outro(message: &str) -> Result<()> {
    cliclack::outro(message).map_err(Error::from)
}

pub(crate) fn ask_variables(
    ctx: &Ctx,
    list_variables: &[VariableCfg],
    mut init: Variables,
) -> Result<Variables> {
    let mut variables = Variables::default();
    variables.append(&mut init);
    let handlebars = new_hbs();

    intro("Configure variables")?;
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
                        template: Box::new(ask.clone()),
                        source: Box::new(source),
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
            Some(v) if v.value == "true" || v.value == "false" => confirm(&req.prompt)
                .initial_value(v.value == "true")
                .interact()
                .map(|r| r.to_string())?,
            _ => {
                let mut input = input(&req.prompt);
                if let Some(default_value) = req.default_value {
                    input = input.default_input(&default_value.value);
                }
                input.interact()?
            }
        };
        Ok(VariableResponse { value, idx: None })
    } else {
        let mut input = select(&req.prompt).items(
            req.values
                .iter()
                .map(|v| (v.clone(), v.clone(), ""))
                .collect::<Vec<_>>()
                .as_slice(),
        );
        if let Some(default_value) = req.default_value {
            let ivalue = default_value.value.clone();
            input = input.initial_value(ivalue);
        }
        let selected = input.interact()?;
        Ok(VariableResponse {
            value: selected.to_string(),
            idx: req.values.iter().position(|v| v == &selected),
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
    debug!(?actions, "plan");
    let prefixes = tree::provide_prefix(actions, |parent, item| {
        Some(parent.dst_path.relative.as_path()) == item.dst_path.relative.parent()
    });
    let mut plan = String::new();
    for (a, prefix) in actions.iter().zip(prefixes.iter()) {
        let p = a.dst_path.base.join(&a.dst_path.relative);
        plan.push_str(&format!(
            "   - {} \x1B[38;2;{};{};{}m{}\x1B[0m{}\n",
            format_operation(&a.operation),
            80,
            80,
            80,
            prefix,
            p.file_name().and_then(|v| v.to_str()).unwrap_or("???"),
        ));
    }
    note("Plan to execute", plan)?;
    let r = if ctx.cmd_opt.confirm == AskConfirmation::Always {
        confirm("Do you want to apply plan ?").interact()?
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
    use std::fs;
    let local_str = fs::read_to_string(&local)?;
    let remote_str = fs::read_to_string(&remote)?;
    show_difference_text(&local_str, &remote_str, false);
    Ok(())
}

pub fn show_difference_text(old: &str, new: &str, show_whitespace: bool) {
    use console::style;
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(old, new);
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("...");
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                print!(
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                );
                for (emphasized, value) in change.iter_strings_lossy() {
                    let value = if show_whitespace {
                        replace_blank_char(&value)
                    } else {
                        value.to_string()
                    };
                    if emphasized {
                        print!("{}", s.apply_to(value).underlined().on_black());
                    } else {
                        print!("{}", s.apply_to(value));
                    }
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }
}

fn replace_blank_char(s: &str) -> String {
    s.replace(' ', "·")
        .replace('\t', "⇒\t")
        .replace("\r\n", "¶\n")
        .replace('\n', "↩\n")
}

struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            Option::None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub fn ask_update_mode<P>(local: P) -> Result<UpdateMode>
where
    P: AsRef<std::path::Path>,
{
    // let values = UpdateMode::variants();
    let values = [
        //("ask what to do", UpdateMode::Ask),
        ("show diff then ask", UpdateMode::ShowDiff),
        (
            "keep existing local file (ignore template)",
            UpdateMode::Keep,
        ),
        (
            "override local file with file from template",
            UpdateMode::Override,
        ),
        (
            "keep existing local file, add template with extension .REMOTE",
            UpdateMode::UpdateAsRemote,
        ),
        (
            "rename existing local file with extension .LOCAL, add template file",
            UpdateMode::CurrentAsLocal,
        ),
        (
            "try to merge existing local with remote template via merge tool (defined in the git's configuration)",
            UpdateMode::Merge,
        ),
    ];
    let mut input = select(format!(
        "Modification of {:?} (use arrow + return to select option)",
        local.as_ref()
    ))
    .items(
        &values
            .iter()
            .map(|v| (v.1.clone(), format!("{} - {}", v.1, v.0), ""))
            .collect::<Vec<_>>(),
    );
    let selected = input.interact()?;

    Ok(selected)
}

pub fn ask_to_update_sample(msg: &str) -> Result<bool> {
    confirm(msg).interact().map_err(Error::from)
}

pub fn show_message(
    _ctx: &Ctx,
    template_name: impl std::fmt::Display,
    message: impl std::fmt::Display,
) -> Result<()> {
    note(
        "",
        format!("\n message from template: {}\n\t{}", template_name, message),
    )
    .map_err(Error::from)
}

pub fn confirm_run_script(
    ctx: &Ctx,
    template_name: impl std::fmt::Display,
    script: impl std::fmt::Display,
    default_confirm_answer: bool,
) -> Result<bool> {
    note(
        "Run script",
        format!(
            "\n command to run:\n\t from template: {}\n\t commands:\n{}",
            template_name, script,
        ),
    )?;
    if ctx.cmd_opt.no_interaction {
        Ok(default_confirm_answer)
    } else {
        confirm("Do you want to run the commands ?")
            .initial_value(default_confirm_answer)
            .interact()
            .map_err(Error::from)
    }
}
