use super::transform_values::TransformsValues;
use crate::Result;

#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
pub(crate) struct ImportCfg {
    pub uri: String,
    pub rev: Option<String>,
    pub subfolder: Option<String>,
}

impl TransformsValues for ImportCfg {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let uri = self.uri.transforms_values(render)?;
        let rev = self.rev.transforms_values(render)?;
        let subfolder = self.subfolder.transforms_values(render)?;
        Ok(ImportCfg {
            uri,
            rev,
            subfolder,
        })
    }
}
