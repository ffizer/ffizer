use crate::source_loc::SourceLoc;
use failure::format_err;
use failure::Error;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    rename_all = "kebab-case",
    raw(author = "env!(\"CARGO_PKG_HOMEPAGE\")")
)]
pub struct CliOpts {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv (very verbose / level debug), -vvv)
    /// print on stderr
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: usize,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: Command,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    rename_all = "kebab-case",
    raw(author = "env!(\"CARGO_PKG_HOMEPAGE\")")
)]
pub enum Command {
    /// Apply a template into a target directory
    #[structopt(
        raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
        rename_all = "kebab-case",
        raw(author = "env!(\"CARGO_PKG_HOMEPAGE\")")
    )]
    Apply(ApplyOpts),
    /// Self upgrade ffizer executable
    #[structopt(
        raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
        rename_all = "kebab-case",
        raw(author = "env!(\"CARGO_PKG_HOMEPAGE\")")
    )]
    Upgrade,
}

#[derive(StructOpt, Debug, Default, Clone)]
pub struct ApplyOpts {
    /// ask confirmation 'never' or 'always'
    #[structopt(long = "confirm", default_value = "never")]
    pub confirm: AskConfirmation,

    /// should not ask for valiables values, always use defautl value or empty (experimental)
    #[structopt(long = "x-always_default_value")]
    pub x_always_default_value: bool,

    /// in offline, only local templates or cached templates are used
    #[structopt(long = "offline")]
    pub offline: bool,

    #[structopt(flatten)]
    pub src: SourceLoc,

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
