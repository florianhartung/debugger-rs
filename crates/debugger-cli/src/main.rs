use debugger_core::add;
use envconfig::Envconfig;
use log::{LevelFilter, info};

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "RUST_LOG", default = "INFO")]
    pub log_level: LevelFilter,
}

fn main() -> anyhow::Result<()> {
    // For development/testing only
    let _ = dotenvy::dotenv();

    let config = Config::init_from_env()?;
    env_logger::builder().filter_level(config.log_level).init();

    info!("1 + 2 = {}", add(1, 2));

    Ok(())
}
