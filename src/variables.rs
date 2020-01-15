use crate::error::*;
use serde::Serialize;
use snafu::ResultExt;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variables(BTreeMap<String, serde_yaml::Value>);

impl Variables {
    pub fn new() -> Self {
        Variables(BTreeMap::new())
    }

    pub fn append(&mut self, v: &mut Variables) {
        self.0.append(&mut v.0);
    }

    pub fn insert<K: Into<String>, V: Serialize>(&mut self, key: K, value: V) -> Result<()> {
        self.0.insert(
            key.into(),
            serde_yaml::to_value(value).context(crate::SerdeYaml {})?,
        );
        Ok(())
    }
}
