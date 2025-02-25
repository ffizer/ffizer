mod ignore_cfg;
mod import_cfg;
mod script_cfg;
mod template_cfg;
mod template_composite;
mod transform_values;
mod variable_cfg;

pub(crate) use import_cfg::*;
pub(crate) use template_cfg::*;
pub(crate) use template_composite::*;
pub(crate) use transform_values::*;
pub(crate) use variable_cfg::*;

use crate::Result;
use crate::path_pattern::PathPattern;
use crate::scripts::Script;
use crate::source_loc::SourceLoc;
use crate::source_uri::SourceUri;
use crate::timeline::FFIZER_DATASTORE_DIRNAME;
use std::path::PathBuf;
use std::str::FromStr;

const TEMPLATE_CFG_FILENAME: &str = ".ffizer.yaml";
pub const TEMPLATE_SAMPLES_DIRNAME: &str = ".ffizer.samples.d";
const DEFAULTS_IGNORE: [&str; 3] = [
    TEMPLATE_CFG_FILENAME,
    FFIZER_DATASTORE_DIRNAME,
    TEMPLATE_SAMPLES_DIRNAME,
];

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
        ignores.extend(
            DEFAULTS_IGNORE
                .iter()
                .map(|x| PathPattern::from_str(x))
                .collect::<Result<Vec<PathPattern>>>()?,
        );
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
