use std::cmp::Ordering;
use std::error::Error;
use std::fs;
use std::fs::FileType;
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
    Presence { expect: bool, actual: bool },
    // Length {
    //     expect: usize,
    //     actual: usize,
    // },
    Kind { expect: FileType, actual: FileType },
    StringContent { expect: String, actual: String },
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
) -> Result<Vec<EntryDiff>, Box<dyn Error>> {
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
            // asserting(&format!("test content of {:?} vs {:?}", a, b))
            //     .that(&read_to_vec(a.path())?)
            //     .is_equal_to(&read_to_vec(b.path())?);
            let actual_content = fs::read_to_string(actual_entry.path())?.replace("\r\n", "\n");
            let expect_content = fs::read_to_string(expect_entry.path())?.replace("\r\n", "\n");
            if actual_content != expect_content {
                add_diff(
                    &expect_rpath,
                    Difference::StringContent {
                        actual: actual_content,
                        expect: expect_content,
                    },
                );
                actual_index += 1;
                expect_index += 1;
                continue;
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
