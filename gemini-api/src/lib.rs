pub mod body;
pub mod model;
pub mod param;

use anyhow::{bail, Result};
use body::{
    request::{GeminiRequestBody, GenerationConfig},
    response::GenerateContentResponse,
    Content, Part, Role,
};
use reqwest::Client;

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent";

pub async fn chat_once(content: String, key: String) -> Result<String> {
    // 创建一个客户端实例
    let client = Client::new();
    let url = format!("{}?key={}", GEMINI_API_URL, key);
    let body = GeminiRequestBody {
        contents: vec![Content {
            role: Some(Role::User),
            parts: vec![Part::Text(content)],
        }],
        generation_config: Some(GenerationConfig::default()),
        ..Default::default()
    };
    let body_json = serde_json::to_string(&body)?;
    // 发送 GET 请求，并添加自定义头部
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body_json)
        .send()
        .await?;
    let response_text = response.text().await?;
    // 解析响应内容
    let response_json: GenerateContentResponse = serde_json::from_str(&response_text)?;
    match response_json.candidates[0].content.parts[0].clone().clone() {
        Part::Text(s) => Ok(s),
        _ => bail!("Unexpected response format"),
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn convert_to_json() -> Result<()> {
        let body = GeminiRequestBody {
            contents: vec![Content {
                role: Some(Role::User),
                parts: vec![Part::Text("Hello, world!".to_owned())],
            }],
            generation_config: Some(GenerationConfig::default()),
            ..Default::default()
        };
        let body_json = serde_json::to_string(&body)?;
        assert_eq!(
            body_json,
            r#"{"contents":[{"parts":[{"text":"Hello, world!"}],"role":"user"}],"generationConfig":{"responseMimeType":"text/plain","maxOutputTokens":8192,"temperature":1.0,"topP":0.95,"topK":64}}"#
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_chat() -> Result<()> {
        let key = env::var("GEMINI_KEY");
        assert!(key.is_ok());
        let content = "你好".to_owned();
        let response = chat_once(content, key.unwrap()).await?;
        assert!(!response.is_empty());
        println!("Response: {}", response);
        Ok(())
    }

    #[test]
    fn test_enum_serialize() {
        #[derive(Serialize, Deserialize)]
        enum Message {
            Quit,
            Move { x: i32, y: i32 },
            Write(String),
        }
        {
            let msg = Message::Quit;
            let serialized = serde_json::to_string(&msg).unwrap();
            assert_eq!(serialized, r#""Quit""#);
        }
        {
            let msg = Message::Move { x: 10, y: 20 };
            let serialized = serde_json::to_string(&msg).unwrap();
            assert_eq!(serialized, r#"{"Move":{"x":10,"y":20}}"#);
        }
        {
            let msg = Message::Write("HelloWorld".to_owned());
            let serialized = serde_json::to_string(&msg).unwrap();
            assert_eq!(serialized, r#"{"Write":"HelloWorld"}"#);
        }
    }
}
