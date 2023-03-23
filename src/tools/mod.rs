pub mod dir_diff_list;

use crate::cli_opt::{ApplyOpts, CliOpts, Command, TestSamplesOpts};
use crate::path_pattern::PathPattern;
use crate::{error::*, SourceLoc};
use clap::Parser;
use dir_diff_list::Difference;
use dir_diff_list::EntryDiff;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};
use tracing::info;

pub fn test_samples(cfg: &TestSamplesOpts) -> Result<()> {
    let template_base_path = &cfg.src.download(cfg.offline)?;
    if !check_samples(template_base_path, &cfg.src)? {
        Err(crate::Error::TestSamplesFailed {})
    } else {
        Ok(())
    }
}

fn check_samples<A: AsRef<Path>>(template_path: A, template_loc: &SourceLoc) -> Result<bool> {
    let mut is_success = true;
    let tmp_dir = tempdir()?;
    let samples_folder = template_path
        .as_ref()
        .join(crate::cfg::TEMPLATE_SAMPLES_DIRNAME);
    let samples = Sample::find_from_folder(template_loc, &samples_folder, &tmp_dir)?;
    info!(nb_samples_detected = samples.len(), ?samples_folder);
    for sample in samples {
        info!(sample = ?sample.name, args = ?sample.args, "checking...");
        let run = SampleRun::run(&sample)?;
        is_success = is_success && run.is_success();
        show_differences(&sample.name, &run.diffs)?;
    }
    Ok(is_success)
}

//TODO move to ui module to be customizable (in future)
pub fn show_differences(name: &str, entries: &[EntryDiff]) -> Result<()> {
    for entry in entries {
        println!("--------------------------------------------------------------");
        match &entry.difference {
            Difference::Presence { expect, actual } => {
                if *expect && !*actual {
                    println!(
                        "missing file in the actual: {}",
                        entry.relative_path.to_string_lossy()
                    );
                } else {
                    println!(
                        "unexpected file in the actual: {}",
                        entry.relative_path.to_string_lossy()
                    );
                }
            }
            Difference::Kind { expect, actual } => {
                println!(
                    "difference kind of entry on: {}, expected: {:?}, actual: {:?}",
                    entry.relative_path.to_string_lossy(),
                    expect,
                    actual
                );
            }
            Difference::StringContent { expect, actual } => {
                println!(
                    "difference detected on: {}\n",
                    entry.relative_path.to_string_lossy()
                );
                crate::ui::show_difference_text(&expect, &actual, true);
            }
            Difference::BinaryContent {
                expect_md5,
                actual_md5,
            } => {
                println!(
                    "difference detected on: {} (detected as binary file)\n",
                    entry.relative_path.to_string_lossy()
                );
                println!("expected md5: {}", expect_md5);
                println!("actual md5: {}", actual_md5);
            }
        }
    }
    println!("--------------------------------------------------------------");
    println!(
        "number of differences in sample '{}': {}",
        name,
        entries.len()
    );
    println!("--------------------------------------------------------------");
    Ok(())
}

#[derive(Debug, Clone)]
struct Sample {
    pub name: String,
    pub args: ApplyOpts,
    pub expected: PathBuf,
    pub existing: PathBuf,
    pub ignores: Vec<PathPattern>,
}

