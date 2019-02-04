use crate::path_pattern::PathPattern;
use failure::format_err;
use failure::Error;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

pub const FILEEXT_HANDLEBARS: &str = ".ffizer.hbs";
pub const FILEEXT_RAW: &str = ".ffizer.raw";

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

pub fn remove_special_suffix(path: &Path) -> Result<PathBuf, Error> {
    match path.file_name().and_then(|s| s.to_str()) {
        None => Ok(path.to_path_buf()),
        Some(v) => {
            let file_name = if v.ends_with(FILEEXT_HANDLEBARS) {
                v.get(..v.len() - FILEEXT_HANDLEBARS.len()).ok_or_else(|| {
                    format_err!("failed to remove {} from file_name", FILEEXT_HANDLEBARS)
                })?
            } else if v.ends_with(FILEEXT_RAW) {
                v.get(..v.len() - FILEEXT_RAW.len())
                    .ok_or_else(|| format_err!("failed to remove {} from file_name", FILEEXT_RAW))?
            } else {
                v
            };
            Ok(path.with_file_name(file_name))
        }
    }
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
