use super::transform_values::TransformsValues;
use crate::Result;

#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
pub(crate) struct ScriptCfg {
    pub(crate) message: Option<String>,
    pub(crate) cmd: Option<String>,
}

impl TransformsValues for ScriptCfg {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let message = self.message.transforms_values(render)?;
        let cmd = self.cmd.transforms_values(render)?;
        Ok(ScriptCfg { message, cmd })
    }
}
