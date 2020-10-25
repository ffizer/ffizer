use super::transform_values::TransformsValues;
use crate::Result;
use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;
use schemars::JsonSchema;

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, JsonSchema)]
pub(crate) struct VariableCfg {
    /// name of variable used in the template
    pub name: String,
    /// optionnal default value
    pub default_value: Option<VariableValueCfg>,
    /// sentence to ask the value (default to the name on variable)
    pub ask: Option<String>,
    /// is the variable hidden to the user (could be usefull to cache shared variable/data)
    pub hidden: Option<String>,
    /// if non-empty then the value should selected into the list of value
    pub select_in_values: Option<VariableValueCfg>,
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

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct VariableValueCfg(pub serde_yaml::Value);

impl JsonSchema for VariableValueCfg {
    //no_ref_schema!();

    fn schema_name() -> String {
        "AnyValue".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        Schema::Bool(true)
    }
}

impl TransformsValues for VariableValueCfg {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        Ok(VariableValueCfg(self.0.transforms_values(render)?))
    }
}
