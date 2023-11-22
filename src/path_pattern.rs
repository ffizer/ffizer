use crate::error::*;
use globset::{Glob, GlobMatcher};
use serde_plain::derive_deserialize_from_fromstr;
use std::{path::Path, str::FromStr};

#[derive(Debug, Clone)]
pub struct PathPattern {
    pub raw: String,
    pub matcher: GlobMatcher,
}

impl FromStr for PathPattern {
    type Err = crate::Error;
    fn from_str(value: &str) -> Result<Self> {
        let value = value.trim();
        let g = Glob::new(value).map_err(|source| Error::ParsePathPattern {
            value: value.to_owned(),
            source,
        })?;
        let matcher = g.compile_matcher();
        Ok(PathPattern {
            raw: value.to_owned(),
            matcher,
        })
    }
}

derive_deserialize_from_fromstr!(PathPattern, "valid path matcher");

impl PartialEq for PathPattern {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl PathPattern {
    pub fn is_match<P: AsRef<Path>>(&self, value: P) -> bool {
        self.matcher.is_match(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pattern_with_glob() {
        let p = PathPattern::from_str("**/foo.bar").unwrap();
        assert_eq!("**/foo.bar".to_owned(), p.raw);
        assert_eq!(false, p.is_match("hello"));
        assert_eq!(true, p.is_match("hello/f/foo.bar"));
    }

    #[test]
    fn test_pattern_are_trimmed() {
        let actual = PathPattern::from_str(" **/foo.bar").unwrap();
        let expected = PathPattern::from_str("**/foo.bar").unwrap();
        assert_eq!(&expected.raw, &actual.raw);
    }
}
