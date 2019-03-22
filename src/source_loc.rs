use crate::git;
use crate::source_uri::SourceUri;
use crate::transform_values::TransformsValues;
use failure::format_err;
use failure::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Default, Clone, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(deny_unknown_fields, default)]
pub struct SourceLoc {
    /// uri / path of the template
    #[structopt(short = "s", long = "source")]
    pub uri: SourceUri,

    /// git revision of the template
    #[structopt(long = "rev", default_value = "master")]
    pub rev: String,

    /// path of the folder under the source uri to use for template
    #[structopt(long = "source-subfolder", parse(from_os_str))]
    pub subfolder: Option<PathBuf>,
}

impl SourceLoc {
    pub fn as_local_path(&self) -> Result<PathBuf, Error> {
        let mut path = match self.uri.host {
            None => self.uri.path.clone(),
            Some(_) => self.remote_as_local()?,
        };
        if let Some(f) = &self.subfolder {
            path = path.join(f.clone());
        }
        Ok(path)
    }

    fn remote_as_local(&self) -> Result<PathBuf, Error> {
        let app_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "".into());
        let project_dirs = directories::ProjectDirs::from("com", "github", &app_name)
            .ok_or_else(|| format_err!("Home directory not found"))?;
        let cache_base = project_dirs.cache_dir();
        let cache_uri = cache_base
            .join("git")
            .join(
                &self
                    .uri
                    .host
                    .clone()
                    .unwrap_or_else(|| "no_host".to_owned()),
            )
            .join(&self.uri.path)
            .join(&self.rev);
        Ok(cache_uri)
    }

    pub fn download(&self, offline: bool) -> Result<PathBuf, Error> {
        let path = self.as_local_path()?;
        if !offline && self.uri.host.is_some() {
            if let Err(v) = git::retrieve(&path, &self.uri.raw, &self.rev) {
                if path.exists() {
                    fs::remove_dir_all(path)?;
                }
                return Err(v);
            }
        }
        if !path.exists() {
            Err(format_err!(
                "local path not found {} for {}{}",
                path.to_str().unwrap_or("(empty)"),
                &self.uri.raw,
                self.subfolder
                    .clone()
                    .and_then(|s| s.to_str().map(|v| format!(" and subfolder {}", v)))
                    .unwrap_or_else(|| "".to_owned()) //path.to_str().unwrap_or("??")
            ))
        } else {
            Ok(path)
        }
    }
}

impl TransformsValues for SourceLoc {
    /// transforms default_value & ignore
    fn transforms_values<F>(&self, render: &F) -> Result<SourceLoc, Error>
    where
        F: Fn(&str) -> String,
    {
        let uri = self.uri.transforms_values(render)?;
        let rev = render(&self.rev);
        let subfolder = self
            .subfolder
            .clone()
            .and_then(|f| f.transforms_values(render).ok());
        Ok(SourceLoc {
            uri,
            rev,
            subfolder,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use spectral::prelude::*;
//     use crate::source_uri::SourceUri;
//     use std::str::FromStr;

//     #[test]
//     fn as_local_path_on_git() -> Result<(), Error> {
//         let sut = SourceLoc {
//             uri: SourceUri::from_str("git@github.com:ffizer/ffizer.git")?,
//             rev: "master".to_owned(),
//             subfolder: None,
//         };
//         assert_that!(&sut.as_local_path().unwrap()).ends_with("/com.github.ffizer/git/github.com/ffizer/ffizer/master");
//         Ok(())
//     }
// }
