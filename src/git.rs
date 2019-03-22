use failure::Error;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{Cred, CredentialType, FetchOptions, Repository};
use std::path::Path;

/// clone a repository at a rev to a directory
// TODO id the directory is already present then fetch and rebase (if not in offline mode)
pub fn retrieve<P, U, R>(dst: P, url: U, rev: R) -> Result<(), Error>
where
    P: AsRef<Path>,
    R: AsRef<str>,
    U: AsRef<str>,
{
    let mut fo = make_fetch_options()?;
    let dst = dst.as_ref();
    if dst.exists() {
        checkout(dst, &rev)?;
        pull(dst, rev, &mut fo)?;
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
        clone(&dst, url, "master", fo)?;
        checkout(&dst, rev)?;
    }
    Ok(())
}

/// a best attempt effort is made to authenticate
/// requests when required to support private
/// git repositories
fn make_fetch_options<'a>() -> Result<FetchOptions<'a>, Error> {
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default()?;
    let mut ch = CredentialHandler::new(git_config);
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
    std::fs::create_dir_all(&dst.as_ref())?;
    RepoBuilder::new()
        .branch(rev.as_ref())
        .fetch_options(fo)
        .clone(url.as_ref(), dst.as_ref())?;
    Ok(())
}

//FIXME doesn't work like "git pull"
fn pull<'a, P, R>(dst: P, rev: R, fo: &mut FetchOptions<'a>) -> Result<(), Error>
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

fn checkout<P, R>(dst: P, rev: R) -> Result<(), Error>
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

struct CredentialHandler {
    usernames: Vec<String>,
    ssh_agent_attempts_count: usize,
    username_attempts_count: usize,
    cred_helper_bad: Option<bool>,
    cfg: git2::Config,
}

// implemention based on code & comment from cargo
// https://github.com/rust-lang/cargo/blob/master/src/cargo/sources/git/utils.rs#L415-L628
// License APACHE
// but adapted to not used wrapper over function like withXxx(FnMut), a more OO approach

impl CredentialHandler {
    fn new(cfg: git2::Config) -> Self {
        let mut usernames = Vec::new();
        usernames.push("".to_string()); //default
        usernames.push("git".to_string());
        if let Ok(s) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
            usernames.push(s);
        }

        // let mut cred_helper = git2::CredentialHelper::new(url);
        // cred_helper.config(cfg);
        // if let Some(ref s) = cred_helper.username {
        //     usernames.push(s.clone());
        // }

