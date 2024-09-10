use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GenerateContentResponseError {
    pub error: Error,
}

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub code: i16,
    pub message: String,
    pub status: String,
    pub details: Option<Vec<Detail>>,
}

#[derive(Serialize, Deserialize)]
pub struct Detail {
    #[serde(rename = "@type")]
    pub type0: String,
    pub reason: String,
    pub domain: String,
    pub metadate: Metadata,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub service: String,
}
