use failure::Error;
use std::fs;
use std::path::Path;

const TEMPLATE_CFG_FILENAME: &'static str = ".ffizer.yaml";

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct TemplateCfg {
    pub variables: Vec<Variable>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Variable {
    /// name of variable used in the template
    pub name: String,
    /// optionnal default value
    pub default_value: Option<String>,
    /// sentence to ask the value (default to the name on variable)
    pub ask: Option<String>,
}

impl TemplateCfg {
    pub fn from_str<S>(str: S) -> Result<TemplateCfg, Error>
    where
        S: AsRef<str>,
    {
        let cfg = serde_yaml::from_str::<TemplateCfg>(str.as_ref())?;
        //let cfg = serde_json::from_str::<TemplateCfg>(str.as_ref())?;
        Ok(cfg)
    }

    pub fn from_template_folder(template_base: &Path) -> Result<TemplateCfg, Error> {
        let cfg_path = template_base.join(TEMPLATE_CFG_FILENAME);
        if cfg_path.exists() {
            let cfg_str = fs::read_to_string(cfg_path)?;
            Self::from_str(cfg_str)
        } else {
            Ok(TemplateCfg::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    // TODO provide a PR for https://github.com/dtolnay/serde-yaml/issues/86
    //     #[test]
    //     fn test_from_str_empty() {
    //         let cfg_str = r#"
    // "#;
    //         let expected = TemplateCfg::default();
    //         let actual = TemplateCfg::from_str(cfg_str).unwrap();
    //         assert_that!(&actual).is_equal_to(&expected);
    //     }

    #[test]
    fn test_deserialize_cfg_yaml() {
        let cfg_str = r#"
        variables:
            - name: k2
              default_value: v2
            - name: k1
              default_value: V1
            - name: k3
        "#;
        let mut expected = TemplateCfg::default();
        expected.variables.push(Variable {
            name: "k2".to_owned(),
            default_value: Some("v2".to_owned()),
            ..Default::default()
        });
        expected.variables.push(Variable {
            name: "k1".to_owned(),
            default_value: Some("V1".to_owned()),
            ..Default::default()
        });
        expected.variables.push(Variable {
            name: "k3".to_owned(),
            ..Default::default()
        });
        let actual = serde_yaml::from_str::<TemplateCfg>(&cfg_str).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }
}
