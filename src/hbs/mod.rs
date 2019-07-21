use failure::Error;
use handlebars::Handlebars;

pub mod env_helpers;
pub mod http_helpers;
pub mod path_helpers;
pub mod string_helpers;

pub fn new_hbs() -> Result<Handlebars, Error> {
    let mut handlebars = Handlebars::new();
    setup_handlebars(&mut handlebars)?;
    Ok(handlebars)
}

pub fn setup_handlebars(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars.set_strict_mode(true);
    register_all(handlebars)
}

pub fn register_all(handlebars: &mut Handlebars) -> Result<(), Error> {
    string_helpers::register(handlebars)?;
    http_helpers::register(handlebars)?;
    path_helpers::register(handlebars)?;
    env_helpers::register(handlebars)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::Variables;
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test_chain_of_helpers_with_1_param() -> Result<(), Error> {
        let vs = Variables::new();
        let hbs = new_hbs()?;
        let tmpl = r#"{{ to_upper_case (to_singular "Hello foo-bars")}}"#.to_owned();
        let actual = hbs.render_template(&tmpl, &vs)?;
        assert_that!(&actual).is_equal_to("BAR".to_string());
        Ok(())
    }

    fn assert_helpers(input: &str, helper_expected: Vec<(&str, &str)>) -> Result<(), Error> {
        let mut vs = Variables::new();
        vs.insert("var".into(), input.into());
        let hbs = new_hbs()?;
        for sample in helper_expected {
            let tmpl = format!("{{{{ {} var}}}}", sample.0);
            assert_that!(hbs.render_template(&tmpl, &vs)?)
                .named(sample.0)
                .is_equal_to(sample.1.to_owned());
        }
        Ok(())
    }

    #[test]
    fn test_register_string_helpers() -> Result<(), Error> {
        assert_helpers(
            "Hello foo-bars",
            vec![
                ("to_lower_case", "hello foo-bars"),
                ("to_upper_case", "HELLO FOO-BARS"),
                ("to_camel_case", "helloFooBars"),
                ("to_pascal_case", "HelloFooBars"),
                ("to_snake_case", "hello_foo_bars"),
                ("to_screaming_snake_case", "HELLO_FOO_BARS"),
                ("to_kebab_case", "hello-foo-bars"),
                ("to_train_case", "Hello-Foo-Bars"),
                ("to_sentence_case", "Hello foo bars"),
                ("to_title_case", "Hello Foo Bars"),
                ("to_class_case", "HelloFooBar"),
                ("to_table_case", "hello_foo_bars"),
                ("to_plural", "bars"),
                ("to_singular", "bar"),
            ],
        )?;
        Ok(())
    }

    #[test]
    fn test_register_path_helpers() -> Result<(), Error> {
        assert_helpers(
            "/hello/bar/foo",
            vec![
                ("file_name", "foo"),
                ("parent", "/hello/bar"),
                ("extension", ""),
                ("canonicalize", ""),
            ],
        )?;
        assert_helpers(
            "foo",
            vec![("file_name", "foo"), ("parent", ""), ("extension", "")],
        )?;
        assert_helpers(
            "bar/foo",
            vec![("file_name", "foo"), ("parent", "bar"), ("extension", "")],
        )?;
        assert_helpers(
            "bar/foo.txt",
            vec![
                ("file_name", "foo.txt"),
                ("parent", "bar"),
                ("extension", "txt"),
            ],
        )?;
        assert_helpers(
            "./foo",
            vec![
                ("file_name", "foo"),
                ("parent", "."),
                ("extension", ""),
                ("canonicalize", ""),
            ],
        )?;
        assert_helpers(
            "/hello/bar/../foo",
            vec![
                ("file_name", "foo"),
                ("parent", "/hello/bar/.."),
                ("extension", ""),
                ("canonicalize", ""),
            ],
        )?;
        Ok(())
    }

    #[test]
    fn test_register_env_helpers() -> Result<(), Error> {
        let key = "KEY";
        std::env::set_var(key, "VALUE");

        assert_helpers(key, vec![("env_var", "VALUE")])?;
        assert_helpers("A_DO_NOT_EXIST_ENVVAR", vec![("env_var", "")])?;
        Ok(())
    }
}
