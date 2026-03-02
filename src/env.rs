use anyhow::Result;
use dotenvy::dotenv;
use env_logger::Env;

pub fn load_env() -> Result<()> {
    dotenv()?;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    Ok(())
}
