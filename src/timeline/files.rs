use crate::timeline::FFIZER_DATASTORE_DIRNAME;
use crate::Result;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};

const FILES_FILENAME: &str = "files.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FileInfo {
    pub key: String,
    pub hash: Cid,
}

impl FileInfo {
    pub fn with_key(self, key: &String) -> Self {
        Self {
            key: key.to_owned(),
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TrackedFiles {
    pub files: HashMap<String, HashMap<String, FileInfo>>,
}

pub(crate) fn save_infos_for_source<T>(infos: T, target_folder: &Path, source: String) -> Result<()>
where
    T: IntoIterator<Item = FileInfo>,
{
    let mut tracked = load_tracked(target_folder)?;
    let mut map: HashMap<String, FileInfo> = HashMap::new();
    for info in infos {
        map.insert(info.key.clone(), info);
    }
    tracked.files.insert(source, map);
    save_tracked(&tracked, target_folder)?;
    Ok(())
}

pub(crate) fn get_infos_for_source(
    target_folder: &Path,
    source: String,
) -> Result<HashMap<String, FileInfo>> {
    let tracked = load_tracked(target_folder)?;
    let infos = tracked
        .files
        .get(&source)
        .cloned()
        .unwrap_or_else(HashMap::default);
    Ok(infos)
}

fn save_tracked(tracked: &TrackedFiles, target_folder: &Path) -> Result<()> {
    serde_json::to_writer(
        std::fs::File::create(
            target_folder
                .join(FFIZER_DATASTORE_DIRNAME)
                .join(FILES_FILENAME),
        )?,
        tracked,
    )?;
    Ok(())
}

fn load_tracked(target_folder: &Path) -> Result<TrackedFiles> {
    let path = target_folder
        .join(FFIZER_DATASTORE_DIRNAME)
        .join(FILES_FILENAME);
    if path.exists() {
        Ok(serde_json::from_reader(std::fs::File::open(path)?)?)
    } else {
        Ok(TrackedFiles::default())
    }
}

pub(crate) fn make_file_info(base_folder: &Path, relative_path: &Path) -> Result<FileInfo> {
    let h = Code::Sha2_256.digest(&fs::read(base_folder.join(relative_path))?);
    let info = FileInfo {
        key: relative_path.to_string_lossy().to_string(),
        hash: Cid::new_v1(Code::Sha2_256.into(), h),
    };
    Ok(info)
}
