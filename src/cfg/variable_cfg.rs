use super::transform_values::TransformsValues;
use crate::variable_def::LabelValue;
use crate::Result;
use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;
use schemars::JsonSchema;

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq, JsonSchema)]
pub struct VariableCfg {
    /// name of variable used in the template
    pub name: String,
    /// optionnal default value
    pub default_value: Option<VariableValueCfg>,
    /// sentence to ask the value (default to the name on variable)
    pub ask: Option<String>,
    /// is the variable hidden to the user (could be usefull to cache shared variable/data)
    pub hidden: Option<String>,
    /// if non-empty then the value should selected into the list of value
    pub select_in_values: Option<VariableValuesCfg>,
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

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
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

impl From<&VariableValueCfg> for LabelValue {
    fn from(v: &VariableValueCfg) -> Self {
        LabelValue {
            label: serde_yaml::to_string(&v.0)
                .expect("to be able to serde_yaml::to_string a yaml value"),
            value: v.0.to_owned(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq, JsonSchema)]
pub struct LabelValueCfg {
    /// display of the value (in select)
    pub label: String,
    /// the value
    pub value: VariableValueCfg,
}

impl TransformsValues for LabelValueCfg {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        Ok(LabelValueCfg {
            label: self.label.transforms_values(render)?,
            value: self.value.transforms_values(render)?,
        })
    }
}

impl From<&LabelValueCfg> for LabelValue {
    fn from(v: &LabelValueCfg) -> Self {
        LabelValue {
            label: v.label.to_owned(),
            value: v.value.0.to_owned(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(untagged)] // order in enum is the priority order to find the right subtype
pub enum VariableValuesCfg {
    ListLV(Vec<LabelValueCfg>),
    ListV(Vec<VariableValueCfg>),
    String(String),
}

impl From<&VariableValuesCfg> for Vec<LabelValue> {
    fn from(v: &VariableValuesCfg) -> Self {
        match v {
            VariableValuesCfg::ListLV(l) => l.iter().map(|v| v.into()).collect(),
            VariableValuesCfg::ListV(l) => l.iter().map(|v| v.into()).collect(),
            VariableValuesCfg::String(s) => vec![LabelValue {
                label: s.to_owned(),
                value: serde_yaml::Value::String(s.to_owned()),
            }],
        }
    }
}

impl TransformsValues for VariableValuesCfg {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let v = match &self {
            Self::String(s) => serde_yaml::from_str(s.transforms_values(render)?.as_str())?,
            Self::ListV(s) => Self::ListV(s.transforms_values(render)?),
            Self::ListLV(s) => Self::ListLV(s.transforms_values(render)?),
        };
        Ok(v)
    }
}
