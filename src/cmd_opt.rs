use crate::source_uri::SourceUri;
use failure::format_err;
use failure::Error;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Default, Clone)]
#[structopt(
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    author = "davidB"
)]
pub struct CmdOpt {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv (very verbose / level debug), -vvv)
    /// print on stderr
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: usize,

    /// ask confirmation 'never' or 'always'
    #[structopt(long = "confirm", default_value = "never")]
    pub confirm: AskConfirmation,

    /// should not ask for valiables values, always use defautl value or empty (experimental)
    #[structopt(long = "x-always_default_value")]
    pub x_always_default_value: bool,

    /// in offline, only local templates or cached templates are used
    #[structopt(long = "offline")]
    pub offline: bool,

    /// uri / path of the template
    #[structopt(short = "s", long = "source",)]
    pub src_uri: SourceUri,

    /// git revision of the template
    #[structopt(long = "rev", default_value = "master")]
    pub src_rev: String,

    /// path of the folder under the source uri to use for template
    #[structopt(long = "source-folder", parse(from_os_str))]
    pub src_folder: Option<PathBuf>,

    /// destination folder (created if doesn't exist)
    #[structopt(
        short = "d",
        long = "destination",
        parse(from_os_str),
        //default_value = "."
    )]
    pub dst_folder: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskConfirmation {
    Auto,
    Always,
    Never,
}

impl FromStr for AskConfirmation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(AskConfirmation::Never),
            "always" => Ok(AskConfirmation::Always),
            "auto" => Ok(AskConfirmation::Auto),
            _ => Err(format_err!(
                "should be 'never', 'always' or 'auto' (default)"
            )),
        }
    }
}

impl Default for AskConfirmation {
    fn default() -> Self {
        AskConfirmation::Auto
    }
}
