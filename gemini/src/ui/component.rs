use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::ListItem,
};

use crate::model::ChatMessage;

use crate::model::Sender::{Bot, Split, User};

impl From<&ChatMessage> for ListItem<'_> {
    fn from(value: &ChatMessage) -> Self {
        let lines = match value.sender {
            User => {
                let message_lines = value.message.lines();
                let mut lines = Vec::new();
                // 用户发送标记以及是否成功标记
                let line = if value.success {
                    Line::from("✅👤").alignment(Alignment::Right)
                } else {
                    Line::from("❌👤").alignment(Alignment::Right)
                };
                lines.push(line);
                // 用户发送的消息
                for line in message_lines {
                    lines.push(
                        Line::from(line.to_owned())
                            .alignment(Alignment::Right)
                            .style(Style::default().fg(Color::Green)),
                    );
                }
                // 消息发送时间
                lines.push(
                    Line::from(value.date_time.format("%H:%M:%S").to_string())
                        .alignment(Alignment::Right)
                        .style(Style::default().fg(Color::Cyan)),
                );
                lines
            }
            Bot => {
                let message_lines = value.message.lines();
                let mut lines = Vec::new();
                let line = Line::from("🤖").alignment(Alignment::Left);
                lines.push(line);
                for line in message_lines {
                    lines.push(
                        Line::from(line.to_owned())
                            .alignment(Alignment::Left)
                            .style(Style::default().fg(Color::Red)),
                    );
                }
                lines.push(
                    Line::from(value.date_time.format("%H:%M:%S").to_string())
                        .alignment(Alignment::Left)
                        .style(Style::default().fg(Color::Cyan)),
                );
                lines
            }
            Split => {
                vec![Line::from(String::new()).alignment(Alignment::Center)]
            }
        };
        ListItem::new(lines)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum AllSettingComponents {
    Model,
    Key,
    SystemInstruction,
    ResponseMineType,
    MaxOutputTokens,
    Temperature,
    TopP,
    TopK,
}

impl TryFrom<i32> for AllSettingComponents {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == AllSettingComponents::Model as i32 => Ok(AllSettingComponents::Model),
            x if x == AllSettingComponents::Key as i32 => Ok(AllSettingComponents::Key),
            x if x == AllSettingComponents::SystemInstruction as i32 => Ok(AllSettingComponents::SystemInstruction),
            x if x == AllSettingComponents::ResponseMineType as i32 => Ok(AllSettingComponents::ResponseMineType),
            x if x == AllSettingComponents::MaxOutputTokens as i32 => Ok(AllSettingComponents::MaxOutputTokens),
            x if x == AllSettingComponents::Temperature as i32 => Ok(AllSettingComponents::Temperature),
            x if x == AllSettingComponents::TopP as i32 => Ok(AllSettingComponents::TopP),
            x if x == AllSettingComponents::TopK as i32 => Ok(AllSettingComponents::TopK),
            _ => Err(()),
        }
    }
}
