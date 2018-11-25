use failure::Error;
use handlebars::handlebars_helper;
use handlebars::*;
use inflector::Inflector;

pub fn new_hbs() -> Result<Handlebars, Error> {
    let mut handlebars = Handlebars::new();
    setup_handlebars(&mut handlebars)?;
    Ok(handlebars)
}

pub fn setup_handlebars(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars.set_strict_mode(true);
    register_string_helpers(handlebars)?;
    Ok(())
}

#[macro_export]
macro_rules! handlebars_register_inflector {
    ($engine:ident, $fct_name:ident) => {
        handlebars_helper!($fct_name: |v: str| v.$fct_name());
        $engine.register_helper(stringify!($fct_name), Box::new($fct_name));
    }
}

fn register_string_helpers(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars_helper!(to_lower_case: |v: str| v.to_lowercase());
    handlebars.register_helper("to_lower_case", Box::new(to_lower_case));

    handlebars_helper!(to_upper_case: |v: str| v.to_uppercase());
    handlebars.register_helper("to_upper_case", Box::new(to_upper_case));

    handlebars_register_inflector!(handlebars, to_camel_case);
    handlebars_register_inflector!(handlebars, to_pascal_case);
    handlebars_register_inflector!(handlebars, to_snake_case);
    handlebars_register_inflector!(handlebars, to_screaming_snake_case);
    handlebars_register_inflector!(handlebars, to_kebab_case);
    handlebars_register_inflector!(handlebars, to_train_case);
    handlebars_register_inflector!(handlebars, to_sentence_case);
    handlebars_register_inflector!(handlebars, to_title_case);
    handlebars_register_inflector!(handlebars, to_class_case);
    handlebars_register_inflector!(handlebars, to_table_case);
    handlebars_register_inflector!(handlebars, to_plural);
    handlebars_register_inflector!(handlebars, to_singular);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::Variables;
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test_register_string_helpers() -> Result<(), Error> {
        let hbs = new_hbs()?;
        let mut vs = Variables::new();
        vs.insert("var".into(), "Hello foo-bars".into());
        let samples = vec![
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
        ];
        for sample in samples {
            let tmpl = format!("{{{{ {} var}}}}", sample.0);
            assert_that!(hbs.render_template(&tmpl, &vs)?)
                .named(sample.0)
                .is_equal_to(sample.1.to_owned());
        }
        Ok(())
    }
}
