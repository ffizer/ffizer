use crate::timeline::FFIZER_DATASTORE_DIRNAME;
use crate::Result;

use std::fs;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};

const FILES_FILENAME: &str = "files.json";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FileHash {
    hash: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FileMeta {
    pub remote: FileHash,
    pub accepted: FileHash,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TrackedFiles {
    files: HashMap<PathBuf, FileMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Source {
    #[non_exhaustive]
    Global,
}

pub(crate) fn save_filemetas_for_source(
    infos: impl IntoIterator<Item = (PathBuf, FileMeta)>,
    target_folder: &Path,
    source: &Source,
) -> Result<()> {
    let mut tracked = TrackedFiles::from_path(target_folder)?;
    let mut new_files = HashMap::new();
    for (key, info) in infos {
        new_files.insert(key, info);
    }
    tracked.files.extend(new_files);
    match source {
        Source::Global => tracked.save(target_folder)?,
    };
    Ok(())
}

pub(crate) fn get_stored_metas_for_source(
    target_folder: &Path,
    source: &Source,
) -> Result<HashMap<PathBuf, FileMeta>> {
    let infos = TrackedFiles::from_path(target_folder)?.files;
    match source {
        Source::Global => Ok(infos),
    }
}

impl TrackedFiles {
    fn save(self, target_folder: &Path) -> Result<()> {
        let file_path = target_folder
            .join(FFIZER_DATASTORE_DIRNAME)
            .join(FILES_FILENAME);
        serde_json::to_writer(std::fs::File::create(file_path)?, &self)?;
        Ok(())
    }

    fn from_path(target_folder: &Path) -> Result<TrackedFiles> {
        let path = target_folder
            .join(FFIZER_DATASTORE_DIRNAME)
            .join(FILES_FILENAME);
        if path.exists() {
            Ok(serde_json::from_reader(std::io::BufReader::new(
                std::fs::File::open(path)?,
            ))?)
        } else {
            Ok(TrackedFiles::default())
        }
    }
}

pub(crate) fn get_hash(path: &Path) -> Result<FileHash> {
    let h = Code::Sha2_256.digest(&fs::read(path)?);
    let hash = Cid::new_v1(Code::Sha2_256.into(), h);
    let filehash = FileHash { hash };
    Ok(filehash)
}
