pub mod dir_diff_list;

use crate::cli_opt::{ApplyOpts, TestSamplesOpts};
use crate::error::*;
use dir_diff_list::EntryDiff;
use slog::{info, o, warn, Logger};
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tempfile::{tempdir, TempDir};

pub fn test_samples(logger: &Logger, cfg: &TestSamplesOpts) -> Result<()> {
    let template_base_path = &cfg.src.download(&logger, cfg.offline)?;
    if !check_samples(&logger, template_base_path)? {
        Err(crate::Error::TestSamplesFailed {})
    } else {
        Ok(())
    }
}

fn check_samples<A: AsRef<Path>>(logger: &Logger, template_path: A) -> Result<bool> {
    let mut is_success = true;
    let tmp_dir = tempdir()?;
    let samples_folder = template_path
        .as_ref()
        .join(crate::cfg::TEMPLATE_SAMPLES_DIRNAME);
    let samples = Sample::find_from_folder(&template_path, &samples_folder, &tmp_dir)?;
    info!(logger, "nb samples detected: {}", samples.len(); "samples_folder" => ?&samples_folder);
    for sample in samples {
        let run_logger = logger.new(o!("sample" => sample.name.clone()));
        info!(run_logger, "checking...");
        let run = SampleRun::run(run_logger.clone(), &sample)?;
        if !run.is_success() {
            is_success = false;
            warn!(run_logger, "check failed {}", run);
        }
    }
    Ok(is_success)
}

#[derive(Debug, Clone)]
struct Sample {
    pub name: String,
    pub args: ApplyOpts,
    pub expected: PathBuf,
    pub existing: PathBuf,
}

impl Sample {
    // scan folder to find sample to test (xxx.args, xxx.expected, xxx.existing)
    fn find_from_folder<A: AsRef<Path>, B: AsRef<Path>>(
        template_path: A,
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
                let args = read_args(&template_path, destination, args_file)?;
                out.push(Sample {
                    name,
                    args,
                    expected,
                    existing,
                });
            }
        }
        Ok(out)
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
struct SampleCfg {
    apply_args: Vec<String>,
}

fn read_args<A: AsRef<Path>, B: AsRef<Path>, C: AsRef<Path>>(
    template_path: A,
    destination: B,
    args_file: C,
) -> Result<ApplyOpts> {
    let sample_cfg = if args_file.as_ref().exists() {
        let cfg_str = fs::read_to_string(args_file.as_ref()).map_err(|source| Error::ReadFile {
            path: args_file.as_ref().into(),
            source,
        })?;
        serde_yaml::from_str::<SampleCfg>(&cfg_str)?
    } else {
        SampleCfg { apply_args: vec![] }
    };
    let mut args_line = sample_cfg
        .apply_args
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();
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
    args_line.push(
        template_path
            .as_ref()
            .to_str()
            .expect("to convert source path into str"),
    );
    //HACK from_iter_safe expect first entry to be the binary name,
    //  unless clap::AppSettings::NoBinaryName has been used
    //  (but I don't know how to use it in this case, patch is welcomed)
    args_line.insert(0, "ffizer apply");
    ApplyOpts::from_iter_safe(args_line).map_err(Error::from)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SampleRun {
    diffs: Vec<EntryDiff>,
}

impl SampleRun {
    pub fn run(logger: slog::Logger, sample: &Sample) -> Result<SampleRun> {
        // ALTERNATIVE: fork a sub-process to run current ffizer in apply mode
        let destination = &sample.args.dst_folder;
        if sample.existing.exists() {
            copy(&sample.existing, destination)?;
        }
        let ctx = crate::Ctx {
            logger,
            cmd_opt: sample.args.clone(),
        };
        crate::process(&ctx)?;
        let diffs = dir_diff_list::search_diff(destination, &sample.expected)?;
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
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

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
                path: dest.clone().into(),
                source,
            })?;
        }

        for entry in fs::read_dir(&working_path).map_err(|source| Error::ListFolder {
            path: working_path.into(),
            source,
        })? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        //println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        fs::copy(&path, &dest_path).map_err(|source| Error::CopyFile {
                            src: path.into(),
                            dst: dest_path.into(),
                            source,
                        })?;
                    }
                    None => {
                        //println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}
