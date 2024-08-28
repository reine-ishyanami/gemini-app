#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub message: String,
    pub sender: Sender,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum Sender {
    User,
    Bot,
}
