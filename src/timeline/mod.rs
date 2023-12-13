pub(crate) mod persist;
mod files;
mod options;

pub(crate) use persist::*;
pub(crate) use files::*;
pub(crate) use options::*;

use crate::cfg::{ImportCfg, TemplateCfg};
use crate::{Result, SourceLoc, SourceUri};
use crate::Error;
use std::path::Path;
use std::str::FromStr;
use pathdiff::diff_paths;


pub(crate) const FFIZER_DATASTORE_DIRNAME: &str = ".ffizer";
const VERSION_FILENAME: &str = "version.txt";

pub(crate) fn make_template(sources: Vec<SourceLoc>) -> TemplateCfg {
    // not ready for standalone command, only used as part of reapply for now
    // does not use variables for now, it relies on the variables reimport during apply
    let imports: Vec<ImportCfg> = sources
        .into_iter()
        .map(|source| ImportCfg {
            uri: source.uri.raw,
            rev: source.rev,
            subfolder: source.subfolder.map(|x| x.to_string_lossy().to_string()),
        })
        .collect();

    TemplateCfg {
        imports,
        use_template_dir: true,
        ..Default::default() // No need to set default variables as they will be read from the target folder, this needs to be improved to make a true template
    }
}

pub(crate) fn make_template_from_folder(
    source_folder: &Path,
    target_folder: &Path,
) -> Result<SourceLoc> {
    let options = load_options(source_folder)?;

    let sources: Vec<SourceLoc> = options.sources.into_iter().map(|saved_src| -> Result<SourceLoc> {
        let src = SourceLoc::try_from(saved_src)?;
        if src.uri.host.is_none() && !src.uri.path.is_absolute() { // uri is local and relative, we need to make it relative to target_folder
            let local_path = src.uri
                .path
                .canonicalize()
                .map_err(|err| Error::CanonicalizePath {
                    path: src.uri.path,
                    source: err,
                })?;
            let relative_path = diff_paths(local_path, target_folder.canonicalize()?).unwrap();
            Ok(SourceLoc {
                uri: SourceUri::from_str(&relative_path.to_string_lossy())?,
                ..src
            })
        } else {
            Ok(src)
        }
    }).collect::<Result<Vec<SourceLoc>>>()?;

    let template_cfg = make_template(sources);

    serde_yaml::to_writer(
        std::fs::File::create(target_folder.join(".ffizer.yaml"))?,
        &template_cfg,
    )?;
    std::fs::create_dir(target_folder.join("template"))?;

    Ok(SourceLoc {
        uri: SourceUri::from_str(&target_folder.to_string_lossy())?,
        rev: None,
        subfolder: None,
    })
}

fn key_from_loc(source: &SourceLoc) -> (String, String, String) {
    let uri = &source.uri;
    (
        uri.host.clone().unwrap_or_default(),
        uri.path.to_string_lossy().into(),
        source
            .subfolder
            .clone()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    use std::path::PathBuf;
    use tempfile::TempDir;

    #[fixture]
    fn tmp_dir() -> TempDir {
        TempDir::new().expect("create a temp dir")
    }

    mod test_make_template {
        use super::*;
        use similar_asserts::assert_eq;

        #[rstest]
        fn empty() {
            let expected = TemplateCfg {
                use_template_dir: true,
                ..Default::default()
            };

            assert_eq!(expected, make_template(vec![]))
        }

        #[rstest]
        fn single_source() -> Result<()> {
            let sources = vec![
                SourceLoc {
                    uri: SourceUri::from_str("/path/to/foo")?,
                    rev: None,
                    subfolder: None    
                }
            ];

            let expected = TemplateCfg {
                imports: vec![ImportCfg {
                    uri: "path/to/foo".to_string(),
                    rev: None,
                    subfolder: None,
                }],
                use_template_dir: true,
                ..Default::default()
            };
            assert_eq!(expected, make_template(sources));
            Ok(())
        }

        #[rstest]
        fn multi_source() -> Result<()> {
            let sources = vec![
                SourceLoc {
                    uri: SourceUri::from_str("path/to/foo")?,
                    rev: None,
                    subfolder: None,
                },
                SourceLoc {
                    uri: SourceUri::from_str("http://blabla.truc/a/path")?,
                    rev: Some("master".into()),
                    subfolder: Some(PathBuf::from_str("some_subfolder").unwrap()),
                },
            ];

            let expected = TemplateCfg {
                imports: vec![
                    ImportCfg {
                        uri: "path/to/foo".to_string(),
                        rev: None,
                        subfolder: None,
                    },
                    ImportCfg {
                        uri: "http://blabla.truc/a/path".into(),
                        rev: Some("master".into()),
                        subfolder: Some("some_subfolder".into()),
                    },
                ],
                use_template_dir: true,
                ..Default::default()
            };
            assert_eq!(expected, make_template(sources));
            Ok(())
        }
    }
}
