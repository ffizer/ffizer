use failure::Error;
use ffizer::ApplyOpts;
use ffizer::CliOpts;
use ffizer::Command;
use ffizer::Ctx;
use self_update;
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

fn upgrade(logger: slog::Logger) -> Result<(), Error> {
    let target = self_update::get_target()?;
    // TODO extract repo info from CARGO_PKG_REPOSITORY
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
    info!(logger, "ugrade"; "status" => status.version());
    Ok(())
}

fn apply(logger: slog::Logger, cmd_opt: ApplyOpts) -> Result<(), Error> {
    let ctx = Ctx { logger, cmd_opt };
    ffizer::process(&ctx)
}

fn main() -> Result<(), Error> {
    human_panic::setup_panic!();
    let cli_opts = CliOpts::from_args();

    let log_level = slog::Level::from_usize(3 + cli_opts.verbose).unwrap_or(slog::Level::Warning);
    let logger = init_log(log_level);
    debug!(logger, "parsed args"; "cmd" => format!("{:?}", &cli_opts));

    match cli_opts.cmd {
        Command::Apply(g) => apply(logger, g),
        Command::Upgrade => upgrade(logger),
    }
}
