use chrono::{DateTime, Local};

/// 单条聊天消息
///
/// 包含消息状态、消息内容、发送者、发送时间等信息
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// 消息状态，true表示已发送成功，false表示发送失败
    pub success: bool,
    /// 消息内容
    pub message: String,
    /// 发送者
    pub sender: Sender,
    /// 发送时间
    pub date_time: DateTime<Local>,
}

/// 发送者类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sender {
    /// 用户发送的消息, 第一个元组参数为图片路径
    User(String),
    /// AI 回复的消息
    Bot,
    /// 处理其他类型的消息，一般不会用到，用作标记作用
    Never,
}
