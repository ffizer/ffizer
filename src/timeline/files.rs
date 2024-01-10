use crate::timeline::FFIZER_DATASTORE_DIRNAME;
use crate::Result;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};

const FILES_FILENAME: &str = "files.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FileHash {
    hash: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FileMeta {
    pub key: String,
    pub remote: FileHash,
    pub accepted: FileHash,
}

pub(crate) fn path_as_key(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TrackedFiles {
    pub files: HashMap<String, HashMap<String, FileMeta>>,
}

pub(crate) fn save_metas_for_source(
    infos: impl IntoIterator<Item = FileMeta>,
    target_folder: &Path,
    source: &str,
) -> Result<()> {
    let mut tracked = load_tracked(target_folder)?;
    let mut map: HashMap<String, FileMeta> = HashMap::new();
    for info in infos {
        map.insert(info.key.to_owned(), info); // bad, should enforce keys to be identical
    }
    tracked.files.insert(String::from(source), map);
    save_tracked(&tracked, target_folder)?;
    Ok(())
}

pub(crate) fn get_stored_metas_for_source(
    target_folder: &Path,
    source: &str,
) -> Result<HashMap<String, FileMeta>> {
    let infos = load_tracked(target_folder)?
        .files
        .remove(source)
        .unwrap_or_else(HashMap::default);
    Ok(infos)
}

fn save_tracked(tracked: &TrackedFiles, target_folder: &Path) -> Result<()> {
    let file_path = target_folder
        .join(FFIZER_DATASTORE_DIRNAME)
        .join(FILES_FILENAME);
    serde_json::to_writer(std::fs::File::create(file_path)?, tracked)?;
    Ok(())
}

fn load_tracked(target_folder: &Path) -> Result<TrackedFiles> {
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

pub(crate) fn get_hash(path: &Path) -> Result<FileHash> {
    let h = Code::Sha2_256.digest(&fs::read(path)?);
    let filehash = FileHash {
        hash: Cid::new_v1(Code::Sha2_256.into(), h),
    };
    Ok(filehash)
}
