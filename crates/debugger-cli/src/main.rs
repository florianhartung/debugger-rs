use std::path::PathBuf;

use clap::Parser;
use debugger_core::add;
use envconfig::Envconfig;
use log::{LevelFilter, info};

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "RUST_LOG", default = "INFO")]
    pub log_level: LevelFilter,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    executable_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    // For development/testing only
    let _ = dotenvy::dotenv();

    let config = Config::init_from_env()?;
    env_logger::builder().filter_level(config.log_level).init();

    let args = Args::parse();

    info!("TODO: open {:?} now", args.executable_path);

    info!("1 + 2 = {}", add(1, 2));

    Ok(())
}
