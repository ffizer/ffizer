use super::transform_values::TransformsValues;
use crate::Result;
use snafu::ResultExt;
use std::fs;
use std::path::Path;

use super::ignore_cfg::IgnoreCfg;
use super::import_cfg::ImportCfg;
use super::script_cfg::ScriptCfg;
use super::variable_cfg::VariableCfg;

#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct TemplateCfg {
    pub(crate) variables: Vec<VariableCfg>,
    pub(crate) ignores: Vec<IgnoreCfg>,
    pub(crate) imports: Vec<ImportCfg>,
    pub(crate) scripts: Vec<ScriptCfg>,
    // set to true if the template content is under a `template` folder (not mixed with metadata)
    pub(crate) use_template_dir: bool,
}

impl TemplateCfg {
    pub fn from_str<S>(str: S) -> Result<TemplateCfg>
    where
        S: AsRef<str>,
    {
        //let cfg = serde_json::from_str::<TemplateCfg>(str.as_ref())?;
        serde_yaml::from_str::<TemplateCfg>(str.as_ref()).context(crate::SerdeYaml {})
    }

    pub fn from_template_folder(template_base: &Path) -> Result<TemplateCfg> {
        let cfg_path = template_base.join(super::TEMPLATE_CFG_FILENAME);
        if cfg_path.exists() {
            let cfg_str = fs::read_to_string(cfg_path).context(crate::Io {})?;
            Self::from_str(cfg_str)
        } else {
            Ok(TemplateCfg::default())
        }
    }
}

impl TransformsValues for TemplateCfg {
    /// transforms ignore, imports
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let variables = self.variables.clone();
        let ignores = self.ignores.transforms_values(render)?;
        let imports = self.imports.transforms_values(render)?;
        let scripts = self.scripts.transforms_values(render)?;
        Ok(TemplateCfg {
            variables,
            ignores,
            imports,
            scripts,
            use_template_dir: self.use_template_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
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
    fn various_assert_equals() {
        let v1_0 = Some(serde_yaml::to_value("v1").expect("yaml parsed"));
        let v1_1 = Some(serde_yaml::to_value("v1").expect("yaml parsed"));
        let v2_0 = Some(serde_yaml::to_value("v2").expect("yaml parsed"));
        assert_that!(&v1_0).is_equal_to(&v1_0);
        assert_that!(&v1_1).is_equal_to(&v1_0);
        assert_that!(&v2_0).is_not_equal_to(&v1_0);
    }

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
        expected.variables.push(VariableCfg {
            name: "k2".to_owned(),
            default_value: Some(serde_yaml::to_value("v2").expect("yaml parsed")),
            ..Default::default()
        });
        expected.variables.push(VariableCfg {
            name: "k1".to_owned(),
            default_value: Some(serde_yaml::to_value("V1").expect("yaml parsed")),
            ..Default::default()
        });
        expected.variables.push(VariableCfg {
            name: "k3".to_owned(),
            ..Default::default()
        });
        let actual = serde_yaml::from_str::<TemplateCfg>(&cfg_str).unwrap();
        assert_that!(&actual.variables).is_equal_to(&expected.variables);
        assert_that!(&actual.use_template_dir).is_false();
    }
    //TODO
    // #[test]
    // fn test_deserialize_cfg_yaml_select() {
    //     let cfg_str = r#"
    //     variables:
    //         - name: k2
    //           select_in_values:
    //             - vk21
    //             - vk22
    //         - name: k1
    //           select_in_values: [ "vk11", "vk12" ]
    //         - name: k3
    //           select_in_values: '[ "vk31", "vk32" ]'
    //         - name: k4
    //           select_in_values: '{{ do_stuff }}'
    //     "#;

    //     let mut variables = vec![];
    //     variables.push(VariableCfg {
    //         name: "k2".to_owned(),
    //         select_in_values: vec![
    //             serde_yaml::Value::String("vk21".to_owned()),
    //             serde_yaml::Value::String("vk22".to_owned()),
    //         ],
    //         ..Default::default()
    //     });
    //     variables.push(VariableDef {
    //         name: "k1".to_owned(),
    //         select_in_values: vec![
    //             serde_yaml::Value::String("vk11".to_owned()),
    //             serde_yaml::Value::String("vk12".to_owned()),
    //         ],
    //         ..Default::default()
    //     });
    //     variables.push(VariableDef {
    //         name: "k3".to_owned(),
    //         select_in_values: ValuesForSelection::String("[ \"vk31\", \"vk32\" ]".to_owned()),
    //         ..Default::default()
    //     });
    //     expected.variables.push(VariableDef {
    //         name: "k4".to_owned(),
    //         select_in_values: ValuesForSelection::String("{{ do_stuff }}".to_owned()),
    //         ..Default::default()
    //     });
    //     let actual = serde_yaml::from_str::<TemplateCfg>(&cfg_str).unwrap();
    //     assert_that!(&actual.variables).is_equal_to(&variables);
    //     assert_that!(&actual.use_template_dir).is_false();
    // }

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
        scripts:
            - cmd: hello to_transform
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
        scripts:
            - cmd: hello transformed
        "#;
        let cfg_in = TemplateCfg::from_str(&cfg_in_str).unwrap();
        let expected = TemplateCfg::from_str(&cfg_expected_str).unwrap();
        let render = |v: &str| v.replace("to_transform", "transformed");
        let actual = cfg_in.transforms_values(&render).unwrap();
        assert_eq!(&actual, &expected);
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

    #[test]
    fn test_accept_ignores_with_values() {
        let cfg_in_str = r#"
        ignores:
            - "{{foo}}"
            - '{{#if (eq k1 "foo")}}foo{{/if}}'
        variables:
            - name: k1
        "#;
        TemplateCfg::from_str(&cfg_in_str).unwrap();
    }
}
