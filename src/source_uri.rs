use crate::error::*;
use regex::Regex;
use serde_plain::derive_deserialize_from_fromstr;
use std::path::PathBuf;
use std::str::FromStr;

// create my own URI because didn't found acceptable solution
// - http = "0.1.13" failed to parse "git@github.com:ffizer/ffizer.git"
// - uriparse = "0.3.3" require rust nightly
// - uri_parser = "0.2.0" use explicit lifetime for URI, too hard for intergration with CmdOpt
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct SourceUri {
    pub raw: String,
    pub path: PathBuf,
    pub host: Option<String>,
}

derive_deserialize_from_fromstr!(SourceUri, "source uri");

impl FromStr for SourceUri {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        let url_re = Regex::new(
            r"^(https?|ssh)://([[:alnum:]:\._-]+@)?(?P<host>[[:alnum:]\._-]+)(:\d+)?/(?P<path>[[:alnum:]\._\-/]+)$",
        ).map_err(|source| Error::ParseGitUri{value: s.to_owned(), source})?;
        // let url_re2 = Regex::new(
        //     r"^(https?|ssh)://([[:alnum:]:\._-]+@)?(?P<host>[[:alnum:]\._-]+)(:\d+)?/(?P<path>[[:alnum:]\._\-/]+)$",
        // ).map_err(|source| Error::ParseGitUri{value: s.to_owned(), source})?;
        let git_re = Regex::new(r"^git@(?P<host>[[:alnum:]\._-]+):(?P<path>[[:alnum:]\._\-/]+)$")
            .map_err(|source| Error::ParseGitUri {
            value: s.to_owned(),
            source,
        })?;
        // let git_re2 = Regex::new(r"^git@(?P<host>[[:alnum:]\._-]+):(?P<path>[[:alnum:]\._\-/]+)$")
        //     .map_err(|source| Error::ParseGitUri {
        //         value: s.to_owned(),
        //         source,
        //     })?;

        let mut text = s.strip_suffix(".git").unwrap_or(s).to_owned();
        if s.starts_with("gh:") {
            text = s.replacen("gh:", "git@github.com:", 1);
        } else if s.starts_with("gl:") {
            text = s.replacen("gl:", "git@gitlab.com:", 1);
        } else if s.starts_with("bb:") {
            text = s.replacen("bb:", "git@bitbucket.org:", 1);
        }

        git_re
            .captures(&text)
            // .or_else(|| git_re2.captures(&text))
            .or_else(|| url_re.captures(&text))
            // .or_else(|| url_re2.captures(&text))
            .map(|caps| SourceUri {
                raw: text.clone(),
                path: PathBuf::from(caps["path"].to_owned()),
                host: Some(caps["host"].to_owned()),
            })
            .or_else(|| {
                Some(SourceUri {
                    raw: text,
                    path: PathBuf::from(change_local_path_sep(s)),
                    host: None,
                })
            })
            .ok_or_else(|| Error::Unknown("failed to parse source uri".to_owned()))
    }
}

//HACK to support Path -> string -> Path
fn change_local_path_sep(s: &str) -> String {
    if cfg!(windows) {
        // canonicalize on windows return UNC path,
        // that can cause probleme when converted into string then back to Path
        // see https://github.com/rust-lang/rust/issues/42869
        s.replace('/', "\\").replace("\\\\?\\", "")
    } else {
        s.replace('\\', "/")
    }
}

impl Default for SourceUri {
    fn default() -> Self {
        SourceUri {
            raw: ".".to_owned(),
            path: PathBuf::from("."),
            host: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use similar_asserts::assert_eq;

    #[rstest]
    #[case::abs_localpath("/foo/bar", "/foo/bar", "/foo/bar", None)]
    #[case::git_with_git_extension(
        "git@github.com:ffizer/ffizer.git",
        "git@github.com:ffizer/ffizer",
        "ffizer/ffizer",
        Some("github.com")
    )]
    #[case::git_without_git_extension(
        "git@github.com:ffizer/ffizer",
        "git@github.com:ffizer/ffizer",
        "ffizer/ffizer",
        Some("github.com")
    )]
    #[case::https_with_git_extension(
        "https://github.com/ffizer/ffizer.git",
        "https://github.com/ffizer/ffizer",
        "ffizer/ffizer",
        Some("github.com")
    )]
    #[case::https_without_git_extension(
        "https://github.com/ffizer/ffizer",
        "https://github.com/ffizer/ffizer",
        "ffizer/ffizer",
        Some("github.com")
    )]
    #[case::https_with_git_extension_and_username(
        "https://user@github.com/ffizer/ffizer.git",
        "https://user@github.com/ffizer/ffizer",
        "ffizer/ffizer",
        Some("github.com")
    )]
    #[case::abbreviation_gh(
        "gh:ffizer/ffizer",
        "git@github.com:ffizer/ffizer",
        "ffizer/ffizer",
        Some("github.com")
    )]
    #[case::abbreviation_gl(
        "gl:ffizer/ffizer",
        "git@gitlab.com:ffizer/ffizer",
        "ffizer/ffizer",
        Some("gitlab.com")
    )]
    #[case::abbreviation_bitbucket(
        "bb:ffizer/ffizer",
        "git@bitbucket.org:ffizer/ffizer",
        "ffizer/ffizer",
        Some("bitbucket.org")
    )]
    fn assert_source_uri_from_str(
        #[case] input: &str,
        #[case] raw: &str,
        #[case] path: &str,
        #[case] host: Option<&str>,
    ) {
        assert_eq!(
            &SourceUri::from_str(input).unwrap(),
            &SourceUri {
                raw: raw.to_owned(),
                path: PathBuf::from(path.to_owned()),
                host: host.map(|s| s.into()),
            }
        );
    }
}
