#[derive(Debug, Clone, Default, PartialEq)]
pub struct VariableDef {
    /// name of variable used in the template
    pub name: String,
    /// optionnal default value
    pub default_value: Option<serde_yaml::Value>,
    /// sentence to ask the value (default to the name on variable)
    pub ask: Option<String>,
    /// is the variable hidden to the user (could be usefull to cache shared variable/data)
    pub hidden: bool,
    /// if non-empty then the value should selected into the list of value
    pub select_in_values: Vec<serde_yaml::Value>,
}
