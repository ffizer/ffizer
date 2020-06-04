use super::transform_values::TransformsValues;
use crate::Result;

#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
pub(crate) struct VariableCfg {
    /// name of variable used in the template
    pub name: String,
    /// optionnal default value
    pub default_value: Option<serde_yaml::Value>,
    /// sentence to ask the value (default to the name on variable)
    pub ask: Option<String>,
    /// is the variable hidden to the user (could be usefull to cache shared variable/data)
    pub hidden: Option<String>,
    /// if non-empty then the value should selected into the list of value
    pub select_in_values: Option<serde_yaml::Value>,
}

impl TransformsValues for VariableCfg {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let name = self.name.transforms_values(render)?;
        let default_value = self.default_value.transforms_values(render)?;
        let ask = self.ask.transforms_values(render)?;
        let hidden = self.hidden.transforms_values(render)?;
        let select_in_values = self.select_in_values.transforms_values(render)?;
        Ok(VariableCfg {
            name,
            default_value,
            ask,
            hidden,
            select_in_values,
        })
    }
}
