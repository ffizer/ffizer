use crate::path_pattern::PathPattern;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

pub const FILEEXT_HANDLEBARS: &str = ".ffizer.hbs";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChildPath {
    pub relative: PathBuf,
    pub base: PathBuf,
    pub is_symlink: bool,
}

impl<'a> From<&'a ChildPath> for PathBuf {
    fn from(v: &ChildPath) -> Self {
        v.base.join(&v.relative)
    }
}

pub fn is_ffizer_handlebars(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|str| str.ends_with(FILEEXT_HANDLEBARS))
        .unwrap_or(false)
}

pub fn find_childpaths<P>(base: P, ignores: &Vec<PathPattern>) -> Vec<ChildPath>
where
    P: AsRef<Path>,
{
    let base = base.as_ref();
    WalkDir::new(base)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            e.clone()
                .into_path()
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
            is_symlink: entry.path_is_symlink(),
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
    use spectral::prelude::*;

    #[test]
    fn test_is_ffizer_handlebars() {
        assert_that!(is_ffizer_handlebars(&PathBuf::from("foo.hbs"))).is_false();
        assert_that!(is_ffizer_handlebars(&PathBuf::from("foo.ffizer.hbs/bar"))).is_false();
        assert_that!(is_ffizer_handlebars(&PathBuf::from("foo_ffizer.hbs"))).is_false();
        assert_that!(is_ffizer_handlebars(&PathBuf::from("fooffizer.hbs"))).is_false();

        assert_that!(is_ffizer_handlebars(&PathBuf::from("foo.ffizer.hbs"))).is_true();
        assert_that!(is_ffizer_handlebars(&PathBuf::from("bar/foo.ffizer.hbs"))).is_true();
    }
}
