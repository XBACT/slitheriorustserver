




use clap::Parser;
use rust_slither::config::{GameConfig, ServerArgs};
use rust_slither::server::run_server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
   
    let args = ServerArgs::parse();

   
    let log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .init();

   
    let mut config = GameConfig::default();
    config.initial_bots = args.bots;
    config.bot_respawn = args.bot_respawn;

    info!("===========================================");
    info!("    Rust Slither.io Server v0.1.0");
    info!("===========================================");
    info!("");
    info!("Configuration:");
    info!("  Port: {}", args.port);
    info!("  Game radius: {}", config.game_radius);
    info!("  Sector size: {}", config.sector_size);
    info!("  Protocol version: {}", config.protocol_version);
    info!("  Initial bots: {}", config.initial_bots);
    info!("  Bot respawn: {}", config.bot_respawn);
    info!("");

   
    run_server(args.port, config).await
}
