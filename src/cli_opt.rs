use crate::source_loc::SourceLoc;
use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
// #[structopt(
//     raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
//     rename_all = "kebab-case",
//     raw(author = "env!(\"CARGO_PKG_HOMEPAGE\")")
// )]
#[structopt(
    global_settings(&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]),
    author = env!("CARGO_PKG_HOMEPAGE"), about
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
pub enum Command {
    /// Apply a template into a target directory
    #[structopt(author = env!("CARGO_PKG_HOMEPAGE"))]
    Apply(ApplyOpts),
    /// Self upgrade ffizer executable
    #[structopt(author = env!("CARGO_PKG_HOMEPAGE"))]
    Upgrade,
    /// Inspect configuration, caches,... (wip)
    #[structopt(author = env!("CARGO_PKG_HOMEPAGE"))]
    Inspect,
}

#[derive(StructOpt, Debug, Default, Clone)]
pub struct ApplyOpts {
    /// ask for plan confirmation
    #[structopt(long, default_value = "Never", possible_values = &AskConfirmation::variants(), case_insensitive = true)]
    pub confirm: AskConfirmation,

    /// mode to update existing file
    #[structopt(long, default_value = "Ask", possible_values = &UpdateMode::variants(), case_insensitive = true)]
    pub update_mode: UpdateMode,

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

arg_enum! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AskConfirmation {
        Auto,
        Always,
        Never,
    }
}

impl Default for AskConfirmation {
    fn default() -> Self {
        AskConfirmation::Auto
    }
}

arg_enum! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    /// mode to process update of existing local file
    pub enum UpdateMode {
        // ask what to do
        Ask,
        // keep existing local file (ignore template)
        Keep,
        // override local file with file from template
        Override,
        // keep existing local file, add template with extension .REMOTE
        UpdateAsRemote,
        // rename existing local file with extension .LOCAL, add template file
        CurrentAsLocal,
        // show diff then ask
        ShowDiff,
        // try to merge existing local with remote template via merge tool (defined in the git's configuration)
        Merge,
    }
}

impl Default for UpdateMode {
    fn default() -> Self {
        UpdateMode::Ask
    }
}
