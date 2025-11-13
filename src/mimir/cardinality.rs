use serde::Deserialize;

#[derive(Deserialize)]
pub struct Response {
    pub labels: Vec<Label>,
}

#[derive(Deserialize)]
pub struct Label {
    pub cardinality: Vec<Cardinality>,
}

#[derive(Deserialize)]
pub struct Cardinality {
    pub label_value: String,
    pub series_count: usize,
}
