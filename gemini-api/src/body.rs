#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GeminiRequestBody {
    pub contents: Vec<Paragraph>,
    pub generationConfig: GenerationConfig,
}

#[derive(Serialize, Deserialize)]
pub struct GeminiResponseBody {
    pub candidates: Vec<ChatResponse>,
    pub usageMetadata: UsageMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: Paragraph,
    pub finishReason: String,
    pub index: i32,
    pub safetyRatings: Vec<SafetyRating>,
}

#[derive(Serialize, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

#[derive(Serialize, Deserialize)]
pub struct UsageMetadata {
    pub promptTokenCount: i32,
    pub candidatesTokenCount: i32,
    pub totalTokenCount: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub role: Role,
    pub parts: Vec<Part>,
}

#[derive(Clone)]
pub enum Role {
    User,
    Model,
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_uppercase().as_str() {
            "USER" => Ok(Role::User),
            "MODEL" => Ok(Role::Model),
            _ => Err(serde::de::Error::custom("Invalid role")),
        }
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Role::User => serializer.serialize_str("user"),
            Role::Model => serializer.serialize_str("model"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub temperature: i32,
    pub topK: i32,
    pub topP: f32,
    pub maxOutputTokens: i32,
    pub responseMimeType: String,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 1,
            topK: 64,
            topP: 0.95,
            maxOutputTokens: 8192,
            responseMimeType: "text/plain".to_owned(),
        }
    }
}
