use crate::timeline::FFIZER_DATASTORE_DIRNAME;
use crate::Result;
use crate::SourceLoc;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};

use super::key_from_loc;

const FILES_FILENAME: &str = "files.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub key: String,
    pub hash: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TrackedFiles {
    pub files: HashMap<String, HashMap<String, FileInfo>>,
}

pub(crate) fn save_source_infos<T>(infos: T, target_folder: &Path, source: String) -> Result<()>
where
    T: IntoIterator<Item = FileInfo>,
{
    let mut tracked = load_tracked(target_folder)?;
    let mut map: HashMap<String, FileInfo> = HashMap::new();
    for info in infos {
        map.insert(info.key.clone(), info.clone());
    }
    tracked.files.insert(source, map);
    save_tracked(&tracked, target_folder)?;
    Ok(())
}

pub(crate) fn get_source_infos(
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

/* pub(crate) fn make_file_infos(folder: &Path) -> Result<Vec<FileInfo>> {
    let entries = walk_dir(folder, &[PathPattern::from_str(FFIZER_DATASTORE_DIRNAME)?])?;

    let mut infos = Vec::new();
    for entry in entries.into_iter() {
        if entry.metadata()?.is_file() {
            infos.push(make_file_info(folder, entry.path().strip_prefix(folder)?)?);
        }
    }
    Ok(infos)
} */

/* pub(crate) fn save_folder(target_folder: &Path, source: &SourceLoc) -> Result<()> {
    let infos = make_file_infos(target_folder)?;
    save_infos(&infos, target_folder)?;
    Ok(())
}

pub(crate) fn load_folder(target_folder: &Path) -> Result<TrackedFiles> {
    let path = target_folder.join(FFIZER_DATASTORE_DIRNAME).join(FILES_FILENAME);
    if path.exists() {
        Ok(serde_json::from_reader(std::fs::File::open(
            path,
        )?)?)
    } else {
        Ok(BTreeMap::default())
    }

}
 */
