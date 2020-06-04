use super::transform_values::TransformsValues;
use crate::Result;

#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
pub(crate) struct ScriptCfg {
    pub(crate) cmd: String,
}

impl TransformsValues for ScriptCfg {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let cmd = self.cmd.transforms_values(render)?;
        Ok(ScriptCfg { cmd })
    }
}
