use crate::error::*;
use git2::build::RepoBuilder;
use git2::{Config, FetchOptions};
use std::path::Path;
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

use super::GitError;

/// clone a repository at a rev to a directory
// TODO if the directory is already present then fetch and rebase (if not in offline mode)
#[tracing::instrument]
pub fn retrieve(dst: &Path, url: &str, rev: &Option<String>) -> Result<(), GitError> {
    let fo = make_fetch_options()?;
    if dst.exists() {
        // HACK until pull is fixed, remove cache if older than ttl (5min) and clone
        if dst
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::now()).ok())
            .map(|d| d > Duration::from_secs(5 * 60))
            .unwrap_or(true)
        {
            std::fs::remove_dir_all(dst)?;
            clone(dst, url, rev, fo)?;
        }
    } else {
        info!("git clone into cached template");
        clone(dst, url, rev, fo)?;
    }
    Ok(())
}

/// a best attempt effort is made to authenticate
/// requests when required to support private
/// git repositories
fn make_fetch_options<'a>() -> Result<FetchOptions<'a>, git2::Error> {
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default()?;
    let mut ch = git2_credentials::CredentialHandler::new(git_config);
    cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

    let mut fo = FetchOptions::new();
    let mut proxy_options = git2::ProxyOptions::new();
    proxy_options.auto();
    fo.proxy_options(proxy_options)
        .remote_callbacks(cb)
        .download_tags(git2::AutotagOption::All)
        .update_fetchhead(true);
    Ok(fo)
}

fn clone(
    dst: &Path,
    url: &str,
    rev: &Option<String>,
    fo: FetchOptions<'_>,
) -> Result<(), GitError> {
    std::fs::create_dir_all(dst).map_err(|source| GitError::CreateFolder {
        path: dst.to_path_buf(),
        source,
    })?;
    let mut builder = RepoBuilder::new();
    if let Some(rev) = rev {
        builder.branch(rev);
    }
    builder
        .fetch_options(fo)
        .clone(url.as_ref(), dst.as_ref())
        .map_err(|err| {
            // remove dst folder on error
            let _ = std::fs::remove_dir_all(dst);
            err
        })?;
    Ok(())
}

/// kind can be "merge" or "diff"
pub fn find_cmd_tool(kind: &str) -> Result<String, git2::Error> {
    let config = Config::open_default()?;
    let tool = config.get_string(&format!("{}.tool", kind))?;
    config.get_string(&format!("{}tool.{}.cmd", kind, tool))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::tempdir;
    use tracing_subscriber::FmtSubscriber;

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn retrieve_should_update_existing_template() {
        let subscriber = FmtSubscriber::builder()
            .with_writer(tracing_subscriber::fmt::writer::TestWriter::default())
            .with_max_level(tracing::Level::WARN)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        if std::process::Command::new("git")
            .arg("version")
            .output()
            .is_err()
        {
            eprintln!("skip the test because `git` is not installed");
            return;
        }

        let tmp_dir = tempdir().unwrap();

        let src_path = tmp_dir.path().join("src");
        let dst_path = tmp_dir.path().join("dst");
        let options = run_script::ScriptOptions::new();
        let args = vec![];

        // template v1
        {
            let span = tracing::span!(tracing::Level::INFO, "template v1");
            let _enter = span.enter();
            let (code, output, error) = run_script::run(
                &format!(
                    r#"
                        mkdir -p {}
                        cd {}
                        git init -b master
                        git config user.email "test@example.com"
                        git config user.name "Test Name"
                        echo "v1: Lorem ipsum" > foo.txt
                        git add foo.txt
                        git commit -m "add foo.txt"
                        "#,
                    src_path.to_str().unwrap(),
                    src_path.to_str().unwrap()
                ),
                &args,
                &options,
            )
            .unwrap();
            if code != 0 {
                warn!(%output, %error);
            }
            assert_eq!(code, 0, "setup template v1");
            retrieve(
                &dst_path,
                src_path.to_str().unwrap(),
                &Some("master".to_string()),
            )
            .unwrap();
            assert_eq!(
                fs::read_to_string(dst_path.join("foo.txt")).unwrap(),
                "v1: Lorem ipsum\n"
            );
        }

        // template v2
        {
            let span = tracing::span!(tracing::Level::INFO, "template v2");
            let _enter = span.enter();

            let (code, output, error) = run_script::run(
                &format!(
                    r#"
                        cd {}
                        echo "v2: Hello" > foo.txt
                        git add foo.txt
                        git commit -m "add foo.txt"
                        "#,
                    src_path.to_str().unwrap()
                ),
                &args,
                &options,
            )
            .unwrap();
            if code != 0 {
                warn!(%output, %error);
            }
            assert_eq!(code, 0, "setup template v2");

            retrieve(&dst_path, src_path.to_str().unwrap(), &None).unwrap();
            assert_eq!(
                fs::read_to_string(dst_path.join("foo.txt")).unwrap(),
                "v2: Hello\n"
            );
        }

        // template v3
        {
            let span = tracing::span!(tracing::Level::INFO, "template v3");
            let _enter = span.enter();

            let (code, output, error) = run_script::run(
                &format!(
                    r#"
                        cd {}
                        echo "v3: Hourra" > foo.txt
                        git add foo.txt
                        git commit -m "add foo.txt"
                        "#,
                    src_path.to_str().unwrap()
                ),
                &args,
                &options,
            )
            .unwrap();
            if code != 0 {
                warn!(%output, %error);
            }
            assert_eq!(code, 0, "setup template v3");

            retrieve(&dst_path, src_path.to_str().unwrap(), &None).unwrap();
            assert_eq!(
                fs::read_to_string(dst_path.join("foo.txt")).unwrap(),
                "v3: Hourra\n"
            );
        }
        fs::remove_dir_all(tmp_dir).expect("remove tmp dir");
    }
}
