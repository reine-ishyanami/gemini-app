use anyhow::{bail, Result};
use reqwest::blocking::Client;
use serde_json::{self, Value};

use crate::body::{
	request::{GeminiRequestBody, GenerationConfig},
	response::GeminiResponseBody,
	Content, Part, Role,
};

use super::LanguageModel;

#[derive(Clone)]
pub struct Gemini {
	pub key: String,
	pub url: String,
	pub contents: Vec<Content>,
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
	pub fn rebuild(key: String, url: String, contents: Vec<Content>, options: GenerationConfig) -> Self {
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

	/// 同步单次对话
	pub fn chat_once(&self, content: String) -> Result<String> {
		// 创建一个客户端实例
		let url = format!("{}?key={}", self.url, self.key);
		let body = GeminiRequestBody {
			contents: vec![Content {
				role: Some(Role::User),
				parts: vec![Part::Text(content)],
			}],
			generation_config: Some(self.options.clone()),
			..Default::default()
		};
		let body_json = serde_json::to_string(&body)?;
		// 发送 GET 请求，并添加自定义头部
		let response = self
			.client
			.post(url)
			.header("Content-Type", "application/json")
			.body(body_json)
			.send()?;
		if response.status().is_success() {
			let response_text = response.text()?;
			// 解析响应内容
			let response_json: GeminiResponseBody = serde_json::from_str(&response_text)?;
			match response_json.candidates[0].content.parts[0].clone() {
				Part::Text(s) => Ok(s),
				_ => bail!("Unexpected response format"),
			}
		} else {
			let response_text = response.text()?;
			// 解析响应内容
			let response_json: Value = serde_json::from_str(&response_text)?;
			let error_message = response_json["error"]["message"].as_str().unwrap().to_owned();
			bail!(error_message)
		}
	}

	/// 同步连续对话
	pub fn chat_conversation(&mut self, content: String) -> Result<String> {
		self.contents.push(Content {
			role: Some(Role::User),
			parts: vec![Part::Text(content)],
		});
		let cloned_contents = self.contents.clone();
		let url = format!("{}?key={}", self.url, self.key);
		let body = GeminiRequestBody {
			contents: cloned_contents,
			generation_config: Some(self.options.clone()),
			..Default::default()
		};
		let body_json = serde_json::to_string(&body)?;
		// 发送 GET 请求，并添加自定义头部
		let response = self
			.client
			.post(url)
			.header("Content-Type", "application/json")
			.body(body_json)
			.send()?;

		if response.status().is_success() {
			let response_text = response.text()?;
			// 解析响应内容
			let response_json: GeminiResponseBody = serde_json::from_str(&response_text)?;
			match response_json.candidates[0].content.parts[0].clone().clone() {
				Part::Text(s) => {
					self.contents.push(Content {
						role: Some(Role::Model),
						parts: vec![Part::Text(s.clone())],
					});
					Ok(s)
				}
				_ => bail!("Unexpected response format"),
			}
		} else {
			// 如果响应失败，则移除最后发送的那次用户请求
			self.contents.pop();
			let response_text = response.text()?;
			// 解析错误响应内容
			let response_json: Value = serde_json::from_str(&response_text)?;
			let error_message = response_json["error"]["message"].as_str().unwrap().to_owned();
			bail!(error_message)
		}
	}
}

#[cfg(test)]
mod test {
	use std::env;

	use super::*;

	#[test]
	fn test_chat_once() {
		let key = env::var("GEMINI_KEY");
		assert!(key.is_ok());
		let client = Gemini::new(key.unwrap(), LanguageModel::Gemini1_5Flash);
		let req1 = "My Name is Reine".to_owned();
		let resp1 = client.chat_once(req1.clone());
		assert!(resp1.is_ok());
		println!("{}: {}", req1, resp1.unwrap());
	}

	#[test]
	fn test_chat_conversation() {
		let key = env::var("GEMINI_KEY");
		assert!(key.is_ok());
		let mut client = Gemini::new(key.unwrap(), LanguageModel::Gemini1_5Flash);
		let req1 = "My Name is Reine".to_owned();
		let resp1 = client.chat_conversation(req1.clone());
		assert!(resp1.is_ok());
		println!("{}: {}", req1, resp1.unwrap());
		let req2 = "Who am I".to_owned();
		let resp2 = client.chat_conversation(req2.clone());
		assert!(resp2.is_ok());
		println!("{}: {}", req2, resp2.unwrap());
	}
}