        CredentialHandler {
            usernames,
            ssh_agent_attempts_count: 0,
            username_attempts_count: 0,
            cred_helper_bad: None,
            cfg,
        }
    }

    /// Prepare the authentication callbacks for cloning a git repository.
    ///
    /// The main purpose of this function is to construct the "authentication
    /// callback" which is used to clone a repository. This callback will attempt to
    /// find the right authentication on the system (without user input) and will
    /// guide libgit2 in doing so.
    ///
    /// The callback is provided `allowed` types of credentials, and we try to do as
    /// much as possible based on that:
    ///
    /// - Prioritize SSH keys from the local ssh agent as they're likely the most
    ///   reliable. The username here is prioritized from the credential
    ///   callback, then from whatever is configured in git itself, and finally
    ///   we fall back to the generic user of `git`.
    ///
    /// - If a username/password is allowed, then we fallback to git2-rs's
    ///   implementation of the credential helper. This is what is configured
    ///   with `credential.helper` in git, and is the interface for the macOS
    ///   keychain, for example.
    ///
    /// - After the above two have failed, we just kinda grapple attempting to
    ///   return *something*.
    ///
    /// If any form of authentication fails, libgit2 will repeatedly ask us for
    /// credentials until we give it a reason to not do so. To ensure we don't
    /// just sit here looping forever we keep track of authentications we've
    /// attempted and we don't try the same ones again.
    pub fn try_next_credential(
        &mut self,
        url: &str,
        username: Option<&str>,
        allowed: CredentialType,
    ) -> Result<Cred, git2::Error> {
        // libgit2's "USERNAME" authentication actually means that it's just
        // asking us for a username to keep going. This is currently only really
        // used for SSH authentication and isn't really an authentication type.
        // The logic currently looks like:
        //
        //      let user = ...;
        //      if (user.is_null())
        //          user = callback(USERNAME, null, ...);
        //
        //      callback(SSH_KEY, user, ...)
        //
        // So if we're being called here then we know that (a) we're using ssh
        // authentication and (b) no username was specified in the URL that
        // we're trying to clone. We need to guess an appropriate username here,
        // but that may involve a few attempts. Unfortunately we can't switch
        // usernames during one authentication session with libgit2, so to
        // handle this we bail out of this authentication session after setting
        // the flag `ssh_username_requested`, and then we handle this below.
        if allowed.contains(CredentialType::USERNAME) {
            // debug_assert!(username.is_none());
            let idx = self.username_attempts_count;
            self.username_attempts_count += 1;
            return match self.usernames.get(idx).map(|s| &s[..]) {
                Some("") if username.is_none() => {
                    Err(git2::Error::from_str("gonna try usernames later"))
                }
                Some("") => Cred::username(&username.unwrap_or("")),
                Some(s) => Cred::username(&s),
                _ => Err(git2::Error::from_str("no more username to try")),
            };
        }

        // An "SSH_KEY" authentication indicates that we need some sort of SSH
        // authentication. This can currently either come from the ssh-agent
        // process or from a raw in-memory SSH key. Cargo only supports using
        // ssh-agent currently.
        //
        // If we get called with this then the only way that should be possible
        // is if a username is specified in the URL itself (e.g., `username` is
        // Some), hence the unwrap() here. We try custom usernames down below.
        if allowed.contains(CredentialType::SSH_KEY) {
            // If ssh-agent authentication fails, libgit2 will keep
            // calling this callback asking for other authentication
            // methods to try. Make sure we only try ssh-agent once,
            // to avoid looping forever.
            let idx = self.ssh_agent_attempts_count;
            self.ssh_agent_attempts_count += 1;
            return match self.usernames.get(idx).map(|s| &s[..]) {
                Some("") => Cred::ssh_key_from_agent(&username.unwrap_or("")),
                Some(s) => Cred::ssh_key_from_agent(&s),
                _ => Err(git2::Error::from_str("no more username to try")),
            };
        }

        // Sometimes libgit2 will ask for a username/password in plaintext. This
        // is where Cargo would have an interactive prompt if we supported it,
        // but we currently don't! Right now the only way we support fetching a
        // plaintext password is through the `credential.helper` support, so
        // fetch that here.
        //
        // If ssh-agent authentication fails, libgit2 will keep calling this
        // callback asking for other authentication methods to try. Check
        // cred_helper_bad to make sure we only try the git credentail helper
        // once, to avoid looping forever.
        if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) && self.cred_helper_bad.is_none() {
            let r = Cred::credential_helper(&self.cfg, url, username);
            self.cred_helper_bad = Some(r.is_err());
            if r.is_err() {
                match Self::ui_ask_user_password(username.unwrap_or("")) {
                    Ok((user, password)) => return Cred::userpass_plaintext(&user, &password),
                    Err(_) => (), //FIXME give a feeback instead of ignore
                }
            }
            return r;
        }

        // I'm... not sure what the DEFAULT kind of authentication is, but seems
        // easy to support?
        if allowed.contains(CredentialType::DEFAULT) {
            return Cred::default();
        }

        // TODO add interactive (user + password)

        // Stop trying
        Err(git2::Error::from_str("no valid authentication available"))
    }

    fn ui_ask_user_password(username: &str) -> Result<(String, String), Error> {
        use dialoguer::Input;
        use dialoguer::PasswordInput;

        let user: String = Input::new()
            .default(username.to_owned())
            .with_prompt("username")
            .interact()?;
        let password: String = PasswordInput::new()
            .with_prompt("password (hidden)")
            .interact()?;
        Ok((user.to_owned(), password.to_owned()))
    }
}
