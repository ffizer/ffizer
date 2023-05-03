use crate::error::*;
use run_script::{IoOptions, ScriptOptions};

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(deny_unknown_fields, default)]
pub struct Script {
    pub message: Option<String>,
    pub cmd: Option<String>,
    pub default_confirm_answer: bool,
}

impl Script {
    pub(crate) fn run(&self) -> Result<()> {
        if let Some(cmd) = &self.cmd {
            let mut options = ScriptOptions::new();
            options.input_redirection = IoOptions::Inherit;
            options.output_redirection = IoOptions::Inherit;
            let args = vec![];
            run_script::run(cmd, &args, &options).map_err(|source| Error::ScriptError {
                script: cmd.clone(),
                source,
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use run_script::IoOptions;
    use similar_asserts::assert_eq;

    use super::*;

    #[test]
    fn should_redirect_io() {
        let cmd = r#"
            echo "What's your name?"
            read username;
            echo "👋 Hello ${username}"
            echo "🚨 Plaf!"  1>&2"#;

        let mut options = ScriptOptions::new();
        options.input_redirection = IoOptions::Pipe;
        options.output_redirection = IoOptions::Pipe;
        let args = vec![];

        // let mut input = stdin().lock();
        let mut child = run_script::spawn(cmd, &args, &options).unwrap();

        let mut stdin = child.stdin.take().unwrap();
        std::thread::spawn(move || {
            stdin.write_all("ffizer".as_bytes()).unwrap();
        });

        let out = child.wait_with_output().unwrap();

        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(stdout, "What's your name?\n👋 Hello ffizer\n");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(stderr, "🚨 Plaf!\n");
    }
}
