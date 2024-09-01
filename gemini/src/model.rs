use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub success: bool,
    pub message: String,
    pub sender: Sender,
    pub date_time: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub enum Sender {
    User,
    Bot,
    Split,
}
