use failure::Error;
use handlebars::handlebars_helper;
use handlebars::*;
use inflector::Inflector;
use reqwest;
use std::path::Path;

pub fn new_hbs() -> Result<Handlebars, Error> {
    let mut handlebars = Handlebars::new();
    setup_handlebars(&mut handlebars)?;
    Ok(handlebars)
}

pub fn setup_handlebars(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars.set_strict_mode(true);
    register_string_helpers(handlebars)?;
    register_http_helpers(handlebars)?;
    register_path_helpers(handlebars)?;
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

fn http_get_fct<T: AsRef<str>>(url: T) -> String {
    match reqwest::get(url.as_ref()).and_then(|mut r| r.text()) {
        Ok(s) => s,
        Err(e) => {
            //TODO better error handler
            //use slog::warn;
            //warn!(ctx.logger, "helper: http_get"; "url" => format!("{:?}", url), "err" => format!("{:?}", e))
            eprintln!(
                "helper: http_get failed for url '{:?}' with error '{:?}'",
                url.as_ref(),
                e
            );
            "".to_owned()
        }
    }
}

fn register_http_helpers(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars_helper!(http_get: |v: str| http_get_fct(&v));
    handlebars.register_helper("http_get", Box::new(http_get));

    handlebars_helper!(gitignore_io: |v: str| http_get_fct(format!("https://www.gitignore.io/api/{}", v)));
    handlebars.register_helper("gitignore_io", Box::new(gitignore_io));
    Ok(())
}

fn register_path_helpers(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars_helper!(parent: |v: str| Path::new(v).parent().and_then(|s| s.to_str()).unwrap_or("".into()));
    handlebars.register_helper("parent", Box::new(parent));

    handlebars_helper!(file_name: |v: str| Path::new(v).file_name().and_then(|s| s.to_str()).unwrap_or("".into()));
    handlebars.register_helper("file_name", Box::new(file_name));

    handlebars_helper!(extension: |v: str| Path::new(v).extension().and_then(|s| s.to_str()).unwrap_or("".into()));
    handlebars.register_helper("extension", Box::new(extension));

    handlebars_helper!(canonicalize: |v: str| {
        Path::new(v).canonicalize().ok().and_then(|s| s.to_str().map(|v| v.to_owned())).unwrap_or("".into())
    });
    handlebars.register_helper("canonicalize", Box::new(canonicalize));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::Variables;
    use super::*;
    use spectral::prelude::*;

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
}
