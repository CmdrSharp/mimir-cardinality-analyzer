use crate::{config::Grafana as GrafanaConfig, grafana::alert::Alert};

pub mod alert;

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

    /// Get alert rules from Grafana
    #[tracing::instrument(skip(self))]
    pub async fn get_alert_rules(&self) -> anyhow::Result<Vec<Alert>> {
        tracing::info!("Fetching alert rules from Grafana");

        let alerts = self
            .client
            .get(format!(
                "{}/api/v1/provisioning/alert-rules",
                self.config.url
            ))
            .bearer_auth(self.config.token.clone())
            .send()
            .await?
            .json::<Vec<Alert>>()
            .await?;

        let alerts = alerts
            .into_iter()
            .filter_map(|mut alert| {
                // Keep only models that have a set expr
                for data in &mut alert.data {
                    data.model.retain(|m| m.expr.is_some());
                }

                // Drop any AlertData that has no models left
                alert.data.retain(|d| !d.model.is_empty());

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

    /// Iterate over alert rules in Grafana and find a metric by name
    #[tracing::instrument(skip(self))]
    pub fn find_metric_in_alerts(
        &self,
        alerts: &Vec<Alert>,
        metric_name: &str,
    ) -> anyhow::Result<bool> {
        let metric_regex = regex::Regex::new(&format!(r"\b{}\b", regex::escape(metric_name)))?;

        for alert in alerts {
            if self.alert_contains_metric(alert, &metric_regex) {
                tracing::info!("Metric '{}' found in alert '{}'", metric_name, alert.title);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if an alert contains a metric matching the given regex
    fn alert_contains_metric(&self, alert: &Alert, metric_regex: &regex::Regex) -> bool {
        alert
            .data
            .iter()
            .flat_map(|data| &data.model)
            .filter_map(|model| model.expr.as_ref())
            .any(|expr| metric_regex.is_match(expr))
    }
}
