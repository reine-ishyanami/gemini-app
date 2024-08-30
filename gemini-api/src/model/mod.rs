#[cfg(feature = "blocking")]
pub mod blocking;

use std::fmt;

use anyhow::Result;
use reqwest::Client;
use serde_json;

use crate::body::{GeminiRequestBody, GeminiResponseBody, GenerationConfig, Paragraph, Part, Role};

pub enum LanguageModel {
    Gemini1_0Pro,
    Gemini1_5Pro,
    Gemini1_5Flash,
}

impl fmt::Display for LanguageModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageModel::Gemini1_0Pro => write!(f, "gemini-1.0-pro"),
            LanguageModel::Gemini1_5Pro => write!(f, "gemini-1.5-pro"),
            LanguageModel::Gemini1_5Flash => write!(f, "gemini-1.5-flash"),
        }
    }
}

pub struct Gemini {
    pub key: String,
    pub url: String,
    pub contents: Vec<Paragraph>,
    client: Client,
    pub options: GenerationConfig,
}

impl Gemini {
    const GEMINI_API_URL: &'static str = "https://generativelanguage.googleapis.com/v1beta/models/";

    /// 创建新实例
    pub fn new(key: String, model: LanguageModel) -> Self {
        let client = Client::new();
        let contents = Vec::new();
        let url = format!("{}{}:generateContent", Self::GEMINI_API_URL, model);
        Self {
            key,
            url,
            contents,
            client,
            options: GenerationConfig::default(),
        }
    }

    /// 重建实例
    pub fn rebuild(key: String, url: String, contents: Vec<Paragraph>, options: GenerationConfig) -> Self {
        let client = Client::new();
        Self {
            key,
            url,
            contents,
            client,
            options,
        }
    }

    /// 参数配置
    pub fn set_options(&mut self, options: GenerationConfig) {
        self.options = options;
    }

    /// 异步单次对话
    pub async fn chat_once(&self, content: String) -> Result<String> {
        // 创建一个客户端实例
        let url = format!("{}?key={}", self.url, self.key);
        let body = GeminiRequestBody {
            contents: vec![Paragraph {
                role: Role::User,
                parts: vec![Part { text: content }],
            }],
            generationConfig: self.options.clone(),
        };
        let body_json = serde_json::to_string(&body)?;
        // 发送 GET 请求，并添加自定义头部
        let response = self
            .client
            .post(url)
            .header("Content-Type", "Application/json")
            .body(body_json)
            .send()
            .await?;
        let response_text = response.text().await?;
        // 解析响应内容
        let response_json: GeminiResponseBody = serde_json::from_str(&response_text)?;

        let response_text = response_json.candidates[0].content.parts[0].text.clone();
        Ok(response_text)
    }

    /// 异步连续对话
    pub async fn chat_conversation(&mut self, content: String) -> Result<String> {
        self.contents.push(Paragraph {
            role: Role::User,
            parts: vec![Part { text: content }],
        });
        let cloned_contents = self.contents.clone();
        let url = format!("{}?key={}", self.url, self.key);
        let body = GeminiRequestBody {
            contents: cloned_contents,
            generationConfig: self.options.clone(),
        };
        let body_json = serde_json::to_string(&body)?;
        // 发送 GET 请求，并添加自定义头部
        let response = self
            .client
            .post(url)
            .header("Content-Type", "Application/json")
            .body(body_json)
            .send()
            .await?;
        let response_text = response.text().await?;
        // 解析响应内容
        let response_json: GeminiResponseBody = serde_json::from_str(&response_text)?;

        let response_text = response_json.candidates[0].content.parts[0].text.clone();
        self.contents.push(Paragraph {
            role: Role::Model,
            parts: vec![Part {
                text: response_text.clone(),
            }],
        });
        Ok(response_text)
    }
}
