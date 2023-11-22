use crate::error::*;
use serde::Serialize;
use std::collections::BTreeMap;
use tracing::instrument;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Variables(BTreeMap<String, serde_yaml::Value>);

impl Variables {
    pub fn append(&mut self, v: &mut Variables) {
        self.0.append(&mut v.0);
    }

    pub fn insert<K: Into<String>, V: Serialize>(&mut self, key: K, value: V) -> Result<()> {
        self.0.insert(key.into(), serde_yaml::to_value(value)?);
        Ok(())
    }

    pub fn contains_key<K: Into<String>>(&self, key: K) -> bool {
        self.0.contains_key(&key.into())
    }

    pub fn tree(&self) -> &BTreeMap<String, serde_yaml::Value> {
        &self.0
    }

    pub fn get<K: Into<String>>(&self, key: K) -> Option<&serde_yaml::Value> {
        self.0.get(&key.into())
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&String, &mut serde_yaml::Value) -> bool,
    {
        self.0.retain(f)
    }

    #[instrument]
    pub fn value_from_str(s: &str) -> Result<serde_yaml::Value> {
        serde_yaml::from_str::<serde_yaml::Value>(s).map_err(Error::from)
    }

    #[instrument]
    pub fn value_as_str(s: &serde_yaml::Value) -> Result<String> {
        serde_yaml::to_string(&s)
            .map(|s| s.trim_start_matches("---").trim().to_owned())
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_value() {
        assert_eq!(
            &serde_yaml::Value::String("v1".to_owned()),
            &Variables::value_from_str("v1").unwrap()
        );
        assert_eq!(
            &serde_yaml::Value::Bool(true),
            &Variables::value_from_str("true").unwrap()
        );
        assert_eq!(
            &serde_yaml::Value::Bool(false),
            &Variables::value_from_str("false").unwrap()
        );
        assert_eq!(
            &serde_yaml::Value::String("true".to_owned()),
            &Variables::value_from_str("\"true\"").unwrap()
        );
        assert_eq!(
            &serde_yaml::to_value(42).unwrap(),
            &Variables::value_from_str("42").unwrap()
        );
    }
}
