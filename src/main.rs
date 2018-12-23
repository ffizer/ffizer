extern crate failure;
extern crate ffizer;
extern crate human_panic;
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate structopt;

use self_update;
use failure::Error;
use ffizer::CmdOpt;
use ffizer::Command;
use ffizer::Ctx;
use slog::Drain;
use slog::{debug, info, o, trace};
use structopt::StructOpt;

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

fn upgrade() -> Result<(), Error> {
    let target = self_update::get_target()?;
    let status = self_update::backends::github::Update::configure()?
        .repo_owner("davidB")
        .repo_name("ffizer")
        .target(&target)
        .bin_name("ffizer")
        .show_download_progress(true)
        //.current_version(self_update::cargo_crate_version!())
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    println!("Update status: `{}`!", status.version());
    Ok(())
}

fn apply(cmd_opt: CmdOpt) -> Result<(), Error> {
    let log_level = slog::Level::from_usize(3 + cmd_opt.verbose).unwrap_or(slog::Level::Warning);
    let logger = init_log(log_level);
    debug!(logger, "parsed args"; "cmd" => format!("{:?}", &cmd_opt));

    let ctx = Ctx { logger, cmd_opt };

    ffizer::process(&ctx)
}

fn main() -> Result<(), Error> {
    human_panic::setup_panic!();
    match Command::from_args() {
        Command::Apply(g) => apply(g),
        Command::Upgrade => upgrade(),
    }


}
