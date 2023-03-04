// see :
// - [std::error::Error - Rust](https://doc.rust-lang.org/std/error/trait.Error.html)
// - [Error Handling - A Gentle Introduction to Rust](https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html)
// - [snafu::guide::comparison::failure - Rust](https://docs.rs/snafu/0.4.3/snafu/guide/comparison/failure/index.html)
// - [Error Handling in Rust - Andrew Gallant's Blog](https://blog.burntsushi.net/rust-error-handling/)
// use std::backtrace::Backtrace;
use std::path::PathBuf;
use thiserror::Error;
use tracing_error::SpanTrace;

use crate::git::GitError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)] // warn to restore to default
pub enum Error {
    #[error("unknown ffizer error: {0}")]
    Unknown(String),

    #[error("value {value:?} of {value_name} is not in {accepted:?}")]
    StringValueNotIn {
        value_name: String,
        value: String,
        accepted: Vec<String>,
    },

    #[error("git retreive {url:?} (rev: {rev:?}) into folder {dst:?}")]
    GitRetrieve {
        dst: PathBuf,
        url: String,
        rev: String,
        source: GitError,
        msg: String,
    },
    #[error("try to find git config '{key:?}'")]
    GitFindConfig { key: String, source: GitError },

    #[error("canonicalize {path:?}")]
    CanonicalizePath {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("create folder {path:?}")]
    CreateFolder {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("create temp folder")]
    CreateTmpFolder { source: std::io::Error },
    #[error("remove folder {path:?}")]
    RemoveFolder {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("list content of folder {path:?}")]
    ListFolder {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("create file {path:?}")]
    CreateFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("rename file from {src:?} to {dst:?}")]
    RenameFile {
        src: PathBuf,
        dst: PathBuf,
        source: std::io::Error,
    },
    #[error("copy file from {src:?} to {dst:?}")]
    CopyFile {
        src: PathBuf,
        dst: PathBuf,
        source: std::io::Error,
    },
    #[error("copy permission from {src:?} to {dst:?}")]
    CopyFilePermission {
        src: PathBuf,
        dst: PathBuf,
        source: std::io::Error,
    },
    #[error("read file {path:?}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("write file {path:?}")]
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("remove file {path:?}")]
    RemoveFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("run command '{cmd:?}'")]
    RunCommand { cmd: String, source: std::io::Error },
    #[error("fail to parse string as path '{value:?}'")]
    ParsePathPattern {
        value: String,
        source: globset::Error,
    },

    #[error("fail to parse string as uri for git repo '{value:?}'")]
    ParseGitUri { value: String, source: regex::Error },

    #[error("local path({path:?}) not found for uri({uri:?}) subfolder({subfolder:?})")]
    LocalPathNotFound {
        path: PathBuf,
        uri: String,
        subfolder: Option<PathBuf>,
    },

    #[error("Application directory not found")]
    ApplicationPathNotFound {},

    #[error("test samples failed")]
    TestSamplesFailed {},

    #[error("failed to parse value '{value}' for variable '{name}'")]
    ReadVariable { name: String, value: String },

    #[error(transparent)]
    // #[error("fail to process io")]
    Io {
        #[from]
        source: std::io::Error,
        // backtrace: Backtrace,
    },
    #[error("fail to process template '{template}' when {when}")]
    Handlebars {
        when: String,
        template: String,
        source: handlebars::RenderError,
    },
    // #[error(transparent)]
    #[error("fail to process yaml")]
    SerdeYaml {
        context: SpanTrace,
        #[source]
        source: serde_yaml::Error,
        // backtrace: Backtrace,
    },
    #[error("fail to process script '{script}'")]
    ScriptError {
        script: String,
        source: run_script::ScriptError,
    },
    #[error(transparent)]
    SerdeJson {
        #[from]
        source: serde_json::Error,
    },
    #[error(transparent)]
    WalkDir {
        #[from]
        source: walkdir::Error,
    },
    #[error(transparent)]
    PathStripPrefixError {
        #[from]
        source: std::path::StripPrefixError,
    },
    #[error(transparent)]
    Clap {
        #[from]
        source: clap::Error,
    },
}

impl From<serde_yaml::Error> for Error {
    fn from(source: serde_yaml::Error) -> Self {
        Error::SerdeYaml {
            context: SpanTrace::capture(),
            source,
        }
    }
}
