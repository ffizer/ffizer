use crate::error::*;
use serde::Serialize;
use std::collections::BTreeMap;
use tracing::instrument;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Variables(BTreeMap<String, serde_yaml::Value>);

impl Variables {
    pub fn append(&mut self, v: &mut Variables) {
        self.0.append(&mut v.0);
    }

    pub fn insert<K: Into<String>, V: Serialize>(&mut self, key: K, value: V) -> Result<()> {
        self.0.insert(key.into(), serde_yaml::to_value(value)?);
        Ok(())
    }

    pub fn contains_key<K: Into<String>>(&mut self, key: K) -> bool {
        self.0.contains_key(&key.into())
    }

    #[instrument]
    pub fn value_from_str(s: &str) -> Result<serde_yaml::Value> {
        serde_yaml::from_str::<serde_yaml::Value>(s).map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test_to_value() {
        assert_that!(&Variables::value_from_str("v1").unwrap())
            .is_equal_to(&serde_yaml::Value::String("v1".to_owned()));
        assert_that!(&Variables::value_from_str("true").unwrap())
            .is_equal_to(&serde_yaml::Value::Bool(true));
        assert_that!(&Variables::value_from_str("false").unwrap())
            .is_equal_to(&serde_yaml::Value::Bool(false));
        assert_that!(&Variables::value_from_str("\"true\"").unwrap())
            .is_equal_to(&serde_yaml::Value::String("true".to_owned()));
        assert_that!(&Variables::value_from_str("42").unwrap())
            .is_equal_to(&serde_yaml::to_value(42).unwrap());
    }
}
