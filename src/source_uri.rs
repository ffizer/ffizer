use failure::format_err;
use failure::Error;
use regex::Regex;
use std::path::PathBuf;
use std::str::FromStr;
use serde_plain::derive_deserialize_from_str;

// create my own URI because didn't found acceptable solution
// - http = "0.1.13" failed to parse "git@github.com:davidB/ffizer.git"
// - uriparse = "0.3.3" require rust nightly
// - uri_parser = "0.2.0" use explicit lifetime for URI, too hard for intergration with CmdOpt
#[derive(Debug, Clone, PartialEq)]
pub struct SourceUri {
    pub raw: String,
    pub path: PathBuf,
    pub host: Option<String>,
}

derive_deserialize_from_str!(SourceUri, "source uri");

impl FromStr for SourceUri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_re = Regex::new(
            r"^(https?|ssh)://(?P<host>[[:alnum:]\._-]+)(:\d+)?/(?P<path>[[:alnum:]\._\-/]+).git$",
        ).unwrap();
        let git_re =
            Regex::new(r"^git@(?P<host>[[:alnum:]\._-]+):(?P<path>[[:alnum:]\._\-/]+).git$")
                .unwrap();
        git_re
            .captures(s)
            .or_else(|| url_re.captures(s))
            .map(|caps| SourceUri {
                raw: s.to_owned(),
                path: PathBuf::from(caps["path"].to_owned()),
                host: Some(caps["host"].to_owned()),
            }).or_else(|| {
                Some(SourceUri {
                    raw: s.to_owned(),
                    path: PathBuf::from(s.to_owned()),
                    host: None,
                })
            }).ok_or_else(|| format_err!("failed to parse source uri"))
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
    fn test_source_uri_from_str() {
        assert_source_uri_from_str("/foo/bar", "/foo/bar", None);
        assert_source_uri_from_str(
            "git@github.com:davidB/ffizer.git",
            "davidB/ffizer",
            Some("github.com"),
        );
        // assert_source_uri_from_str(
        //     "git@github.com:davidB/ffizer",
        //     "davidB/ffizer",
        //     Some("github.com"),
        // );
        assert_source_uri_from_str(
            "https://github.com/davidB/ffizer.git",
            "davidB/ffizer",
            Some("github.com"),
        );
        // assert_source_uri_from_str(
        //     "https://github.com/davidB/ffizer",
        //     "davidB/ffizer",
        //     Some("github.com"),
        // );
    }
}
