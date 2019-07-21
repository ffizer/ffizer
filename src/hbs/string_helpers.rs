use failure::Error;
use handlebars::{handlebars_helper, Handlebars};
use inflector::Inflector;

#[macro_export]
macro_rules! handlebars_register_inflector {
    ($engine:ident, $fct_name:ident) => {
        handlebars_helper!($fct_name: |v: str| v.$fct_name());
        $engine.register_helper(stringify!($fct_name), Box::new($fct_name));
    }
}

pub fn register(handlebars: &mut Handlebars) -> Result<(), Error> {
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
