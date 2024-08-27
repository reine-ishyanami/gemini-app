pub mod body;
pub mod model;

use anyhow::Result;
use body::{GeminiRequestBody, GeminiResponseBody, GenerationConfig, Paragraph, Part, Role};
use reqwest::Client;
use serde_json;

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent";

pub async fn chat_once(content: String, key: String) -> Result<String> {
    // 创建一个客户端实例
    let client = Client::new();
    let url = format!("{}?key={}", GEMINI_API_URL, key);
    let body = GeminiRequestBody {
        contents: vec![Paragraph {
            role: Role::User,
            parts: vec![Part { text: content }],
        }],
        generationConfig: GenerationConfig::default(),
    };
    let body_json = serde_json::to_string(&body)?;
    // 发送 GET 请求，并添加自定义头部
    let response = client
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

#[cfg(test)]
mod tests {
    use model::{Gemini, LanguageModel};
    use std::env;

    use super::*;

    #[test]
    fn convert_to_json() -> Result<()> {
        let body = GeminiRequestBody {
            contents: vec![Paragraph {
                role: Role::User,
                parts: vec![Part {
                    text: "Hello, world!".to_owned(),
                }],
            }],
            generationConfig: GenerationConfig::default(),
        };
        let body_json = serde_json::to_string(&body)?;
        assert_eq!(
            body_json,
            r#"{"contents":[{"role":"user","parts":[{"text":"Hello, world!"}]}],"generationConfig":{"temperature":1,"topK":64,"topP":0.95,"maxOutputTokens":8192,"responseMimeType":"text/plain"}}"#
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_chat() -> Result<()> {
        let key = env::var("GEMINI_KEY");
        assert!(key.is_ok());
        let content = "Write Hello World with Rust Programming Language".to_owned();
        let response = chat_once(content, key.unwrap()).await?;
        assert!(response.len() > 0);
        println!("Response: {}", response);
        Ok(())
    }

    #[tokio::test]
    async fn test_chat_conversation() -> Result<()> {
        let key = env::var("GEMINI_KEY");
        assert!(key.is_ok());
        let mut client = Gemini::new(key.unwrap(), LanguageModel::Gemini1_5Flash);
        let req1 = "My Name is Reine".to_owned();
        let resp1 = client.chat_conversation(req1.clone()).await?;
        assert!(resp1.len() > 0);
        println!("{}: {}", req1, resp1);
        let req2 = "Who am I".to_owned();
        let resp2 = client.chat_conversation(req2.clone()).await?;
        assert!(resp2.len() > 0);
        println!("{}: {}", req2, resp2);
        Ok(())
    }
}
