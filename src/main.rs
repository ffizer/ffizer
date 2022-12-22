use clap::Parser;
use ffizer::provide_json_schema;
use ffizer::ApplyOpts;
use ffizer::CliOpts;
use ffizer::Command;
use ffizer::Ctx;
use ffizer::SourceLoc;
use ffizer::TestSamplesOpts;
use std::error::Error;
use tracing::{debug, error, info, trace};
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter, fmt};

fn tracing_level_from_usize(level: u8) -> tracing::Level {
    match level {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        3 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    }
}

fn init_log(level_min: tracing::Level) {
    // a builder for `FmtSubscriber`.
    let fmt_layer = fmt::layer()
        // .with_target(false)
        .with_writer(std::io::stderr)
        .pretty();
    // let filter_layer = filter::EnvFilter::try_from_default_env()
    //     .or_else(|_| EnvFilter::try_new("info"))
    //     .unwrap();
    let filter_layer = filter::LevelFilter::from_level(level_min);
    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    info!(version = env!("CARGO_PKG_VERSION"), "start");
    debug!("debug enabled");
    trace!("trace enabled");
}

#[tracing::instrument]
fn upgrade() -> Result<(), Box<dyn Error>> {
    let target = self_update::get_target();
    // TODO extract repo info from CARGO_PKG_REPOSITORY
    let status = self_update::backends::github::Update::configure()
        .repo_owner("ffizer")
        .repo_name("ffizer")
        .target(target)
        .bin_name("ffizer")
        .show_download_progress(true)
        //.current_version(self_update::cargo_crate_version!())
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    info!(status = status.version(), "success");
    Ok(())
}

#[tracing::instrument]
fn apply(cmd_opt: ApplyOpts) -> Result<(), Box<dyn Error>> {
    let ctx = Ctx { cmd_opt };
    ffizer::process(&ctx)?;
    Ok(())
}

#[tracing::instrument]
fn inspect() -> Result<(), Box<dyn Error>> {
    println!(
        "remote cache folder: {}",
        SourceLoc::find_remote_cache_folder()?.to_string_lossy()
    );
    Ok(())
}

#[tracing::instrument]
fn show_json_schema() -> Result<(), Box<dyn Error>> {
    let schema = provide_json_schema()?;
    println!("{}", schema);
    Ok(())
}

#[tracing::instrument]
fn test_samples(cfg: &TestSamplesOpts) -> Result<(), Box<dyn Error>> {
    ffizer::tools::test_samples(cfg)?;
    Ok(())
}

fn main() {
    human_panic::setup_panic!();
    let cli_opts = CliOpts::parse();

    let log_level = tracing_level_from_usize(1 + cli_opts.verbose);
    init_log(log_level);
    debug!(cmd = ?&cli_opts, "parsed args");

    let r = match &cli_opts.cmd {
        Command::Apply(g) => apply(g.clone()),
        Command::Upgrade => upgrade(),
        Command::Inspect => inspect(),
        Command::ShowJsonSchema => show_json_schema(),
        Command::TestSamples(g) => test_samples(g),
    };
    if let Err(e) = r {
        error!("cmd: {:#?}", &cli_opts);
        error!("failed: {:#?}", &e);
        std::process::exit(1)
    }
}