impl Sample {
    // scan folder to find sample to test (xxx.args, xxx.expected, xxx.existing)
    fn find_from_folder<B: AsRef<Path>>(
        template_loc: &SourceLoc,
        samples_folder: B,
        tmp_dir: &TempDir,
    ) -> Result<Vec<Sample>> {
        let mut out = vec![];
        for e in fs::read_dir(&samples_folder).map_err(|source| Error::ListFolder {
            path: samples_folder.as_ref().into(),
            source,
        })? {
            let path = e?.path();
            if path
                .extension()
                .filter(|x| x.to_string_lossy() == "expected")
                .is_some()
            {
                let name = path
                    .file_stem()
                    .expect("folder should have a file name without extension")
                    .to_string_lossy()
                    .to_string();
                let expected = path.clone();
                let existing = path.with_extension("existing");
                let args_file = path.with_extension("cfg.yaml");
                let destination = tmp_dir.path().join(&name).to_path_buf();
                let sample_cfg = SampleCfg::from_file(args_file)?;
                let args = sample_cfg.make_args(template_loc, destination)?;
                let ignores = sample_cfg.make_ignores()?;
                out.push(Sample {
                    name,
                    args,
                    expected,
                    existing,
                    ignores,
                });
            }
        }
        Ok(out)
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
struct SampleCfg {
    apply_args: Option<Vec<String>>,
    check_ignores: Option<Vec<String>>,
}

impl SampleCfg {
    fn from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let v = if file.as_ref().exists() {
            let cfg_str = fs::read_to_string(file.as_ref()).map_err(|source| Error::ReadFile {
                path: file.as_ref().into(),
                source,
            })?;
            serde_yaml::from_str::<SampleCfg>(&cfg_str)?
        } else {
            SampleCfg::default()
        };
        Ok(v)
    }

    fn make_ignores(&self) -> Result<Vec<PathPattern>> {
        use std::str::FromStr;
        let trim_chars: &[_] = &['\r', '\n', ' ', '\t', '"', '\''];
        let ignores = self
            .check_ignores
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|v| v.trim_matches(trim_chars))
            .filter(|v| !v.is_empty())
            .map(PathPattern::from_str)
            .collect::<Result<Vec<PathPattern>>>()?;
        Ok(ignores)
    }

    fn make_args<B: AsRef<Path>>(
        &self,
        template_loc: &SourceLoc,
        destination: B,
    ) -> Result<ApplyOpts> {
        let cfg_args = self.apply_args.clone().unwrap_or_default();
        let mut args_line = cfg_args.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        args_line.push("--confirm");
        args_line.push("never");
        args_line.push("--no-interaction");
        args_line.push("--destination");
        args_line.push(
            destination
                .as_ref()
                .to_str()
                .expect("to convert destination path into str"),
        );
        args_line.push("--source");
        args_line.push(&template_loc.uri.raw);
        args_line.push("--rev");
        args_line.push(&template_loc.rev);
        let buff = template_loc.subfolder.as_ref().map(|v| v.to_string_lossy());
        if let Some(subfolder) = buff.as_ref() {
            args_line.push("--source-subfolder");
            args_line.push(subfolder);
        }
        //HACK from_iter_safe expect first entry to be the binary name,
        //  unless clap::AppSettings::NoBinaryName has been used
        //  (but I don't know how to use it in this case, patch is welcomed)
        args_line.insert(0, "apply");
        args_line.insert(0, "ffizer");
        CliOpts::try_parse_from(args_line)
            .map_err(Error::from)
            .and_then(|o| match o.cmd {
                Command::Apply(g) => Ok(g),
                e => Err(Error::Unknown(format!(
                    "command should always be parsed as 'apply' not as {:?}",
                    e
                ))),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SampleRun {
    diffs: Vec<EntryDiff>,
}

impl SampleRun {
    #[tracing::instrument]
    pub fn run(sample: &Sample) -> Result<SampleRun> {
        // ALTERNATIVE: fork a sub-process to run current ffizer in apply mode
        let destination = &sample.args.dst_folder;
        if sample.existing.exists() {
            copy(&sample.existing, destination)?;
        }
        let ctx = crate::Ctx {
            cmd_opt: sample.args.clone(),
        };
        crate::process(&ctx)?;
        let diffs = dir_diff_list::search_diff(destination, &sample.expected, &sample.ignores)?;
        Ok(SampleRun { diffs })
    }

    pub fn is_success(&self) -> bool {
        self.diffs.is_empty()
    }
}

impl std::fmt::Display for SampleRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Differences: {:#?}", self.diffs)
    }
}

/// recursively copy a directory
/// based on https://stackoverflow.com/a/60406693/469066
pub fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<()> {
    let mut stack = vec![PathBuf::from(from.as_ref())];
    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        //println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            // println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest).map_err(|source| Error::CreateFolder {
                path: dest.clone(),
                source,
            })?;
        }

        for entry in fs::read_dir(&working_path).map_err(|source| Error::ListFolder {
            path: working_path,
            source,
        })? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(filename) = path.file_name() {
                let dest_path = dest.join(filename);
                //println!("  copy: {:?} -> {:?}", &path, &dest_path);
                fs::copy(&path, &dest_path).map_err(|source| Error::CopyFile {
                    src: path,
                    dst: dest_path,
                    source,
                })?;
            }
        }
    }

    Ok(())
}
