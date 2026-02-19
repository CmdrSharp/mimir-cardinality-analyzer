use crate::{
    config::Config,
    metrics::{
        self,
        external::{Command as ExternalCommand, Target},
    },
};
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::process::Command;

pub mod cardinality;

pub struct Mimir {
    config: Config,
    client: Client,
}

impl Mimir {
    /// Create a new Mimir instance
    pub fn new(config: Config) -> Self {
        let client = Client::new();

        Self { config, client }
    }

    /// Get a list of tenants from the store-gateway
    pub async fn get_tenants(&self) -> anyhow::Result<Vec<String>> {
        tracing::info!("Fetching tenants from store-gateway");
        let _timer = metrics::external::external_request_timer(Target::StoreGateway);

        let url = format!(
            "{}/store-gateway/tenants",
            self.config.mimir.store_gateway_url
        );

        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            metrics::external::record_external_request_failure(Target::StoreGateway);

            return Err(anyhow::anyhow!(
                "Failed to fetch tenants: HTTP {}",
                resp.status()
            ));
        }

        let body = resp.text().await?;
        let document = Html::parse_document(&body);
        let selector = Selector::parse("table tbody tr td a")
            .map_err(|e| anyhow::anyhow!("Failed to parse HTML selector: {}", e))?;

        // Get all the tenant names
        let tenants: Vec<String> = document
            .select(&selector)
            .filter_map(|element| Some(element.text().next()?.to_string()))
            .collect();

        // Record the number of tenants discovered
        metrics::analysis::record_tenants_discovered(tenants.len() as u64);

        Ok(tenants)
    }

    /// Analyze Grafana instance
    #[tracing::instrument(skip(self))]
    pub async fn analyze_grafana(&self) -> anyhow::Result<()> {
        tracing::info!("Analyzing metric usage in dashboards");
        let _timer = metrics::external::mimirtool_timer(ExternalCommand::AnalyzeGrafana);

        let grafana_output = self.config.output_dir.join("grafana.json");
        let grafana_output = grafana_output.to_string_lossy();

        let args = vec![
            "analyze",
            "grafana",
            "--address",
            &self.config.grafana.url,
            "--key",
            &self.config.grafana.token,
            "--output",
            &grafana_output,
        ];

        match Command::new("mimirtool").args(args).output().await {
            Ok(output) => {
                if !output.status.success() {
                    metrics::external::record_mimirtool_execution(
                        ExternalCommand::AnalyzeGrafana,
                        metrics::Status::Failure,
                    );

                    let stderr = String::from_utf8_lossy(&output.stderr);

                    return Err(anyhow::anyhow!(
                        "Mimirtool command failed: {}",
                        stderr.trim()
                    ));
                }

                metrics::external::record_mimirtool_execution(
                    ExternalCommand::AnalyzeGrafana,
                    metrics::Status::Success,
                );

                Ok(())
            }
            Err(e) => {
                metrics::external::record_mimirtool_execution(
                    ExternalCommand::AnalyzeGrafana,
                    metrics::Status::Failure,
                );

                return Err(anyhow::anyhow!("Failed to execute mimirtool: {}", e));
            }
        }
    }

    /// Analyze tenant in Mimir
    #[tracing::instrument(skip(self))]
    pub async fn analyze_tenant(&self, tenant_id: &str) -> anyhow::Result<Vec<String>> {
        tracing::info!("Analyzing metric cardinality in Mimir");
        let _timer = metrics::external::mimirtool_timer(ExternalCommand::AnalyzePrometheus)
            .with_label("tenant_id", tenant_id);

        let grafana_input = self.config.output_dir.join("grafana.json");
        let grafana_input = grafana_input.to_string_lossy();

        let prometheus_output = self.config.output_dir.join("prometheus-metrics.json");
        let prometheus_output = prometheus_output.to_string_lossy();

        let args = vec![
            "analyze",
            "prometheus",
            "--address",
            &self.config.mimir.querier_url,
            "--id",
            tenant_id,
            "--prometheus-http-prefix",
            "prometheus",
            "--grafana-metrics-file",
            &grafana_input,
            "--output",
            &prometheus_output,
        ];

        match Command::new("mimirtool").args(args).output().await {
            Ok(output) => {
                if !output.status.success() {
                    metrics::external::record_mimirtool_execution(
                        ExternalCommand::AnalyzePrometheus,
                        metrics::Status::Failure,
                    );

                    let stderr = String::from_utf8_lossy(&output.stderr);

                    return Err(anyhow::anyhow!(
                        "Mimirtool command failed: {}",
                        stderr.trim()
                    ));
                }
            }
            Err(e) => {
                metrics::external::record_mimirtool_execution(
                    ExternalCommand::AnalyzePrometheus,
                    metrics::Status::Failure,
                );

                return Err(anyhow::anyhow!("Failed to execute mimirtool: {}", e));
            }
        };

        let content =
            std::fs::read_to_string(self.config.output_dir.join("prometheus-metrics.json"))?;

        let data: serde_json::Value = serde_json::from_str(&content)?;

        let metrics = data["in_use_metric_counts"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("in_use_metric_counts field not found or not an array"))?
            .iter()
            .filter_map(|entry| entry["metric"].as_str().map(String::from))
            .collect();

        metrics::external::record_mimirtool_execution(
            ExternalCommand::AnalyzePrometheus,
            metrics::Status::Success,
        );

        Ok(metrics)
    }

    /// Gets the top 100 metrics by cardinality for a tenant
    pub async fn get_tenant_top_metrics(&self, tenant_id: &str) -> anyhow::Result<Vec<String>> {
        let url = format!(
            "{}/prometheus/api/v1/cardinality/label_values?label_names[]=__name__&limit=100",
            self.config.mimir.querier_url
        );

        let _timer = metrics::external::external_request_timer(Target::Querier)
            .with_label("tenant_id", tenant_id);

        let resp = self
            .client
            .get(&url)
            .header("X-Scope-OrgID", tenant_id)
            .send()
            .await?;

        if !resp.status().is_success() {
            metrics::external::record_external_request_failure(Target::Querier);

            return Err(anyhow::anyhow!(
                "Failed to fetch tenant metrics: HTTP {}",
                resp.status()
            ));
        }

        let json = resp.json::<cardinality::Response>().await?;

        let metrics: Vec<String> = json
            .labels
            .into_iter()
            .flat_map(|label| label.cardinality.into_iter().map(|card| card.label_value))
            .collect();

        Ok(metrics)
    }
}
