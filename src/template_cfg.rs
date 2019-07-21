use crate::path_pattern::PathPattern;
use crate::source_loc::SourceLoc;
use crate::transform_values::TransformsValues;
use failure::Error;
use std::fs;
use std::path::Path;
use std::str::FromStr;

const TEMPLATE_CFG_FILENAME: &str = ".ffizer.yaml";

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct TemplateCfg {
    pub variables: Vec<Variable>,
    pub ignores: Vec<PathPattern>,
    pub imports: Vec<SourceLoc>,
    // set to true if the template content is under a `template` folder (not mixed with metadata)
    pub use_template_dir: bool,
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
        cfg.post_load()
    }

    pub fn from_template_folder(template_base: &Path) -> Result<TemplateCfg, Error> {
        let cfg_path = template_base.join(TEMPLATE_CFG_FILENAME);
        if cfg_path.exists() {
            let cfg_str = fs::read_to_string(cfg_path)?;
            Self::from_str(cfg_str)
        } else {
            TemplateCfg::default().post_load()
        }
    }

    fn post_load(mut self) -> Result<Self, Error> {
        let cfg_pattern = PathPattern::from_str(TEMPLATE_CFG_FILENAME)?;
        self.ignores.push(cfg_pattern);
        Ok(self)
    }
}

impl TransformsValues for TemplateCfg {
    /// transforms ignore, imports
    fn transforms_values<F>(&self, render: &F) -> Result<Self, Error>
    where
        F: Fn(&str) -> String,
    {
        let variables = self.variables.clone();
        let ignores = self.ignores.transforms_values(render)?;
        let imports = self.imports.transforms_values(render)?;
        Ok(TemplateCfg {
            variables,
            ignores,
            imports,
            use_template_dir: self.use_template_dir,
        })
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
        assert_that!(&actual.variables).is_equal_to(&expected.variables);
        assert_that!(&actual.use_template_dir).is_false();
    }

    #[test]
    fn test_transforms_values() {
        let cfg_in_str = r#"
        ignores:
            - keep
            - to_transform
        variables:
            - name: k2
              default_value: v2
            - name: k1
              default_value: to_transform
            - name: k3
        "#;
        let cfg_expected_str = r#"
        ignores:
            - keep
            - transformed
        variables:
            - name: k2
              default_value: v2
            - name: k1
              default_value: to_transform
            - name: k3
        "#;
        let cfg_in = TemplateCfg::from_str(&cfg_in_str).unwrap();
        let expected = TemplateCfg::from_str(&cfg_expected_str).unwrap();
        let render = |v: &str| {
            if v == "to_transform" {
                "transformed".to_owned()
            } else {
                v.to_string()
            }
        };
        let actual = cfg_in.transforms_values(&render).unwrap();
        // variables are transformed on-demand
        assert_that!(&actual.variables).is_equal_to(&expected.variables);
        assert_that!(&actual.ignores).is_equal_to(&expected.ignores);
        //assert_that!(&actual.ignores).is_equal_to(&expected.ignores);
    }

    #[test]
    fn test_deserialize_cfg_yaml_use_template_dir_false() {
        let cfg_str = r#"
        use_template_dir: false
        "#;
        let actual = serde_yaml::from_str::<TemplateCfg>(&cfg_str).unwrap();
        assert_that!(&actual.use_template_dir).is_false();
    }

    #[test]
    fn test_deserialize_cfg_yaml_use_template_dir_true() {
        let cfg_str = r#"
        use_template_dir: true
        "#;
        let actual = serde_yaml::from_str::<TemplateCfg>(&cfg_str).unwrap();
        assert_that!(&actual.use_template_dir).is_true();
    }
}
