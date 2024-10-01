#![allow(unused)]

use chrono::{DateTime, Local};

use super::view::Sender;

/// 一个聊天记录项
#[derive(Debug, Clone)]
pub struct Conversation {
    /// conversation id
    pub conversation_id: String,
    /// 聊天标题
    pub conversation_title: String,
    /// 聊天开始时间
    pub conversation_start_time: DateTime<Local>,
    /// 最后一条聊天记录发送时间
    pub conversation_modify_time: DateTime<Local>,
    /// 聊天记录
    pub conversation_records: Vec<MessageRecord>,
}

/// 单条聊天记录
#[derive(Debug, Clone)]
pub struct MessageRecord {
    /// 记录 id
    pub record_id: String,
    /// 对应的 conversation id
    pub conversation_id: String,
    /// 记录内容
    pub record_content: String,
    /// 记录发送时间
    pub record_time: DateTime<Local>,
    /// 记录发送者
    pub record_sender: Sender,
    /// 排序
    pub sort_index: u8,
    /// 图片内容
    pub image_record: Option<ImageRecord>,
}

/// 单条聊天记录携带的图片
#[derive(Debug, Clone)]
pub struct ImageRecord {
    /// 图片记录 id
    pub image_record_id: String,
    /// 对应的 record id
    pub record_id: String,
    /// 图片路径
    pub image_path: String,
    /// 图片类型
    pub image_type: String,
}
