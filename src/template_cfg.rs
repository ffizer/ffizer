use failure::Error;
use globset::{Glob, GlobMatcher};
use std::fs;
use std::path::Path;

const TEMPLATE_CFG_FILENAME: &'static str = ".ffizer.yaml";

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(deny_unknown_fields, default)]
pub struct TemplateCfg {
    pub variables: Vec<Variable>,
    #[serde(rename = "ignores")]
    ignores_str: Vec<String>,
    #[serde(skip)]
    pub ignores: Vec<GlobMatcher>,
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
        self.ignores = self
            .ignores_str
            .iter()
            .map(|s| Glob::new(s).expect("TODO").compile_matcher())
            .collect::<Vec<_>>();
        self.ignores.push(
            Glob::new(TEMPLATE_CFG_FILENAME)
                .expect("TODO")
                .compile_matcher(),
        );
        Ok(self)
    }

    /// transforms default_value & ignore
    pub fn transforms_values<F>(&self, mut render: F) -> Result<TemplateCfg, Error>
    where
        F: FnMut(&str) -> String,
    {
        let variables = self
            .variables
            .iter()
            .map(|v| {
                let mut nv = v.clone();
                nv.default_value = nv.default_value.map(|s| render(&s));
                nv
            }).collect::<Vec<_>>();
        let ignores_str = self
            .ignores_str
            .iter()
            .map(|s| render(s))
            .collect::<Vec<_>>();
        let cfg = TemplateCfg {
            variables,
            ignores_str,
            ignores: vec![],
        };
        cfg.post_load()
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
              default_value: transformed
            - name: k3
        "#;
        let cfg_in = TemplateCfg::from_str(&cfg_in_str).unwrap();
        let expected = TemplateCfg::from_str(&cfg_expected_str).unwrap();
        let actual = cfg_in
            .transforms_values(|v| {
                if v == "to_transform" {
                    "transformed".to_owned()
                } else {
                    v.to_string()
                }
            }).unwrap();
        assert_that!(&actual.variables).is_equal_to(&expected.variables);
        assert_that!(&actual.ignores_str).is_equal_to(&expected.ignores_str);
        //assert_that!(&actual.ignores).is_equal_to(&expected.ignores);
    }
}
