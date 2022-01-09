use crate::source_loc::SourceLoc;
use clap::{ArgEnum, Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
// #[clap(
//     raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
//     rename_all = "kebab-case",
//     raw(author = "env!(\"CARGO_PKG_HOMEPAGE\")")
// )]
#[clap(
    version, author = env!("CARGO_PKG_HOMEPAGE"), about, 
)]
pub struct CliOpts {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv (very verbose / level debug), -vvv)
    /// print on stderr
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub verbose: usize,

    #[clap(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Apply a template into a target directory
    #[clap(version, author = env!("CARGO_PKG_HOMEPAGE"))]
    Apply(ApplyOpts),
    /// Self upgrade ffizer executable
    #[clap(version, author = env!("CARGO_PKG_HOMEPAGE"))]
    Upgrade,
    /// Inspect configuration, caches,... (wip)
    #[clap(version, author = env!("CARGO_PKG_HOMEPAGE"))]
    Inspect,
    /// Show the json schema of the .ffizer.yaml files
    #[clap(version, author = env!("CARGO_PKG_HOMEPAGE"))]
    ShowJsonSchema,
    /// test a template against its samples
    #[clap(version, author = env!("CARGO_PKG_HOMEPAGE"))]
    TestSamples(TestSamplesOpts),
}

#[derive(Args, Debug, Default, Clone)]
pub struct ApplyOpts {
    /// ask for plan confirmation
    #[clap(long, default_value = "Never", arg_enum, ignore_case = true)]
    pub confirm: AskConfirmation,

    /// mode to update existing file
    #[clap(long, default_value = "Ask", arg_enum, ignore_case = true)]
    pub update_mode: UpdateMode,

    /// should not ask for confirmation (to use default value, to apply plan, to override, to run script,...)
    #[clap(short = 'y', long = "no-interaction")]
    pub no_interaction: bool,

    /// in offline, only local templates or cached templates are used
    #[clap(long = "offline")]
    pub offline: bool,

    #[clap(flatten)]
    pub src: SourceLoc,

    /// destination folder (created if doesn't exist)
    #[clap(
        short = 'd',
        long = "destination",
        parse(from_os_str),
        //default_value = "."
    )]
    pub dst_folder: PathBuf,

    /// set variable's value from cli ("key=value")
    #[clap(short = 'v', long = "variables", parse(from_str=parse_keyvalue))]
    pub key_value: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, ArgEnum)]
pub enum AskConfirmation {
    Auto,
    Always,
    Never,
}

impl Default for AskConfirmation {
    fn default() -> Self {
        AskConfirmation::Auto
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ArgEnum)]
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

impl Default for UpdateMode {
    fn default() -> Self {
        UpdateMode::Ask
    }
}

impl std::fmt::Display for UpdateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            self.to_possible_value()
                .expect("UpdateMode should have a possible value")
                .get_name()
        ))
    }
}

fn parse_keyvalue(src: &str) -> (String, String) {
    let kv: Vec<&str> = src.splitn(2, '=').collect();
    if kv.len() == 2 {
        (kv[0].to_owned(), kv[1].to_owned())
    } else {
        (src.to_owned(), "".to_owned())
    }
}

#[derive(Parser, Debug, Default, Clone)]
pub struct TestSamplesOpts {
    #[clap(flatten)]
    pub src: SourceLoc,
    /// in offline, only local templates or cached templates are used
    #[clap(long = "offline")]
    pub offline: bool,
}
