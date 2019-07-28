use crate::Error;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, Repository};
use git2_credentials;
use snafu::ResultExt;
use std::path::Path;

/// clone a repository at a rev to a directory
// TODO id the directory is already present then fetch and rebase (if not in offline mode)
pub fn retrieve<P, U, R>(dst: P, url: U, rev: R) -> Result<(), Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
    U: AsRef<str>,
{
    let dst = dst.as_ref();
    let mut fo = make_fetch_options().context(crate::GitRetrieve {
        dst: dst.to_path_buf(),
        url: url.as_ref().to_owned(),
        rev: rev.as_ref().to_owned(),
    })?;
    if dst.exists() {
        checkout(dst, &rev).context(crate::GitRetrieve {
            dst: dst.to_path_buf(),
            url: url.as_ref().to_owned(),
            rev: rev.as_ref().to_owned(),
        })?;
        pull(dst, &rev, &mut fo).context(crate::GitRetrieve {
            dst: dst.to_path_buf(),
            url: url.as_ref().to_owned(),
            rev: rev.as_ref().to_owned(),
        })?;
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
        clone(&dst, &url, "master", fo)?;
        checkout(&dst, &rev).context(crate::GitRetrieve {
            dst: dst.to_path_buf(),
            url: url.as_ref().to_owned(),
            rev: rev.as_ref().to_owned(),
        })?;
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
    fo.remote_callbacks(cb)
        .download_tags(git2::AutotagOption::All)
        .update_fetchhead(true);
    Ok(fo)
}

fn clone<P, U, R>(dst: P, url: U, rev: R, fo: FetchOptions<'_>) -> Result<(), Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
    U: AsRef<str>,
{
    std::fs::create_dir_all(&dst.as_ref()).context(crate::CreateFolder {
        path: dst.as_ref().to_path_buf(),
    })?;
    RepoBuilder::new()
        .branch(rev.as_ref())
        .fetch_options(fo)
        .clone(url.as_ref(), dst.as_ref())
        .context(crate::GitRetrieve {
            dst: dst.as_ref().to_path_buf(),
            url: url.as_ref().to_owned(),
            rev: rev.as_ref().to_owned(),
        })?;
    Ok(())
}

//FIXME doesn't work like "git pull"
fn pull<'a, P, R>(dst: P, rev: R, fo: &mut FetchOptions<'a>) -> Result<(), git2::Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
{
    let repository = Repository::discover(dst.as_ref())?;
    let revref = rev.as_ref();
    // assert!(Reference::is_valid_name(&revref));
    let mut remote = repository.find_remote("origin")?;
    remote.fetch(&[revref], Some(fo), None)?;
    repository.set_head("FETCH_HEAD")?;
    // // remote.update_tips(None, true, AutotagOption::Unspecified, None)?;
    // // remote.disconnect();
    // let mut co = CheckoutBuilder::new();
    // co.force().remove_ignored(true);
    // let reference = repository.find_reference(&revref)?;
    // repository.set_head(&revref)?;
    // repository.checkout_head(Some(&mut co))?;
    checkout(dst, rev)?;
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
    co.force().remove_ignored(true);
    let treeish = repository.revparse_single(rev)?;
    repository.checkout_tree(&treeish, Some(&mut co))?;
    Ok(())
}
