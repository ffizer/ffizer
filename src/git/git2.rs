use crate::error::*;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{Config, FetchOptions, Repository};
use std::path::Path;
use tracing::{debug, info, warn};

use super::GitError;

/// clone a repository at a rev to a directory
// TODO id the directory is already present then fetch and rebase (if not in offline mode)
#[tracing::instrument]
pub fn retrieve(dst: &Path, url: &str, rev: &str) -> Result<(), GitError> {
    let mut fo = make_fetch_options()?;
    if dst.exists() {
        info!("git reset cached template");
        checkout(dst, &rev)?;
        info!("git pull cached template");
        pull(dst, &rev, &mut fo)?;
    //until pull is fixed and work as expected
    // let mut tmp = dst.to_path_buf().clone();
    // tmp.set_extension("part");
    // if tmp.exists() {
    //     std::fs::remove_dir_all(&tmp)?;
    // }
    // clone(&tmp, url, "master", fo)?;
    // checkout(&tmp, rev)?;
    // std::fs::remove_dir_all(&dst)?;
    // std::fs::rename(&tmp, &dst)?;
    } else {
        info!("git clone into cached template");
        clone(dst, &url, "master", fo)?;
        checkout(dst, &rev)?;
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

fn clone(dst: &Path, url: &str, rev: &str, fo: FetchOptions<'_>) -> Result<(), GitError> {
    std::fs::create_dir_all(dst).map_err(|source| GitError::CreateFolder {
        path: dst.to_path_buf(),
        source,
    })?;
    RepoBuilder::new()
        .branch(rev.as_ref())
        .fetch_options(fo)
        .clone(url.as_ref(), dst.as_ref())?;
    Ok(())
}

// from https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
fn pull<P, R>(dst: P, rev: R, fo: &mut FetchOptions) -> Result<(), git2::Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
{
    let repository = Repository::discover(dst.as_ref())?;

    // fetch
    let mut revref = rev.as_ref().to_string();
    //FIXME workaround see https://github.com/rust-lang/git2-rs/issues/819
    if revref.len() == 40
        && revref
            .chars()
            .all(|c| ('0'..='9').contains(&c) || ('a'..='f').contains(&c))
    {
        revref = format!("+{}:{}", rev.as_ref(), rev.as_ref());
    }
    let mut remote = repository.find_remote("origin")?;
    remote.fetch(&[revref], Some(fo), None)?;
    let reference = repository.find_reference("FETCH_HEAD")?;
    let fetch_head_commit = repository.reference_to_annotated_commit(&reference)?;
    do_merge(&repository, rev.as_ref(), fetch_head_commit)?;
    Ok(())
}

// from https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    debug!(msg = ?msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

// from https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        warn!("merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

// from https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;
    debug!(analysis = ?&analysis.0);
    // 2. Do the appopriate merge
    if analysis.0.is_fast_forward() {
        debug!("git merge: doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        debug!("git merge: doing normal merge");
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(repo, &head_commit, &fetch_commit)?;
    } else {
        debug!("git merge: nothing to do");
    }
    Ok(())
}

fn checkout<P, R>(dst: P, rev: R) -> Result<(), git2::Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
{
    let rev = rev.as_ref();
    let repository = Repository::discover(dst.as_ref())?;
    let mut co = CheckoutBuilder::new();
    co.force().remove_ignored(true).remove_untracked(true);
    let treeish = repository.revparse_single(rev)?;
    repository.checkout_tree(&treeish, Some(&mut co))?;
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
            retrieve(&dst_path, src_path.to_str().unwrap(), "master").unwrap();
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

            retrieve(&dst_path, src_path.to_str().unwrap(), "master").unwrap();
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

            retrieve(&dst_path, src_path.to_str().unwrap(), "master").unwrap();
            assert_eq!(
                fs::read_to_string(dst_path.join("foo.txt")).unwrap(),
                "v3: Hourra\n"
            );
        }
        fs::remove_dir_all(tmp_dir).expect("remove tmp dir");
    }
}
