use failure::Error;
use handlebars::{handlebars_helper, Handlebars};
use reqwest;

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

pub fn register(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars_helper!(http_get: |v: str| http_get_fct(&v));
    handlebars.register_helper("http_get", Box::new(http_get));

    handlebars_helper!(gitignore_io: |v: str| http_get_fct(format!("https://www.gitignore.io/api/{}", v)));
    handlebars.register_helper("gitignore_io", Box::new(gitignore_io));
    Ok(())
}
