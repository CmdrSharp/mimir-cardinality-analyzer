use crate::{
    config::Grafana as GrafanaConfig,
    grafana::{alert::Alert, datasource::Datasource},
};

pub mod alert;
pub mod datasource;

pub struct Grafana {
    config: GrafanaConfig,
    client: reqwest::Client,
}

impl Grafana {
    /// Create a new Grafana instance
    pub fn new(config: GrafanaConfig) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.insecure)
            .build()?;

        Ok(Self { config, client })
    }

    /// Get datasources from Grafana
    #[tracing::instrument(skip(self))]
    pub async fn get_datasources(&self) -> anyhow::Result<Vec<Datasource>> {
        tracing::info!("Fetching datasources from Grafana");

        let response = self
            .client
            .get(format!("{}/api/datasources", self.config.url))
            .bearer_auth(self.config.token.clone())
            .send()
            .await?
            .json::<Vec<Datasource>>()
            .await?;

        Ok(response)
    }

    /// Get alert rules from Grafana
    #[tracing::instrument(skip(self))]
    pub async fn get_alert_rules(&self) -> anyhow::Result<Vec<Alert>> {
        tracing::info!("Fetching alert rules from Grafana");

        let response = self
            .client
            .get(format!(
                "{}/api/v1/provisioning/alert-rules",
                self.config.url
            ))
            .bearer_auth(self.config.token.clone())
            .send()
            .await?;

        let body = response.text().await?;
        let alerts: Vec<Alert> = serde_json::from_str(&body)?;

        let alerts = alerts
            .into_iter()
            .filter_map(|mut alert| {
                // Keep only AlertData that have a model with a set expr
                alert.data.retain(|d| d.model.expr.is_some());

                // Drop the entire alert if it has no data left
                if alert.data.is_empty() {
                    None
                } else {
                    Some(alert)
                }
            })
            .collect();

        Ok(alerts)
    }

    /// Iterate over alert rules in Grafana and find a metric by name in alerts that use tenant datasources
    #[tracing::instrument(skip(self, alerts, datasources))]
    pub fn find_metric_in_alerts(
        &self,
        tenant: &str,
        alerts: &Vec<Alert>,
        datasources: &[Datasource],
        metric_name: &str,
    ) -> anyhow::Result<bool> {
        let metric_regex = regex::Regex::new(&format!(r"\b{}\b", regex::escape(metric_name)))?;

        for alert in alerts {
            if self.alert_contains_metric(alert, &metric_regex, datasources, tenant) {
                tracing::info!(
                    "Metric '{}' found in alert '{}' for tenant '{}'",
                    metric_name,
                    alert.title,
                    tenant
                );

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if an alert contains a metric matching the given regex and uses a tenant datasource
    #[tracing::instrument(skip_all)]
    fn alert_contains_metric(
        &self,
        alert: &Alert,
        metric_regex: &regex::Regex,
        datasources: &[Datasource],
        tenant: &str,
    ) -> bool {
        let has_metric = alert.data.iter().any(|alert_data| {
            alert_data
                .model
                .expr
                .as_ref()
                .map(|expr| metric_regex.is_match(expr))
                .unwrap_or(false)
        });

        let uses_tenant_datasource = alert.data.iter().any(|alert_data| {
            alert_data
                .datasource_uid
                .as_ref()
                .and_then(|uid| datasources.iter().find(|ds| &ds.uid == uid))
                .map(|ds| ds.name.contains(tenant))
                .unwrap_or(false)
        });

        has_metric && uses_tenant_datasource
    }
}
