use globset::{Glob, GlobMatcher};
use std::str::FromStr;
use serde_plain::derive_deserialize_from_str;
use std::cmp::PartialEq;

#[derive(Debug, Clone)]
pub struct PathPattern {
    pub raw: String,
    pub matcher: GlobMatcher,
}

impl FromStr for PathPattern {
    type Err = globset::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let g = Glob::new(value)?;
        Ok(PathPattern{
            raw: value.to_owned(),
            matcher: g.compile_matcher()
        })
    }
}

derive_deserialize_from_str!(PathPattern, "valid path matcher");

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
}
