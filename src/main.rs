extern crate failure;
extern crate ffizer;
extern crate human_panic;
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate structopt;

use failure::Error;
use ffizer::CmdOpt;
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

fn main() -> Result<(), Error> {
    human_panic::setup_panic!();
    let cmd_opt = CmdOpt::from_args();

    let log_level = slog::Level::from_usize(3 + cmd_opt.verbose).unwrap_or(slog::Level::Warning);
    let logger = init_log(log_level);
    debug!(logger, "parsed args"; "cmd" => format!("{:?}", &cmd_opt));

    let ctx = Ctx { logger, cmd_opt };

    ffizer::process(&ctx)
}
