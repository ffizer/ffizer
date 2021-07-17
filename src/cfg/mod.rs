mod ignore_cfg;
mod import_cfg;
mod script_cfg;
mod template_cfg;
mod template_composite;
mod transform_values;
mod variable_cfg;

pub(crate) use template_composite::*;

use crate::path_pattern::PathPattern;
use crate::scripts::Script;
use crate::source_loc::SourceLoc;
use crate::source_uri::SourceUri;
use crate::variable_def::VariableDef;
use crate::Result;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::instrument;

const TEMPLATE_CFG_FILENAME: &str = ".ffizer.yaml";
pub const TEMPLATE_SAMPLES_DIRNAME: &str = ".ffizer.samples.d";

impl template_cfg::TemplateCfg {
    pub(crate) fn find_ignores(&self) -> Result<Vec<PathPattern>> {
        let mut ignores = self
            .ignores
            .iter()
            .filter(|v| !v.is_empty())
            .map(|v| PathPattern::from_str(v.as_str()))
            .collect::<Result<Vec<PathPattern>>>()?;
        let cfg_pattern = PathPattern::from_str(TEMPLATE_CFG_FILENAME)?;
        ignores.push(cfg_pattern);
        let samples_pattern = PathPattern::from_str(TEMPLATE_SAMPLES_DIRNAME)?;
        ignores.push(samples_pattern);
        Ok(ignores)
    }

    pub(crate) fn find_variabledefs(&self) -> Result<Vec<VariableDef>> {
        self.variables.iter().map(|v| to_variabledef(v)).collect()
    }

    pub(crate) fn find_scripts(&self) -> Result<Vec<Script>> {
        Ok(self
            .scripts
            .iter()
            .map(|v| Script {
                message: v.message.clone().filter(|x| !x.is_empty()),
                cmd: v.cmd.clone().filter(|x| !x.is_empty()),
            })
            .collect())
    }

    pub(crate) fn find_sourcelocs(&self) -> Result<Vec<SourceLoc>> {
        self.imports
            .iter()
            .map(|v| {
                let uri = SourceUri::from_str(v.uri.as_str())?;
                let subfolder = v.subfolder.as_ref().map(|x| PathBuf::from(x.as_str()));
                let rev = v.rev.as_deref().unwrap_or("master").to_owned();
                Ok(SourceLoc {
                    uri,
                    rev,
                    subfolder,
                })
            })
            .collect()
    }
}

#[instrument]
fn to_variabledef(v: &variable_cfg::VariableCfg) -> Result<VariableDef> {
    let hidden: bool = match v.hidden {
        None => false,
        Some(ref v) => serde_yaml::from_str(&v)?,
    };
    let select_in_values: Vec<serde_yaml::Value> = match &v.select_in_values {
        None => vec![],
        Some(v) => match &v.0 {
            serde_yaml::Value::String(ref s) => serde_yaml::from_str(&s)?,
            // serde_yaml::Value::Sequence(ref s) => serde_yaml::Value::Sequence(s.clone()),
            s => serde_yaml::from_value(s.clone())?,
        },
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

pub fn provide_json_schema() -> Result<String> {
    let schema = schemars::schema_for!(template_cfg::TemplateCfg);
    Ok(serde_json::to_string_pretty(&schema)?)
}
