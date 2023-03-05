use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::{io, process};

use tracing::{error, info};

use crate::error::Error;

#[cfg(feature = "git2")]
pub(self) mod git2;

pub(self) mod cli;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[cfg(feature = "git2")]
    #[error(transparent)]
    Git2Error(#[from] ::git2::Error),

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("create folder {path:?}")]
    CreateFolder {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    GitCliError(#[from] self::cli::GitCliError),

    #[error("No usable git")]
    GitNotFound,
}

fn has_git_cli() -> bool {
    current_dir()
        .map_err(GitError::from)
        .and_then(|path| git_cmd(&path, &["version"]))
        .map(|status| status.success())
        .unwrap_or_default()
}

#[tracing::instrument(fields(dst = ?dst.as_ref(), url = url.as_ref(), rev = rev.as_ref()))]
pub fn retrieve<P, U, R>(dst: P, url: U, rev: R) -> Result<(), Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
    U: AsRef<str>,
{
    let dst = dst.as_ref();
    let url = url.as_ref();
    let rev = rev.as_ref();

    #[cfg(feature = "git2")]
    match self::git2::retrieve(dst, url, rev) {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            error!("Oops, fail with git2: {e:?}");
        }
    }

    // Fallback to cli
    let result = if has_git_cli() {
        cli::retrieve(dst, url, rev)
    } else {
        Err(GitError::GitNotFound)
    };

    result.map_err(|source| Error::GitRetrieve {
        dst: dst.to_path_buf(),
        url: url.to_owned(),
        rev: rev.to_owned(),
        source,
        msg: Box::new("Fail to retrieve repository".to_string()),
    })
}

pub fn find_cmd_tool(kind: &str) -> Result<String, GitError> {
    #[cfg(feature = "git2")]
    match self::git2::find_cmd_tool(kind) {
        Ok(s) => {
            return Ok(s);
        }
        Err(e) => {
            error!("Oops, fail with git2: {e:?}");
        }
    }

    // Fallback to cli
    if has_git_cli() {
        cli::find_cmd_tool(kind)
    } else {
        Err(GitError::GitNotFound)
    }
}

fn git_cmd(current_dir: &Path, args: &[&str]) -> Result<ExitStatus, GitError> {
    info!(
        "Running command `git {}` in {current_dir:?}",
        args.join(" ")
    );
    let status = process::Command::new("git")
        .args(args)
        .current_dir(current_dir)
        .status()?;
    Ok(status)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use pretty_assertions::assert_eq;
    use run_script::ScriptOptions;
    use tempfile::tempdir;
    use tracing::warn;

    #[test]
    #[ignore = "Only works on my laptop"]
    fn should_get_merge_cmd() {
        let result = find_cmd_tool("merge").unwrap();
        assert_eq!(result, "code --wait $MERGED");
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn retrieve_should_update_existing_template() {
        let _ = tracing_subscriber::fmt::try_init();

        if !has_git_cli() {
            eprintln!("skip the test because `git` is not installed");
            return;
        }

        let tmp_dir = tempdir().unwrap();

        let src_path = tmp_dir.path().join("src");
        let dst_path = tmp_dir.path().join("dst");
        let options = ScriptOptions::new();
        let args = vec![];

        template_v1(&src_path, &dst_path, &args, &options);
        assert_eq!(
            fs::read_to_string(dst_path.join("foo.txt")).unwrap(),
            "v1: Lorem ipsum\n"
        );

        template_v2(&src_path, &dst_path, &args, &options);
        assert_eq!(
            fs::read_to_string(dst_path.join("foo.txt")).unwrap(),
            "v2: Hello\n"
        );

        template_v3(&src_path, &dst_path, &args, &options);
        assert_eq!(
            fs::read_to_string(dst_path.join("foo.txt")).unwrap(),
            "v3: Hourra\n"
        );
    }

    #[tracing::instrument]
    fn template_v1(src_path: &Path, dst_path: &Path, args: &Vec<String>, options: &ScriptOptions) {
        let (code, output, error) = run_script::run(
            &format!(
                r#"
                    mkdir -p {src_path:?}
                    cd {src_path:?}
                    git init -b master
                    git config user.email "test@example.com"
                    git config user.name "Test Name"
                    echo "v1: Lorem ipsum" > foo.txt
                    git add foo.txt
                    git commit -m "add foo.txt"
                    "#
            ),
            args,
            options,
        )
        .unwrap();
        if code != 0 {
            warn!(%output, %error);
        }
        assert_eq!(code, 0, "setup template v1");
        retrieve(dst_path, src_path.to_str().unwrap(), "master").unwrap();
    }

    #[tracing::instrument]
    fn template_v2(src_path: &Path, dst_path: &Path, args: &Vec<String>, options: &ScriptOptions) {
        let (code, output, error) = run_script::run(
            &format!(
                r#"
                    cd {src_path:?}
                    echo "v2: Hello" > foo.txt
                    git add foo.txt
                    git commit -m "add foo.txt"
                    "#,
            ),
            args,
            options,
        )
        .unwrap();
        if code != 0 {
            warn!(%output, %error);
        }
        assert_eq!(code, 0, "setup template v2");

        retrieve(dst_path, src_path.to_str().unwrap(), "master").unwrap();
    }

    #[tracing::instrument]
    fn template_v3(src_path: &Path, dst_path: &Path, args: &Vec<String>, options: &ScriptOptions) {
        let (code, output, error) = run_script::run(
            &format!(
                r#"
                    cd {src_path:?}
                    echo "v3: Hourra" > foo.txt
                    git add foo.txt
                    git commit -m "add foo.txt"
                    "#,
            ),
            args,
            options,
        )
        .unwrap();
        if code != 0 {
            warn!(%output, %error);
        }
        assert_eq!(code, 0, "setup template v3");

        retrieve(dst_path, src_path.to_str().unwrap(), "master").unwrap();
    }
}
