extern crate failure;
extern crate spectral;
extern crate walkdir;

use self::spectral::prelude::*;
use self::walkdir::{DirEntry, WalkDir};
use failure::Error;
use std::cmp::Ordering;
use std::fs;
// use std::fs::File;
// use std::io::prelude::*;
use std::path::Path;

/// Are the contents of two directories same?
pub fn is_same<A: AsRef<Path>, B: AsRef<Path>>(a_base: A, b_base: B) -> Result<bool, Error> {
    let mut a_walker = walk_dir(a_base);
    let mut b_walker = walk_dir(b_base);

    for (a, b) in (&mut a_walker).zip(&mut b_walker) {
        let a = a?;
        let b = b?;

        asserting(&format!("test depth of {:?} vs {:?}", a, b))
            .that(&a.depth())
            .is_equal_to(&b.depth());
        asserting(&format!("test file_type of {:?} vs {:?}", a, b))
            .that(&a.file_type())
            .is_equal_to(&b.file_type());
        asserting(&format!("test file_name of {:?} vs {:?}", a, b))
            .that(&a.file_name())
            .is_equal_to(&b.file_name());
        if a.file_type().is_file() {
            // asserting(&format!("test content of {:?} vs {:?}", a, b))
            //     .that(&read_to_vec(a.path())?)
            //     .is_equal_to(&read_to_vec(b.path())?);
            asserting(&format!("test content of {:?} vs {:?}", a, b))
                .that(&fs::read_to_string(a.path())?)
                .is_equal_to(&fs::read_to_string(b.path())?);
        }
    }

    Ok(!a_walker.next().is_none() || !b_walker.next().is_none())
}

fn walk_dir<P: AsRef<Path>>(path: P) -> std::iter::Skip<walkdir::IntoIter> {
    WalkDir::new(path)
        .sort_by(compare_by_file_name)
        .into_iter()
        .skip(1)
}

fn compare_by_file_name(a: &DirEntry, b: &DirEntry) -> Ordering {
    a.file_name().cmp(&b.file_name())
}

// fn read_to_vec<P: AsRef<Path>>(file: P) -> Result<Vec<u8>, Error> {
//     let mut data = Vec::new();
//     let mut file = File::open(file.as_ref())?;

//     file.read_to_end(&mut data)?;

//     Ok(data)
// }
