use crate::transform_values::TransformsValues;
use crate::Result;
use globset::{Glob, GlobMatcher};
use serde_plain::derive_deserialize_from_str;
use snafu::ResultExt;
use std::cmp::PartialEq;
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
        let g = Glob::new(value).context(crate::ParsePathPattern {
            value: value.to_owned(),
        })?;
        Ok(PathPattern {
            raw: value.to_owned(),
            matcher: g.compile_matcher(),
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

impl TransformsValues for PathPattern {
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let v = PathPattern::from_str(&render(&self.raw))?;
        Ok(v)
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
