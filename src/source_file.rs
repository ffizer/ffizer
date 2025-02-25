use crate::ChildPath;
use crate::files;
use std::cmp::{Ord, Ordering};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceFileMetadata {
    Dir,
    // TODO Symlink { target: String },
    RawFile,
    RenderableFile { extension: &'static str },
}

impl SourceFileMetadata {
    fn kind_idx(&self) -> usize {
        match self {
            // Self::Symlink { .. } => 0,
            Self::Dir => 1,
            Self::RenderableFile { .. } => 2,
            Self::RawFile => 3,
        }
    }
}

impl Ord for SourceFileMetadata {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind_idx().cmp(&other.kind_idx())
    }
}

impl PartialOrd for SourceFileMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
    pub childpath: ChildPath,
    pub layer_order: usize,
    pub metadata: SourceFileMetadata,
}

impl Ord for SourceFile {
    fn cmp(&self, other: &Self) -> Ordering {
        let l = self.layer_order.cmp(&other.layer_order);
        if l == Ordering::Equal {
            self.metadata.cmp(&other.metadata)
        } else {
            l
        }
    }
}

impl PartialOrd for SourceFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl SourceFile {
    pub fn childpath(&self) -> &ChildPath {
        &self.childpath
    }
}

// // TODO add test
// // TODO add priority for generated file name / folder name
// // TODO document priority (via test ?)
// fn cmp_path_for_plan(a: &Action, b: &Action) -> Ordering {
//     let cmp_dst = a.dst_path.relative.cmp(&b.dst_path.relative);
//     if cmp_dst != Ordering::Equal {
//         cmp_dst
//     } else if a
//         .src_path
//         .relative
//         .to_str()
//         .map(|s| s.contains("{{"))
//         .unwrap_or(false)
//     {
//         Ordering::Greater
//     } else if is_ffizer_handlebars(&a.src_path.relative) {
//         Ordering::Less
//     } else if is_ffizer_handlebars(&b.src_path.relative) {
//         Ordering::Greater
//     } else {
//         a.src_path.relative.cmp(&b.src_path.relative)
//     }
// }

impl From<(ChildPath, usize)> for SourceFile {
    //TODO manage symlink
    fn from((childpath, layer_order): (ChildPath, usize)) -> Self {
        let path = PathBuf::from(&childpath);
        //let metadata = std::fs::symlink_metadata(path)?;
        if path.is_dir() {
            SourceFile {
                childpath,
                layer_order,
                metadata: SourceFileMetadata::Dir,
            }
        } else if files::is_ffizer_handlebars(&path) {
            SourceFile {
                childpath,
                layer_order,
                metadata: SourceFileMetadata::RenderableFile {
                    extension: files::FILEEXT_HANDLEBARS,
                },
            }
        } else {
            SourceFile {
                childpath,
                layer_order,
                metadata: SourceFileMetadata::RawFile,
            }
        }
    }
}

pub(crate) fn optimize_sourcefiles(sources: &mut Vec<SourceFile>) {
    sources.sort();
    // take until not Renderable
    // Because apply Dir, Symlink, RawFile erase previous change
    if let Some(pos) = sources
        .iter()
        .position(|x| !matches!(x.metadata, SourceFileMetadata::RenderableFile { .. }))
    {
        sources.truncate(pos + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // pub use crate::cli_opt::*;
    use pretty_assertions::assert_eq;
    // use tempfile::TempDir;

    #[test]
    fn test_cmp_sourcefile() {
        let mut input = vec![
            SourceFile::from((ChildPath::new("./tests/test_1/template", "file_2.txt"), 0)),
            SourceFile::from((
                ChildPath::new("./tests/test_1/template", "file_2.txt.ffizer.hbs"),
                0,
            )),
        ];
        let expected = vec![
            SourceFile::from((
                ChildPath::new("./tests/test_1/template", "file_2.txt.ffizer.hbs"),
                0,
            )),
            SourceFile::from((ChildPath::new("./tests/test_1/template", "file_2.txt"), 0)),
        ];
        optimize_sourcefiles(&mut input);
        assert_eq!(&expected, &input);
    }
}
