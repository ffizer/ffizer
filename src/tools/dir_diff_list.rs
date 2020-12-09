use crate::error::*;
use std::cmp::Ordering;
use std::fs;
use std::fs::FileType;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntryDiff {
    expect_base_path: PathBuf,
    actual_base_path: PathBuf,
    relative_path: PathBuf,
    difference: Difference,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Difference {
    Presence {
        expect: bool,
        actual: bool,
    },
    // Length {
    //     expect: usize,
    //     actual: usize,
    // },
    Kind {
        expect: FileType,
        actual: FileType,
    },
    StringContent {
        expect: String,
        actual: String,
    },
    BinaryContent {
        expect_md5: String,
        actual_md5: String,
    },
    //Permission
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct SearchOptions {
//     check_content: bool,
//     content_as_string: Vec<String>,
// }

pub fn search_diff<A: AsRef<Path>, B: AsRef<Path>>(
    actual: A,
    expect: B,
    // options: SearchOptions,
) -> Result<Vec<EntryDiff>> {
    let actual = actual.as_ref();
    let expect = expect.as_ref();

    let mut differences = vec![];
    let actual_listing = walk_dir(&actual)?;
    let expect_listing = walk_dir(&expect)?;

    let mut actual_index = 0;
    let mut expect_index = 0;

    let mut add_diff = |rpath: &Path, difference: Difference| {
        differences.push(EntryDiff {
            actual_base_path: actual.to_path_buf(),
            expect_base_path: expect.to_path_buf(),
            relative_path: rpath.to_path_buf(),
            difference,
        });
    };
    loop {
        if actual_index == actual_listing.len() || expect_index == expect_listing.len() {
            break;
        }
        let actual_entry = &actual_listing[actual_index];
        let actual_rpath = actual_entry.path().strip_prefix(actual)?;

        let expect_entry = &expect_listing[expect_index];
        let expect_rpath = expect_entry.path().strip_prefix(expect)?;

        if actual_rpath > expect_rpath {
            add_diff(
                &expect_rpath,
                Difference::Presence {
                    actual: false,
                    expect: true,
                },
            );
            expect_index += 1;
            continue;
        }
        if actual_rpath < expect_rpath {
            add_diff(
                &actual_rpath,
                Difference::Presence {
                    actual: true,
                    expect: false,
                },
            );
            actual_index += 1;
            continue;
        }
        if actual_entry.file_type() != expect_entry.file_type() {
            add_diff(
                &expect_rpath,
                Difference::Kind {
                    actual: actual_entry.file_type(),
                    expect: expect_entry.file_type(),
                },
            );
            actual_index += 1;
            expect_index += 1;
            continue;
        }
        if expect_entry.file_type().is_file() {
            if let Some(diff) = compare_file(
                expect_entry.path().to_path_buf(),
                actual_entry.path().to_path_buf(),
            )? {
                add_diff(&expect_rpath, diff);
            }
        }
        actual_index += 1;
        expect_index += 1;
    }
    if expect_index != expect_listing.len() {
        while expect_index < expect_listing.len() {
            let expect_entry = &expect_listing[expect_index];
            let expect_rpath = expect_entry.path().strip_prefix(expect)?;
            differences.push(EntryDiff {
                actual_base_path: actual.to_path_buf(),
                expect_base_path: expect.to_path_buf(),
                relative_path: expect_rpath.to_path_buf(),
                difference: Difference::Presence {
                    actual: false,
                    expect: true,
                },
            });
            expect_index += 1;
        }
    }
    if actual_index != actual_listing.len() {
        while actual_index < actual_listing.len() {
            let actual_entry = &actual_listing[actual_index];
            let actual_rpath = actual_entry.path().strip_prefix(actual)?;
            differences.push(EntryDiff {
                actual_base_path: actual.to_path_buf(),
                expect_base_path: expect.to_path_buf(),
                relative_path: actual_rpath.to_path_buf(),
                difference: Difference::Presence {
                    actual: true,
                    expect: false,
                },
            });
            actual_index += 1;
        }
    }

    Ok(differences)
}

fn compare_file(expect_path: PathBuf, actual_path: PathBuf) -> Result<Option<Difference>> {
    let expect_md5 = md5::compute(fs::read(&expect_path).map_err(|source| Error::ReadFile {
        path: expect_path.clone(),
        source,
    })?);
    let actual_md5 = md5::compute(fs::read(&actual_path).map_err(|source| Error::ReadFile {
        path: actual_path.clone(),
        source,
    })?);
    if expect_md5 == actual_md5 {
        Ok(None)
    } else {
        match fs::read_to_string(&expect_path) {
            // content is text
            Ok(expect_content) => {
                let expect_content = expect_content.replace("\r\n", "\n");
                let actual_content = fs::read_to_string(&actual_path)
                    .map_err(|source| Error::ReadFile {
                        path: actual_path,
                        source: source,
                    })?
                    .replace("\r\n", "\n");
                if actual_content != expect_content {
                    Ok(Some(Difference::StringContent {
                        actual: actual_content,
                        expect: expect_content,
                    }))
                } else {
                    Ok(None)
                }
            }
            // content is maybe binary
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                Ok(Some(Difference::BinaryContent {
                    actual_md5: format!("{:x}", actual_md5),
                    expect_md5: format!("{:x}", expect_md5),
                }))
            }
            // other error
            Err(source) => Err(Error::ReadFile {
                path: expect_path,
                source: source,
            }),
        }
    }
}

fn walk_dir<P: AsRef<Path>>(path: P) -> Result<Vec<DirEntry>, walkdir::Error> {
    WalkDir::new(path).sort_by(compare).into_iter().collect()
}

fn compare(a: &DirEntry, b: &DirEntry) -> Ordering {
    let d = a.depth().cmp(&b.depth());
    if d == Ordering::Equal {
        a.file_name().cmp(&b.file_name())
    } else {
        d
    }
}
