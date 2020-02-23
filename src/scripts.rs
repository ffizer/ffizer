use crate::error::*;
use crate::transform_values::TransformsValues;
use run_script::ScriptOptions;
use snafu::ResultExt;
use std::fmt;
#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(deny_unknown_fields, default)]
pub struct Script {
    cmd: String,
}

impl Script {
    pub(crate) fn run(&self) -> Result<()> {
        if !self.cmd.is_empty() {
            let options = ScriptOptions::new();
            let args = vec![];
            run_script::run(&self.cmd, &args, &options).context(ScriptError {
                script: self.cmd.clone(),
            })?;
        }
        Ok(())
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cmd.fmt(f)
    }
}

impl TransformsValues for Script {
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        Ok(Script {
            cmd: self.cmd.transforms_values(render)?,
        })
    }
}
