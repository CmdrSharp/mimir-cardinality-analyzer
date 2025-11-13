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
    pub model: Vec<AlertDataModel>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlertDataModel {
    pub expr: Option<String>,
}
