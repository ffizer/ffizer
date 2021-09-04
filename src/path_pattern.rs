use crate::error::*;
use globset::{Glob, GlobMatcher};
use serde_plain::derive_deserialize_from_fromstr;
use std::str::FromStr;

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
    pub fn is_match(&self, value: &str) -> bool {
        self.matcher.is_match(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test_pattern_with_glob() {
        let p = PathPattern::from_str("**/foo.bar").unwrap();
        assert_that!(p.raw).is_equal_to("**/foo.bar".to_owned());
        assert_that!(p.is_match("hello")).is_false();
        assert_that!(p.is_match("hello/f/foo.bar")).is_true();
    }

    #[test]
    fn test_pattern_are_trimmed() {
        let actual = PathPattern::from_str(" **/foo.bar").unwrap();
        let expected = PathPattern::from_str("**/foo.bar").unwrap();
        assert_that!(&actual.raw).is_equal_to(&expected.raw);
    }
}
