use std::{
    borrow::{Borrow, BorrowMut},
    env,
};

use nanoid::nanoid;

use anyhow::Result;
use rusqlite::Connection;
use std::cell::LazyCell;

use crate::model::{
    db::{Conversation, ImageRecord, MessageRecord},
    view::{ChatMessage, Sender},
};

use super::image_utils::{cache_image, delete_image_cache};

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
    conn.execute_batch(sql_file)?;
    Ok(())
}

/// 查询所有会话
pub fn query_all() -> Result<Vec<Conversation>> {
    let binding = DB_CONNECTION;
    let conn = binding.borrow();
    let mut stmt = conn.prepare(
        r#"SELECT conversation_id, conversation_title, conversation_start_time, conversation_modify_time
        FROM gemini_conversation ORDER BY conversation_modify_time DESC"#,
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
    let binding = DB_CONNECTION;
    let conn = binding.borrow();
    let mut stmt = conn.prepare(
        r#"SELECT
        gemini_message_record.record_id, record_content, record_time, record_sender, sort_index,
        image_record_id, image_path, image_type
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
                record_id: row.get(0)?,
                image_path: row.get(6)?,
                image_type: row.get(7)?,
            })
        } else {
            None
        };
        let sender_str: String = row.get(3)?;
        let image_path: Option<String> = row.get(6)?;
        let record_sender = match sender_str.as_str() {
            "User" => Sender::User(image_path.unwrap_or_default()),
            "Bot" => Sender::Bot,
            _ => Sender::Never,
        };
        Ok(MessageRecord {
            conversation_id: conversation.conversation_id.clone(),
            record_id: row.get(0)?,
            record_content: row.get(1)?,
            record_time: row.get(2)?,
            record_sender,
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
pub fn delete_one(conversation: Conversation) -> Result<()> {
    let binding = DB_CONNECTION;
    let conn = binding.borrow();
    // 删除图片记录
    conversation
        .conversation_records
        .iter()
        .filter(|record| record.image_record.is_some())
        .map(|record| record.image_record.clone())
        .for_each(|record| {
            let image_record_id = record.unwrap().image_record_id;
            let _ = delete_image_cache(image_record_id);
        });
    // 删除表
    let sql = format!(
        r#"
    PRAGMA foreign_keys = ON;
    DELETE FROM gemini_conversation WHERE conversation_id = '{}';
    PRAGMA foreign_keys = OFF;
    "#,
        conversation.conversation_id
    );
    conn.execute_batch(sql.as_str())?;
    Ok(())
}

/// 保存对话
pub fn save_conversation(conversation_id: String, conversation_title: String, message: ChatMessage) -> Result<()> {
    let binding = DB_CONNECTION;
    let conn = binding.borrow();
    // 查询是否存在此会话
    let mut stmt = conn.prepare(
        r#"
        SELECT conversation_id FROM main.gemini_conversation WHERE conversation_id = ?1
        "#,
    )?;
    let exists = stmt
        .query_row([conversation_id.clone()], |row| {
            let conversation_id: String = row.get(0)?;
            Ok(!conversation_id.is_empty())
        })
        .unwrap_or_default();

    if !exists {
        // 如果不存在，则新增一个会话
        let date_time = message.date_time;
        let _ = conn.execute(r#"
        INSERT INTO gemini_conversation (conversation_id, conversation_title, conversation_start_time, conversation_modify_time)
        VALUES (?1, ?2, ?3, ?4)
        "#, [conversation_id.clone(), conversation_title.clone(), date_time.clone().to_string(), date_time.to_string()])?;
    } else {
        // 如果存在，则更新会话修改时间
        let date_time = message.date_time;
        let _ = conn.execute(
            r#"
        UPDATE gemini_conversation SET conversation_modify_time = ?1
        WHERE conversation_id = ?2
        "#,
            [date_time.to_string(), conversation_id.clone()],
        );
    }

    // 获取当前会话 ID 的最新消息序号 + 1
    let mut stmt = conn.prepare(
        r#"
    SELECT MAX(sort_index) FROM gemini_message_record WHERE conversation_id = ?1
    "#,
    )?;
    let sort_index = stmt
        .query_row([conversation_id.clone()], |row| {
            let sort_index: Option<i32> = row.get(0)?;
            Ok(sort_index.unwrap_or_default())
        })
        .map_or(0, |index| index + 1);

    // 新增一条消息到对应会话
    match message.sender {
        crate::model::view::Sender::User(image_url) => {
            let record_id = generate_unique_id();
            let conversation_id = conversation_id.clone();
            let record_content = message.message.clone();
            let record_time = message.date_time;
            let record_sender = "User".to_string();
            conn.execute(r#"
                INSERT INTO gemini_message_record (record_id, conversation_id, record_content, record_time, record_sender, sort_index)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#, [record_id.clone(), conversation_id, record_content.to_string(), record_time.to_string(), record_sender, sort_index.to_string()])?;
            // 如果图片路径不为空，则插入图片记录
            if !image_url.is_empty() {
                let image_record_id = generate_unique_id();
                let image_path = image_url.clone();
                // 写入文件
                cache_image(image_url, image_record_id.clone())?;
                // 压缩后的图片格式
                let image_type = "image/jpeg".into();
                conn.execute(
                    r#"
                    INSERT INTO gemini_image_record (image_record_id, record_id, image_path, image_type)
                    VALUES (?1, ?2, ?3, ?4)
                "#,
                    [image_record_id, record_id, image_path, image_type],
                )?;
            }
        }
        crate::model::view::Sender::Bot => {
            let record_id = generate_unique_id();
            let conversation_id = conversation_id.clone();
            let record_content = message.message.clone();
            let record_time = message.date_time;
            let record_sender = "Bot".to_string();
            conn.execute(r#"
            INSERT INTO gemini_message_record (record_id, conversation_id, record_content, record_time, record_sender, sort_index)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#, [record_id, conversation_id, record_content.to_string(), record_time.to_string(), record_sender, sort_index.to_string()])?;
        }
        crate::model::view::Sender::Never => {}
    }

    Ok(())
}

/// 修改会话标题
pub fn modify_title(conversation_id: String, conversation_title: String) -> Result<()> {
    let binding = DB_CONNECTION;
    let conn = binding.borrow();
    let _ = conn.execute(
        r#"
        UPDATE gemini_conversation SET conversation_title = ?1
        WHERE conversation_id = ?2
        "#,
        [conversation_title.clone(), conversation_id.clone()],
    )?;
    Ok(())
}

/// 生成唯一 ID
pub fn generate_unique_id() -> String {
    nanoid!(10)
}
