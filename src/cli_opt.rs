use crate::source_loc::SourceLoc;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
// #[arg(
//     raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
//     rename_all = "kebab-case",
//     raw(author = "env!(\"CARGO_PKG_HOMEPAGE\")")
// )]
#[command(
    version, author = env!("CARGO_PKG_HOMEPAGE"), about, 
)]
#[command(propagate_version = true)]
pub struct CliOpts {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv (very verbose / level debug), -vvv)
    /// print on stderr
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Apply a template into a target directory
    Apply(ApplyOpts),

    /// Self upgrade ffizer executable
    Upgrade,

    /// Inspect configuration, caches,... (wip)
    Inspect,

    /// Show the json schema of the .ffizer.yaml files
    ShowJsonSchema,

    /// test a template against its samples
    TestSamples(TestSamplesOpts),
}

#[derive(Args, Debug, Default, Clone)]
pub struct ApplyOpts {
    /// ask for plan confirmation
    #[arg(long, default_value = "Never", value_enum, ignore_case = true)]
    pub confirm: AskConfirmation,

    /// mode to update existing file
    #[arg(long, default_value = "Ask", value_enum, ignore_case = true)]
    pub update_mode: UpdateMode,

    /// should not ask for confirmation (to use default value, to apply plan, to override, to run script,...)
    #[arg(short = 'y', long = "no-interaction")]
    pub no_interaction: bool,

    /// in offline, only local templates or cached templates are used
    #[arg(long = "offline")]
    pub offline: bool,

    #[command(flatten)]
    pub src: SourceLoc,

    /// destination folder (created if doesn't exist)
    #[arg(
        short = 'd',
        long = "destination",
        //default_value = "."
        value_name = "FOLDER"
    )]
    pub dst_folder: PathBuf,

    /// set variable's value from cli ("key=value")
    #[arg(short = 'v', long = "variables", value_parser = parse_keyvalue)]
    pub key_value: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
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

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
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

fn parse_keyvalue(src: &str) -> Result<(String, String), String> {
    let kv: Vec<&str> = src.splitn(2, '=').collect();
    if kv.len() == 2 {
        Ok((kv[0].to_owned(), kv[1].to_owned()))
    } else {
        Ok((src.to_owned(), "".to_owned()))
    }
}

#[derive(Parser, Debug, Default, Clone)]
pub struct TestSamplesOpts {
    #[command(flatten)]
    pub src: SourceLoc,
    /// in offline, only local templates or cached templates are used
    #[arg(long = "offline")]
    pub offline: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        CliOpts::command().debug_assert()
    }
}
