mod files;
mod options;

pub(crate) use files::*;
pub(crate) use options::*;
use path_absolutize::Absolutize;

use crate::cfg::{ImportCfg, TemplateCfg};
use crate::Error;
use crate::{Result, SourceLoc, SourceUri};
use pathdiff::diff_paths;
use std::path::Path;
use std::str::FromStr;

pub(crate) const FFIZER_DATASTORE_DIRNAME: &str = ".ffizer";
const VERSION_FILENAME: &str = "version.txt";

pub(crate) fn make_relative(loc: SourceLoc, from: &Path, to: &Path) -> Result<SourceLoc> {
    // It's a no op if loc is absolute or an url
    if loc.uri.host.is_none() && loc.uri.path.is_relative() {
        let source_path = loc.uri.path.absolutize_from(from)?;

        let new_base = to.absolutize()?;

        let relative_path = diff_paths(&source_path, &new_base).ok_or(Error::DiffPathError {
            path: source_path.into(),
            base: new_base.into(),
        })?;
        Ok(SourceLoc {
            uri: SourceUri::from_str(&relative_path.to_string_lossy())?,
            ..loc
        })
    } else {
        Ok(loc)
    }
}

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

    let sources: Vec<SourceLoc> = options
        .sources
        .into_iter()
        .map(SourceLoc::try_from)
        .map(|result| {
            result.and_then(|loc| make_relative(loc, &std::env::current_dir()?, target_folder))
        })
        .collect::<Result<Vec<SourceLoc>>>()?;

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
            let sources = vec![SourceLoc {
                uri: SourceUri::from_str("path/to/foo")?,
                rev: None,
                subfolder: None,
            }];

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
