use std::path::Path;
use std::process::ExitStatus;
use std::{fs, io, process};

use tracing::{debug, error};

use super::{GitError, git_cmd};

#[derive(Debug, thiserror::Error)]
pub enum GitCliError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("Fail to execute command `{0}` returning status {1}")]
    CommandError(String, ExitStatus),
}

#[tracing::instrument]
pub(super) fn retrieve(dst: &Path, url: &str, rev: &Option<String>) -> Result<(), GitError> {
    if dst.join(".git").exists() {
        debug!("Repository already exists, update it");
        git_cmd(dst, &["reset", "--hard" /*, &format!("origin/{rev}")*/])?;
        git_cmd(dst, &["pull", "--force", "--prune"])?;
    } else {
        debug!("Repository does not exists, create it");
        fs::create_dir_all(dst).map_err(|source| GitError::CreateFolder {
            path: dst.to_path_buf(),
            source,
        })?;
        git_cmd(dst, &["clone", "-q", url, dst.to_str().unwrap_or_default()])?;
        if let Some(rev) = rev {
            git_cmd(dst, &["checkout", rev])?;
        }
    }

    Ok(())
}

pub fn find_cmd_tool(kind: &str) -> Result<String, GitError> {
    let tool = config_get_string(&format!("{}.tool", kind))?;
    let result = config_get_string(&format!("{}tool.{}.cmd", kind, tool))?;
    Ok(result)
}

fn config_get_string(value: &str) -> Result<String, GitCliError> {
    let output = process::Command::new("git")
        .args(["config", value])
        .output()?;
    let status = output.status;
    if status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(result)
    } else {
        let msg = format!("git config {value}");
        Err(GitCliError::CommandError(msg, status))
    }
}
