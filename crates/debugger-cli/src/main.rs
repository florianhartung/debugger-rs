use std::path::PathBuf;

use clap::Parser;
use debugger_core::{ContinueExecutionOutcome, Debugger};
use envconfig::Envconfig;
use log::{LevelFilter, error, info};

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

    let mut debugger = Debugger::new_with_forked_child(args.executable_path).unwrap();

    for _ in 0..10 {
        info!("Continuing execution...");
        match debugger.continue_execution() {
            Ok(ContinueExecutionOutcome::ProcessExited(code)) => {
                info!("Process exited with code {code}");
                break;
            }
            Ok(ContinueExecutionOutcome::Other) => {}
            Err(err) => {
                error!("Error: {err}");
            }
        }
    }

    Ok(())
}
