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

    pub fn to_value(s: &str) -> Result<serde_yaml::Value> {
        //serde_yaml::to_value(value).context(crate::SerdeYaml {})
        serde_yaml::from_str::<serde_yaml::Value>(s).context(crate::SerdeYaml {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test_to_value() {
        assert_that!(&Variables::to_value("v1").unwrap())
            .is_equal_to(&serde_yaml::Value::String("v1".to_owned()));
        assert_that!(&Variables::to_value("true").unwrap())
            .is_equal_to(&serde_yaml::Value::Bool(true));
        assert_that!(&Variables::to_value("false").unwrap())
            .is_equal_to(&serde_yaml::Value::Bool(false));
        assert_that!(&Variables::to_value("\"true\"").unwrap())
            .is_equal_to(&serde_yaml::Value::String("true".to_owned()));
        assert_that!(&Variables::to_value("42").unwrap())
            .is_equal_to(&serde_yaml::to_value(42).unwrap());
    }
}
