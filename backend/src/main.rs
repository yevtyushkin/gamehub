use crate::config::application_config::ApplicationConfig;

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let application_config = ApplicationConfig::from_environment()?;

    Ok(())
}
