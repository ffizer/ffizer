use failure::Error;
use handlebars::{handlebars_helper, Handlebars};

fn env_var_fct<T: AsRef<str>>(key: T) -> String {
    match std::env::var(key.as_ref()) {
        Ok(s) => s,
        Err(e) => {
            //TODO better error handler
            //use slog::warn;
            //warn!(ctx.logger, "helper: http_get"; "url" => format!("{:?}", url), "err" => format!("{:?}", e))
            eprintln!(
                "helper: env_var failed for key '{:?}' with error '{:?}'",
                key.as_ref(),
                e
            );
            "".to_owned()
        }
    }
}

pub fn register(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars_helper!(env_var: |v: str| env_var_fct(&v));
    handlebars.register_helper("env_var", Box::new(env_var));
    Ok(())
}
