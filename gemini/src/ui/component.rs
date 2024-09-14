use ratatui::{
    layout::{
        Constraint::{Fill, Length, Max},
        Layout,
    },
    style::{Color, Stylize},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::model::ChatMessage;

use crate::model::Sender::{Bot, Split, User};

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

impl Widget for ChatMessage {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.sender {
            User => {
                let [_, right] = Layout::horizontal([Max(10), Fill(1)]).areas(area);
                let [top, time_area] = Layout::vertical([Fill(1), Length(1)]).areas(right);
                // 渲染时间
                let time_paragraph = Paragraph::new(self.date_time.format(" %H:%M:%S ").to_string())
                    .style(Color::Blue)
                    .right_aligned();
                time_paragraph.render(time_area, buf);
                let [content_area, avatar_area] = Layout::horizontal([Fill(1), Length(3)]).areas(top);
                // 渲染头像
                let avatar_paragraph = Paragraph::new(if self.success { "\n👤\n✅" } else { "\n👤\n❌" })
                    .style(Color::Blue)
                    .left_aligned();
                avatar_paragraph.render(avatar_area, buf);
                // 渲染消息内容
                let message_block = if self.success {
                    Block::default().green().borders(Borders::ALL)
                } else {
                    Block::default().red().borders(Borders::ALL)
                };
                let message_paragraph = Paragraph::new(self.message).block(message_block).left_aligned();
                message_paragraph.render(content_area, buf);
            }
            Bot => {
                let [left, _] = Layout::horizontal([Fill(1), Max(10)]).areas(area);
                let [top, time_area] = Layout::vertical([Fill(1), Length(1)]).areas(left);
                // 渲染时间
                let time_paragraph = Paragraph::new(self.date_time.format(" %H:%M:%S ").to_string())
                    .style(Color::Blue)
                    .left_aligned();
                time_paragraph.render(time_area, buf);
                let [avatar_area, content_area] = Layout::horizontal([Length(3), Fill(1)]).areas(top);
                // 渲染头像
                let avatar_paragraph = Paragraph::new("\n🤖").style(Color::Blue).right_aligned();
                avatar_paragraph.render(avatar_area, buf);
                // 渲染消息内容
                let message_block = Block::default().green().borders(Borders::ALL);
                let message_paragraph = Paragraph::new(self.message).block(message_block).left_aligned();
                message_paragraph.render(content_area, buf);
            }
            Split => {
                Paragraph::new("").render(area, buf);
            }
        }
    }
}
