#![allow(unused)]

use chrono::{DateTime, Local};

use super::view::Sender;

/// 一个聊天记录项
pub struct ChatItem {
    /// chat id
    pub chat_id: i32,
    /// 聊天标题
    pub chat_title: String,
    /// 聊天开始时间
    pub chat_start_time: DateTime<Local>,
    /// 最后一条聊天记录发送时间
    pub chat_modify_time: DateTime<Local>,
    /// 聊天记录
    pub chat_records: Vec<SingleChatRecord>,
}

/// 单条聊天记录
pub struct SingleChatRecord {
    /// 对应的 chat id
    pub chat_id: i32,
    /// 记录 id
    pub record_id: i32,
    /// 记录内容
    pub record_content: String,
    /// 记录发送时间
    pub record_time: DateTime<Local>,
    /// 记录发送者
    pub record_sender: Sender,
}
