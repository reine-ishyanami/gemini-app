use serde::{Deserialize, Serialize};

use super::Content;

#[derive(Serialize, Deserialize)]
pub struct GeminiResponseBody {
    pub candidates: Vec<ChatResponse>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: UsageMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: Content,
    #[serde(rename = "finishReason")]
    pub finish_reason: String,
    pub index: isize,
    #[serde(rename = "safetyRatings")]
    pub safety_ratings: Vec<SafetyRating>,
}

#[derive(Serialize, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

#[derive(Serialize, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: isize,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: isize,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: isize,
}
