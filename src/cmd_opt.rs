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

    /// ask confirmation 'never', 'always' or 'auto' (default)
    #[structopt(long = "confirm", default_value = "auto")]
    pub confirm: AskConfirmation,

    /// uri / path of the template
    #[structopt(
        short = "s",
        long = "source",
        //parse(from_os_str),
        //default_value = "."
    )]
    pub src_uri: String,

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
