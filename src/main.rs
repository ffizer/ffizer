#[macro_use]
extern crate slog;
extern crate ffizer;
extern crate slog_async;
extern crate slog_term;
extern crate structopt;

use slog::Drain;
//use std::env;
use ffizer::Ctx;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    author = "davidB"
)]
struct Cmd {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    /// print on stderr
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// root directory of the search
    #[structopt(
        name = "TEMPLATE_URI",
        //parse(from_os_str),
        default_value = "."
    )]
    template_uri: String,

    /// root directory of the search
    #[structopt(
        name = "DEST_FOLDER",
        parse(from_os_str),
        default_value = "."
    )]
    dest_folder: PathBuf,
}

fn init_log(level_min: slog::Level) -> slog::Logger {
    let drain = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(drain).build().fuse();
    let drain = slog_async::Async::new(drain)
        .build()
        .filter_level(level_min)
        .fuse();
    let log = slog::Logger::root(drain, o!());
    info!(log, "start"; "version" => env!("CARGO_PKG_VERSION"));
    debug!(log, "debug enabled");
    trace!(log, "trace enabled");
    log
}

fn main() -> Result<(), Box<Error>> {
    let cmd = Cmd::from_args();

    let log_level = slog::Level::from_usize(3 + cmd.verbose).unwrap_or(slog::Level::Warning);
    let logger = init_log(log_level);
    debug!(logger, "parsed args"; "cmd" => format!("{:?}", &cmd));

    let ctx = Ctx {
        logger,
        dest_folder: fs::canonicalize(&cmd.dest_folder)?.clone(),
        template_uri: cmd.template_uri.clone(),
    };

    println!("todo process {:?}", &ctx);
    ffizer::process(ctx)
}
