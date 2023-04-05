mod ignore_cfg;
mod import_cfg;
mod script_cfg;
mod template_cfg;
mod template_composite;
mod transform_values;
mod variable_cfg;

pub(crate) use template_composite::*;
pub(crate) use transform_values::*;
pub(crate) use variable_cfg::*;

use crate::path_pattern::PathPattern;
use crate::scripts::Script;
use crate::source_loc::SourceLoc;
use crate::source_uri::SourceUri;
use crate::Result;
use std::path::PathBuf;
use std::str::FromStr;

const TEMPLATE_CFG_FILENAME: &str = ".ffizer.yaml";
pub const TEMPLATE_SAMPLES_DIRNAME: &str = ".ffizer.samples.d";

impl template_cfg::TemplateCfg {
    pub(crate) fn find_ignores(&self) -> Result<Vec<PathPattern>> {
        let trim_chars: &[_] = &['\r', '\n', ' ', '\t', '"', '\''];
        let mut ignores = self
            .ignores
            .iter()
            .map(|v| v.trim_matches(trim_chars))
            .filter(|v| !v.is_empty())
            .map(PathPattern::from_str)
            .collect::<Result<Vec<PathPattern>>>()?;
        let cfg_pattern = PathPattern::from_str(TEMPLATE_CFG_FILENAME)?;
        ignores.push(cfg_pattern);
        let samples_pattern = PathPattern::from_str(TEMPLATE_SAMPLES_DIRNAME)?;
        ignores.push(samples_pattern);
        Ok(ignores)
    }

    pub(crate) fn find_scripts(&self) -> Result<Vec<Script>> {
        Ok(self
            .scripts
            .iter()
            .map(|v| Script {
                message: v.message.clone().filter(|x| !x.is_empty()),
                cmd: v.cmd.clone().filter(|x| !x.is_empty()),
                default_confirm_answer: v
                    .default_confirm_answer
                    .clone()
                    .map(|x| x.parse::<bool>().unwrap_or(false))
                    .unwrap_or_default(),
            })
            .collect())
    }

    pub(crate) fn find_sourcelocs(&self) -> Result<Vec<SourceLoc>> {
        self.imports
            .iter()
            .map(|v| {
                let uri = SourceUri::from_str(v.uri.as_str())?;
                let subfolder = v.subfolder.as_ref().map(|x| PathBuf::from(x.as_str()));
                let rev = v.rev.clone();
                Ok(SourceLoc {
                    uri,
                    rev,
                    subfolder,
                })
            })
            .collect()
    }
}

pub fn provide_json_schema() -> Result<String> {
    let schema = schemars::schema_for!(template_cfg::TemplateCfg);
    Ok(serde_json::to_string_pretty(&schema)?)
}
