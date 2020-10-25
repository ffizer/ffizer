use ffizer::provide_json_schema;
use ffizer::ApplyOpts;
use ffizer::CliOpts;
use ffizer::Command;
use ffizer::Ctx;
use ffizer::SourceLoc;
use self_update;
use slog::Drain;
use slog::{debug, error, info, o, trace};
use std::error::Error;
use structopt::StructOpt;

fn init_log(level_min: slog::Level) -> slog::Logger {
    let drain = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let drain = slog_term::FullFormat::new(drain)
        .build()
        .filter_level(level_min)
        .fuse();
    let log = slog::Logger::root(drain, o!());
    slog_stdlog::init().unwrap();
    info!(log, "start"; "version" => env!("CARGO_PKG_VERSION"));
    debug!(log, "debug enabled");
    trace!(log, "trace enabled");
    log
}

fn upgrade(logger: slog::Logger) -> Result<(), Box<dyn Error>> {
    let target = self_update::get_target();
    // TODO extract repo info from CARGO_PKG_REPOSITORY
    let status = self_update::backends::github::Update::configure()
        .repo_owner("ffizer")
        .repo_name("ffizer")
        .target(&target)
        .bin_name("ffizer")
        .show_download_progress(true)
        //.current_version(self_update::cargo_crate_version!())
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    info!(logger, "success"; "status" => status.version());
    Ok(())
}

fn apply(logger: slog::Logger, cmd_opt: ApplyOpts) -> Result<(), Box<dyn Error>> {
    let ctx = Ctx { logger, cmd_opt };
    ffizer::process(&ctx)?;
    Ok(())
}

fn inspect() -> Result<(), Box<dyn Error>> {
    println!(
        "remote cache folder: {}",
        SourceLoc::find_remote_cache_folder()?.to_string_lossy()
    );
    Ok(())
}

fn show_json_schema() -> Result<(), Box<dyn Error>> {
    let schema = provide_json_schema()?;
    println!("{}", schema);
    Ok(())
}

fn main() {
    human_panic::setup_panic!();
    let cli_opts = CliOpts::from_args();

    let log_level = slog::Level::from_usize(3 + cli_opts.verbose).unwrap_or(slog::Level::Warning);
    let logger = init_log(log_level);
    debug!(logger, "parsed args"; "cmd" => ?&cli_opts);

    let r = match &cli_opts.cmd {
        Command::Apply(g) => apply(logger.new(o!("sub-cmd" => "apply")), g.clone()),
        Command::Upgrade => upgrade(logger.new(o!("sub-cmd" => "upgrade"))),
        Command::Inspect => inspect(),
        Command::ShowJsonSchema => show_json_schema(),
    };
    if let Err(e) = r {
        error!(logger, "failed"; "error" => ?&e, "cmd" => ?&cli_opts);
        std::process::exit(1)
    }
}
