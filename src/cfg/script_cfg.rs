use super::transform_values::TransformsValues;
use crate::Result;
use schemars::JsonSchema;

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, JsonSchema)]
pub(crate) struct ScriptCfg {
    /// message to display
    pub(crate) message: Option<String>,
    /// command to execute
    pub(crate) cmd: Option<String>,
    /// default answer for confirmation of execution
    pub(crate) default_confirm_answer: Option<String>,
}

impl TransformsValues for ScriptCfg {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let message = self.message.transforms_values(render)?;
        let cmd = self.cmd.transforms_values(render)?;
        let default_confirm_answer = self.default_confirm_answer.transforms_values(render)?;
        Ok(ScriptCfg {
            message,
            cmd,
            default_confirm_answer,
        })
    }
}
