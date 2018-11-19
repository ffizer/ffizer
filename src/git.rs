use failure::Error;
use git2::build::RepoBuilder;
use git2::FetchOptions;
use std::path::Path;

/// clone a repository at a rev to a directory
// TODO id the directory is already present then fetch and rebase (if not in offline mode)
pub fn retreive<P, U, R>(dst: P, url: U, rev: R) -> Result<(), Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
    U: AsRef<str>,
{
    let fo = make_fetch_options()?;
    if dst.as_ref().exists() {
        //pull(dst, rev, &mut fo)
        //until pull is fixed and work as expected
        std::fs::remove_dir_all(&dst)?;
        clone(dst, url, rev, fo)
    } else {
        clone(dst, url, rev, fo)
    }
}

/// a best attempt effort is made to authenticate
/// requests when required to support private
/// git repositories
fn make_fetch_options<'a>() -> Result<FetchOptions<'a>, Error> {
    let mut cb = git2::RemoteCallbacks::new();
    let mut tried_sshkey = false;
    cb.credentials(move |url, username, cred_type| {
        if cred_type.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            let cfg = git2::Config::open_default()?;
            return git2::Cred::credential_helper(&cfg, url, username);
        }
        if cred_type.contains(git2::CredentialType::SSH_KEY) && !tried_sshkey {
            // If ssh-agent authentication fails, libgit2 will keep
            // calling this callback asking for other authentication
            // methods to try. Make sure we only try ssh-agent once,
            // to avoid looping forever.
            tried_sshkey = true;
            let username = username.unwrap();
            return git2::Cred::ssh_key_from_agent(&username);
        }
        Err(git2::Error::from_str("no authentication available"))
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb)
        .download_tags(git2::AutotagOption::All);
    Ok(fo)
}

fn clone<'a, P, U, R>(dst: P, url: U, rev: R, fo: FetchOptions<'a>) -> Result<(), Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
    U: AsRef<str>,
{
    RepoBuilder::new()
        .branch(rev.as_ref())
        .fetch_options(fo)
        .clone(url.as_ref(), dst.as_ref())?;
    //.chain_err(|| format!("failed to clone repo {}@{}", &url, revision.clone()))?;
    Ok(())
}

// FIXME doesn't work like "git pull"
// fn pull<'a, P, R>(dst: P, rev: R, fo: &mut FetchOptions<'a>) -> Result<(), Error>
// where
//     P: AsRef<Path>,
//     R: AsRef<str>,
// {
//     let repository = Repository::discover(dst.as_ref())?;
//     let revref = format!("origin/{}", rev.as_ref());
//     assert!(Reference::is_valid_name(&revref));
//     let mut remote = repository.find_remote("origin")?;
//     remote.fetch(&[&revref], Some(fo), None);
//     // remote.update_tips(None, true, AutotagOption::Unspecified, None)?;
//     // remote.disconnect();
//     let mut co = CheckoutBuilder::new();
//     co.force().remove_ignored(true);
//     let reference = repository.find_reference(&revref)?;
//     repository.set_head(&revref)?;
//     repository.checkout_head(Some(&mut co))?;
//     Ok(())
// }
