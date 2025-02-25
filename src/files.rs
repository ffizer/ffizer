use crate::Result;
use crate::path_pattern::PathPattern;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

pub const FILEEXT_HANDLEBARS: &str = ".ffizer.hbs";
pub const FILEEXT_RAW: &str = ".ffizer.raw";

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, Ord, PartialOrd)]
pub struct ChildPath {
    pub relative: PathBuf,
    pub base: PathBuf,
}

impl ChildPath {
    pub fn new<P1, P2>(base: P1, relative: P2) -> ChildPath
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        ChildPath {
            relative: PathBuf::from(relative.as_ref()),
            base: PathBuf::from(base.as_ref()),
        }
    }
}

impl From<&ChildPath> for PathBuf {
    fn from(v: &ChildPath) -> Self {
        v.base.join(&v.relative)
    }
}

pub fn is_ffizer_handlebars(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|str| str.contains(FILEEXT_HANDLEBARS))
        .unwrap_or(false)
}

pub fn remove_special_suffix(path: &Path) -> Result<PathBuf> {
    match path.file_name().and_then(|s| s.to_str()) {
        Option::None => Ok(path.to_path_buf()),
        Some(v) => {
            let file_name = remove_special_suffix_on_filename(v);
            Ok(path.with_file_name(file_name))
        }
    }
}

fn remove_special_suffix_on_filename(v: &str) -> String {
    if v.contains(FILEEXT_RAW) {
        v.replacen(FILEEXT_RAW, "", 1)
    } else if v.contains(FILEEXT_HANDLEBARS) {
        v.replacen(FILEEXT_HANDLEBARS, "", 1)
    } else {
        v.to_owned()
    }
}

pub fn add_suffix<P>(path: P, suffix: &str) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    Ok(PathBuf::from(format!(
        "{}{}",
        path.as_ref().to_string_lossy(),
        suffix
    )))
}

pub fn find_childpaths<P>(base: P, ignores: &[PathPattern]) -> Vec<ChildPath>
where
    P: AsRef<Path>,
{
    let base = base.as_ref();
    WalkDir::new(base)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            e.path()
                .strip_prefix(base)
                .expect("scanned child path to be under base")
                .to_str()
                .map(|s| !ignores.iter().any(|f| f.is_match(s)))
                // .map(|s| true)
                .unwrap_or(true)
        })
        .filter_map(|e| e.ok())
        .map(|entry| ChildPath {
            base: base.to_path_buf(),
            relative: entry
                .into_path()
                .strip_prefix(base)
                .expect("scanned child path to be under base")
                .to_path_buf(),
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[test]
    fn test_is_ffizer_handlebars() {
        assert_eq!(false, is_ffizer_handlebars(&PathBuf::from("foo.hbs")));
        assert_eq!(
            false,
            is_ffizer_handlebars(&PathBuf::from("foo.ffizer.hbs/bar"))
        );
        assert_eq!(
            false,
            is_ffizer_handlebars(&PathBuf::from("foo_ffizer.hbs"))
        );
        assert_eq!(false, is_ffizer_handlebars(&PathBuf::from("fooffizer.hbs")));

        assert_eq!(true, is_ffizer_handlebars(&PathBuf::from("foo.ffizer.hbs")));
        assert_eq!(
            true,
            is_ffizer_handlebars(&PathBuf::from("bar/foo.ffizer.hbs"))
        );
    }

    #[rstest]
    #[case("foo.hbs", "foo.hbs")]
    #[case("foo.json.ffizer.hbs", "foo.json")]
    #[case("foo.ffizer.hbs.json", "foo.json")]
    #[case("foo.json.ffizer.raw", "foo.json")]
    #[case("foo.ffizer.raw.json", "foo.json")]
    #[case("foo.json.ffizer.raw", "foo.json")]
    #[case("foo.ffizer.raw.ffizer.raw.json", "foo.ffizer.raw.json")]
    #[case("foo.ffizer.raw.ffizer.hbs.json", "foo.ffizer.hbs.json")]
    #[case("foo.json.ffizer.raw.ffizer.hbs", "foo.json.ffizer.hbs")]
    fn test_remove_special_suffix_on_filename(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            expected.to_string(),
            remove_special_suffix_on_filename(input)
        );
    }

    #[test]
    fn test_add_suffix() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            PathBuf::from("foo.ext1"),
            add_suffix(PathBuf::from("foo.ext1"), "")?
        );
        assert_eq!(
            PathBuf::from("foo.ext1.REMOTE"),
            add_suffix(PathBuf::from("foo.ext1"), ".REMOTE")?
        );
        Ok(())
    }
}
