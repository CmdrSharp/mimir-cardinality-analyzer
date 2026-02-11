use clap::Parser;
use mimir_cardinality_analyzer::{Args, config, exporter::Exporter, http, metrics, signal_handler};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Register metrics
    metrics::register_metrics();

    // Parse config
    let args = Args::parse();
    let config = config::Config::from_file(&args.config)?.with_output_dir(args.output_dir);

    // Handle signals
    signal_handler();

    // Create and start exporter
    tokio::spawn({
        let config = config.clone();

        async move {
            let exporter = Exporter::new(config).unwrap();
            exporter.start().await.unwrap();
        }
    });

    // Start the HTTP server
    http::create_server(config).await;

    Ok(())
}
