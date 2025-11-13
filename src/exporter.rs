use crate::{config::Config, grafana::Grafana, metrics, mimir::Mimir};

pub struct Exporter {
    grafana: Grafana,
    mimir: Mimir,
}

impl Exporter {
    /// Create a new Exporter instance
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let grafana = Grafana::new(config.grafana.clone())?;
        let mimir = Mimir::new(config.clone());

        Ok(Self { grafana, mimir })
    }

    /// Start the exporter loop
    pub async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("Starting exporter");

        loop {
            if let Err(e) = self.analyze().await {
                tracing::error!("Analysis failed: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(120)).await;
                continue;
            }

            tokio::time::sleep(std::time::Duration::from_secs(86400)).await;
        }
    }

    /// Perform analysis
    #[tracing::instrument(skip(self))]
    async fn analyze(&self) -> anyhow::Result<()> {
        // Fetch tenants
        let tenants = self.mimir.get_tenants().await?;
        tracing::info!("Fetched {} tenants", tenants.len());

        // Analyze Grafana dashboards
        self.mimir.analyze_grafana().await?;

        // Get alert rules
        let alerts = self.grafana.get_alert_rules().await?;

        // Analyze each tenant
        for tenant in tenants {
            let used_metrics = self.mimir.analyze_tenant(&tenant).await?;
            let top_metrics = self.mimir.get_tenant_top_metrics(&tenant).await?;

            for metric in top_metrics {
                let in_use = used_metrics.contains(&metric);
                metrics::set_metric(&metric, &tenant, in_use);

                let in_alerts = self
                    .grafana
                    .find_metric_in_alerts(&alerts, &metric)
                    .unwrap_or(false);

                let status = match (in_use, in_alerts) {
                    (true, _) => "in use",
                    (false, true) => "not in use (may be used by alerts)",
                    (false, false) => "not in use",
                };

                tracing::info!("Metric '{}' in tenant '{}' is {}", metric, tenant, status);
            }
        }

        Ok(())
    }
}
