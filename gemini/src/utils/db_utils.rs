#![allow(unused)]
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    env,
    rc::Rc,
};

use nanoid::nanoid;

use anyhow::{bail, Result};
use rusqlite::Connection;
use std::cell::LazyCell;

use crate::model::{
    db::{Conversation, ImageRecord, MessageRecord},
    view::ChatMessage,
};

/// 数据库连接
#[allow(clippy::declare_interior_mutable_const)]
const DB_CONNECTION: LazyCell<Connection> = LazyCell::new(|| {
    let exe_path = env::current_exe().unwrap();
    let db_path = exe_path.parent().unwrap().join("gemini.db");
    Connection::open(db_path).unwrap()
});

/// 创建表结构
pub fn create_table() -> Result<()> {
    let sql_file = include_str!("../../migrations/20240229_create.sql");
    let mut binding = DB_CONNECTION;
    let conn = binding.borrow_mut();
    let _ = conn.execute(sql_file, ())?;
    Ok(())
}

/// 查询所有会话
pub fn query_all() -> Result<Vec<Conversation>> {
    let mut binding = DB_CONNECTION;
    let conn = binding.borrow();
    let mut stmt = conn.prepare(
        r#"SELECT conversation_id, conversation_title, conversation_start_time, conversation_modify_time
        FROM gemini_conversation"#,
    )?;
    let mut rows = stmt.query_map([], |row| {
        Ok(Conversation {
            conversation_id: row.get(0)?,
            conversation_title: row.get(1)?,
            conversation_start_time: row.get(2)?,
            conversation_modify_time: row.get(3)?,
            conversation_records: vec![],
        })
    })?;
    let mut conversations = Vec::new();
    while let Some(Ok(e)) = rows.next() {
        conversations.push(e);
    }
    Ok(conversations)
}

/// 根据会话ID查询会话详情
pub fn query_detail_by_id(conversation: Conversation) -> Result<Conversation> {
    let mut binding = DB_CONNECTION;
    let conn = binding.borrow();
    let mut stmt = conn.prepare(
        r#"SELECT
        record_id, record_content, record_time, record_sender, sort_index,
        image_record_id, image_path, image_type, image_base64
        FROM gemini_message_record LEFT JOIN gemini_image_record
        ON gemini_message_record.record_id = gemini_image_record.record_id
        WHERE conversation_id = ?1
        ORDER BY sort_index ASC"#,
    )?;
    let mut rows = stmt.query_map([conversation.conversation_id.clone()], |row| {
        let image_record_id: Option<String> = row.get(5)?;
        let image_record = if let Some(image_record_id) = image_record_id.clone() {
            Some(ImageRecord {
                image_record_id,
                image_path: row.get(6)?,
                image_type: row.get(7)?,
                image_base64: row.get(8)?,
            })
        } else {
            None
        };
        Ok(MessageRecord {
            conversation_id: conversation.conversation_id.clone(),
            record_id: row.get(0)?,
            record_content: row.get(1)?,
            record_time: row.get(2)?,
            record_sender: row.get(3)?,
            sort_index: row.get(4)?,
            image_record,
        })
    })?;

    let mut conversation_records = Vec::new();
    while let Some(Ok(record)) = rows.next() {
        conversation_records.push(record);
    }

    Ok(Conversation {
        conversation_records,
        ..conversation
    })
}

/// 根据对话 ID 删除一个对话
pub fn delete_one(conversation_id: String) -> Result<()> {
    todo!()
}

pub fn insert_one_into(chat_message: ChatMessage, chat_id: i32) {
    todo!()
}

pub fn generate_unique_id() -> String {
    nanoid!(10)
}
