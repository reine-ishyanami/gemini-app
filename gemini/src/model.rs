#![allow(dead_code)]

use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub message: String,
    pub sender: Sender,
    pub date_time: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub enum Sender {
    User,
    Bot,
}
