use crate::Result;
use crate::timeline::FFIZER_DATASTORE_DIRNAME;
use crate::tools::dir_diff_list::walk_dir;
use crate::PathPattern;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};

const FILES_FILENAME: &str = "files.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub key: String,
    pub hash: Cid,
}


pub(crate) fn save_file_infos(target_folder: &Path) -> Result<()> {
    let infos = make_file_infos(target_folder)?;
    serde_json::to_writer(
        std::fs::File::create(target_folder.join(FFIZER_DATASTORE_DIRNAME).join(FILES_FILENAME))?,
        &infos,
    )?;
    Ok(())
}

pub(crate) fn load_file_infos(target_folder: &Path) -> Result<BTreeMap<String, FileInfo>> {
    let path = target_folder.join(FFIZER_DATASTORE_DIRNAME).join(FILES_FILENAME);
    if path.exists() {
        Ok(serde_json::from_reader(std::fs::File::open(
            path,
        )?)?)
    } else {
        Ok(BTreeMap::default())
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


pub(crate) fn make_file_infos(folder: &Path) -> Result<BTreeMap<String, FileInfo>> {
    let entries = walk_dir(folder, &[PathPattern::from_str(".ffizer")?])?;

    let mut infos = BTreeMap::new();
    for entry in entries.into_iter() {
        if entry.metadata()?.is_file() {
            let relative_path = entry.path().strip_prefix(folder)?;
            let file_info = make_file_info(folder, relative_path)?;
            infos.insert(file_info.key.clone(), file_info);
        }
    }
    Ok(infos)
}
