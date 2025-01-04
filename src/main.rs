mod airports;
mod waypoints;

use air_traffic_simulator::WorldData;
use color_eyre::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use crate::airports::airports;
use crate::waypoints::waypoints;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(EnvFilter::from_env("RUST_LOG"))
        .with(fmt::layer())
        .try_init()?;

    let mut world_data: WorldData = serde_yaml::from_str(include_str!("wd.yml"))?;

    airports(&mut world_data).await?;
    waypoints(&mut world_data).await?;

    let engine_config = serde_yaml::from_str(include_str!("engine_config.yml"))?;
    let engine = air_traffic_simulator::Engine::new(world_data, engine_config);

    air_traffic_simulator::run_server(engine, Some(include_str!("client_config.js"))).await?;

    Ok(())
}
