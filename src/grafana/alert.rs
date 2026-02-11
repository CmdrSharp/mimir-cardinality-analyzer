use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Alert {
    pub id: usize,
    pub uid: String,
    pub title: String,
    pub data: Vec<AlertData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlertData {
    #[serde(rename = "datasourceUid")]
    pub datasource_uid: Option<String>,
    pub model: AlertDataModel,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlertDataModel {
    #[serde(default)]
    pub expr: Option<String>,
}
