use crate::error::*;
use crate::git;
use crate::source_uri::SourceUri;
use clap::Args;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use tracing::warn;

#[derive(Args, Debug, Default, Clone, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(deny_unknown_fields, default)]
pub struct SourceLoc {
    /// uri / path of the template
    #[arg(short = 's', long = "source")]
    pub uri: SourceUri,

    /// git revision of the template
    #[arg(long = "rev", default_value = "master")]
    pub rev: Option<String>,

    /// path of the folder under the source uri to use for template
    #[arg(long = "source-subfolder", value_name = "FOLDER")]
    pub subfolder: Option<PathBuf>,
}

impl SourceLoc {
    pub fn find_remote_cache_folder() -> Result<PathBuf> {
        let app_name = env!("CARGO_PKG_NAME");
        let project_dirs = directories::ProjectDirs::from("", app_name, app_name)
            .ok_or(crate::Error::ApplicationPathNotFound {})?;
        let cache_base = project_dirs.cache_dir();
        Ok(cache_base.join("git"))
    }

    pub fn as_local_path(&self) -> Result<PathBuf> {
        let mut path = match self.uri.host {
            None => self
                .uri
                .path
                .canonicalize()
                .map_err(|source| Error::CanonicalizePath {
                    path: self.uri.path.clone(),
                    source,
                })?,
            Some(_) => self.remote_as_local()?,
        };
        if let Some(f) = &self.subfolder {
            path = path.join(f.clone());
        }
        Ok(path)
    }

    // the remote_as_local ignore subfolder
    fn remote_as_local(&self) -> Result<PathBuf> {
        let cache_uri = Self::find_remote_cache_folder()?
            .join(self.uri.host.as_deref().unwrap_or("no_host"))
            .join(&self.uri.path)
            .join(self.rev.as_deref().unwrap_or("_default_"));
        Ok(cache_uri)
    }
    pub fn download(&self, offline: bool) -> Result<PathBuf> {
        if !offline && self.uri.host.is_some() {
            let remote_path = self.remote_as_local()?;
            if let Err(v) = git::retrieve(&remote_path, &self.uri.raw, &self.rev) {
                warn!(
                    src = ?self,
                    path = ?remote_path,
                    error = ?v,
                    "failed to download"
                );
                if remote_path.exists() {
                    fs::remove_dir_all(&remote_path).map_err(|source| Error::RemoveFolder {
                        path: remote_path,
                        source,
                    })?;
                }
                return Err(v);
            }
        }
        let path = self.as_local_path()?;
        if !path.try_exists()? {
            Err(crate::Error::LocalPathNotFound {
                path,
                uri: self.uri.raw.clone(),
                subfolder: self.subfolder.clone(),
            })
        } else {
            Ok(path)
        }
    }
}

impl fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}{})",
            self.uri.raw,
            self.rev
                .as_ref()
                .map(|s| format!("rev: '{}' ", s))
                .unwrap_or_else(|| "".to_string()),
            self.subfolder
                .as_ref()
                .map(|s| format!("subfolder: '{}'", s.to_string_lossy()))
                .unwrap_or_else(|| "".to_string())
        )
    }
}
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use pretty_assertions::assert_eq;
//     use crate::source_uri::SourceUri;
//     use std::str::FromStr;

//     #[test]
//     fn as_local_path_on_git() -> Result<()> {
//         let sut = SourceLoc {
//             uri: SourceUri::from_str("git@github.com:ffizer/ffizer.git")?,
//             rev: "master".to_owned(),
//             subfolder: None,
//         };
//         assert_eq!(true, &sut.as_local_path().unwrap().ends_with("/com.github.ffizer/git/github.com/ffizer/ffizer/master"));
//         Ok(())
//     }
// }
