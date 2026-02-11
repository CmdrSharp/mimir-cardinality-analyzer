use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

impl Config {
    /// Set the output directory for intermediate files
    pub fn with_output_dir(mut self, output_dir: PathBuf) -> Self {
        self.output_dir = output_dir;
        self
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub grafana: Grafana,
    pub mimir: Mimir,
    pub http: Http,
    #[serde(skip)]
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Grafana {
    pub url: String,
    pub token: String,
    pub insecure: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Mimir {
    #[serde(rename = "storeGatewayUrl")]
    pub store_gateway_url: String,
    #[serde(rename = "querierUrl")]
    pub querier_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Http {
    pub host: String,
    pub port: u16,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        tracing::info!("Loading config from file");

        let config = std::fs::read_to_string(path)?;
        Ok(serde_norway::from_str(&config)?)
    }
}

impl Grafana {
    /// Create a new Grafana instance, resolving token from environment variable if needed
    pub fn new(
        url: String,
        token: Option<String>,
        token_from: Option<String>,
        insecure: bool,
    ) -> anyhow::Result<Self> {
        let token = if token.is_none() && token_from.is_some() {
            std::env::var(token_from.unwrap())?
        } else {
            token.unwrap_or_default()
        };

        Ok(Self {
            url,
            token,
            insecure,
        })
    }
}

impl<'de> Deserialize<'de> for Grafana {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct GrafanaRaw {
            url: String,
            token: Option<String>,
            #[serde(rename = "tokenFrom")]
            token_from: Option<String>,
            #[serde(default)]
            insecure: Option<bool>,
        }

        let raw = GrafanaRaw::deserialize(deserializer)?;
        Grafana::new(
            raw.url,
            raw.token,
            raw.token_from,
            raw.insecure.unwrap_or(false),
        )
        .map_err(serde::de::Error::custom)
    }
}
