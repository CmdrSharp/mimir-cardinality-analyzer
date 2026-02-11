use clap::Parser;
use std::path::PathBuf;
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
};

pub mod config;
pub mod exporter;
pub mod grafana;
pub mod http;
pub mod metrics;
pub mod mimir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Config file
    #[arg(short, long)]
    pub config: PathBuf,

    /// Output directory for intermediate files (grafana.json, prometheus-metrics.json)
    #[arg(short, long, default_value = ".")]
    pub output_dir: PathBuf,
}

/// Handle signals
pub fn signal_handler() {
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();

        select! {
            _ = sigterm.recv() => {
                tracing::info!("SIGTERM received, exiting");
                std::process::exit(0);
            }
            _ = sigint.recv() => {
                tracing::info!("SIGINT received, exiting");
                std::process::exit(0);
            }
        }
    });
}
