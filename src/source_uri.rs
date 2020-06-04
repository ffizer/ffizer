use crate::Result;
use regex::Regex;
use serde_plain::derive_deserialize_from_str;
use snafu::ResultExt;
use std::path::PathBuf;
use std::str::FromStr;

// create my own URI because didn't found acceptable solution
// - http = "0.1.13" failed to parse "git@github.com:ffizer/ffizer.git"
// - uriparse = "0.3.3" require rust nightly
// - uri_parser = "0.2.0" use explicit lifetime for URI, too hard for intergration with CmdOpt
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceUri {
    pub raw: String,
    pub path: PathBuf,
    pub host: Option<String>,
}

derive_deserialize_from_str!(SourceUri, "source uri");

impl FromStr for SourceUri {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        let url_re = Regex::new(
            r"^(https?|ssh)://([[:alnum:]:\._-]+@)?(?P<host>[[:alnum:]\._-]+)(:\d+)?/(?P<path>[[:alnum:]\._\-/]+).git$",
        ).context(crate::ParseGitUri{value: s.to_owned()})?;
        let url_re2 = Regex::new(
            r"^(https?|ssh)://([[:alnum:]:\._-]+@)?(?P<host>[[:alnum:]\._-]+)(:\d+)?/(?P<path>[[:alnum:]\._\-/]+)$",
        ).context(crate::ParseGitUri{value: s.to_owned()})?;
        let git_re =
            Regex::new(r"^git@(?P<host>[[:alnum:]\._-]+):(?P<path>[[:alnum:]\._\-/]+).git$")
                .context(crate::ParseGitUri {
                    value: s.to_owned(),
                })?;
        let git_re2 = Regex::new(r"^git@(?P<host>[[:alnum:]\._-]+):(?P<path>[[:alnum:]\._\-/]+)$")
            .context(crate::ParseGitUri {
                value: s.to_owned(),
            })?;
        git_re
            .captures(s)
            .or_else(|| git_re2.captures(s))
            .or_else(|| url_re.captures(s))
            .or_else(|| url_re2.captures(s))
            .map(|caps| SourceUri {
                raw: s.to_owned(),
                path: PathBuf::from(caps["path"].to_owned()),
                host: Some(caps["host"].to_owned()),
            })
            .or_else(|| {
                Some(SourceUri {
                    raw: s.to_owned(),
                    path: PathBuf::from(s.to_owned()),
                    host: None,
                })
            })
            .ok_or(crate::Error::Any {
                msg: "failed to parse source uri".to_owned(),
            })
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
    use spectral::prelude::*;

    fn assert_source_uri_from_str(s: &str, path: &str, host: Option<&str>) {
        assert_that!(&SourceUri::from_str(s).unwrap()).is_equal_to(&SourceUri {
            raw: s.to_owned(),
            path: PathBuf::from(path.to_owned()),
            host: host.map(|s| s.into()),
        });
    }

    #[test]
    fn test_source_uri_from_str_abs_localpath() {
        assert_source_uri_from_str("/foo/bar", "/foo/bar", None);
    }

    #[test]
    fn test_source_uri_from_str_git_with_git_extension() {
        assert_source_uri_from_str(
            "git@github.com:ffizer/ffizer.git",
            "ffizer/ffizer",
            Some("github.com"),
        );
    }

    #[test]
    fn test_source_uri_from_str_git_without_git_extension() {
        assert_source_uri_from_str(
            "git@github.com:ffizer/ffizer",
            "ffizer/ffizer",
            Some("github.com"),
        );
    }

    #[test]
    fn test_source_uri_from_str_http_with_git_extension() {
        assert_source_uri_from_str(
            "https://github.com/ffizer/ffizer.git",
            "ffizer/ffizer",
            Some("github.com"),
        );
        assert_source_uri_from_str(
            "http://github.com/ffizer/ffizer.git",
            "ffizer/ffizer",
            Some("github.com"),
        );
    }

    #[test]
    fn test_source_uri_from_str_http_without_git_extension() {
        assert_source_uri_from_str(
            "https://github.com/ffizer/ffizer",
            "ffizer/ffizer",
            Some("github.com"),
        );
    }

    #[test]
    fn test_source_uri_from_str_http_with_git_extension_and_username() {
        assert_source_uri_from_str(
            "https://user@github.com/ffizer/ffizer.git",
            "ffizer/ffizer",
            Some("github.com"),
        );
        assert_source_uri_from_str(
            "http://user@github.com/ffizer/ffizer.git",
            "ffizer/ffizer",
            Some("github.com"),
        );
    }

    #[test]
    fn test_source_uri_from_str_http_with_git_extension_and_username_and_password() {
        assert_source_uri_from_str(
            "https://user:pass@github.com/ffizer/ffizer.git",
            "ffizer/ffizer",
            Some("github.com"),
        );
        assert_source_uri_from_str(
            "http://user:pass@github.com/ffizer/ffizer.git",
            "ffizer/ffizer",
            Some("github.com"),
        );
    }
}
