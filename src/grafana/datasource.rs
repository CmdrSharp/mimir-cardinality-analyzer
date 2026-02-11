use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Datasource {
    pub id: usize,
    pub uid: String,
    pub name: String,
}
