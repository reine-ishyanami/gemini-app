use std::cmp::max;

use ratatui::{
    layout::{
        Constraint::{Fill, Length, Max},
        Flex, Layout,
    },
    style::{Color, Stylize},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::{model::view::ChatMessage, utils::char_utils::s_length};

use crate::model::view::Sender::{Bot, Split, User};

impl Widget for ChatMessage {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.sender {
            User(image_path) => {
                // 拿到所有消息中最长一行的宽度
                let x = self
                    .message
                    .clone()
                    .lines()
                    .map(Into::into)
                    .map(s_length)
                    .max()
                    .unwrap_or_default();
                // 标题
                let title = if image_path.is_empty() {
                    "Simple".into()
                } else {
                    format!("Image {}", image_path)
                };
                // 拿到最大宽度
                let width = max(x, s_length(title.clone())) as u16;
                // 魔法数 5 为左右边框宽度 1 + 1 加上头像区域宽度 3
                // 此处之所以和 21 比较，因为21是时间显示区域的宽度，不得少于这个宽度，否则时间显示会有问题
                let [right] = Layout::horizontal([Max(max(width + 5, 21))])
                    .flex(Flex::End)
                    .areas(area);
                let [top, time_area] = Layout::vertical([Fill(1), Length(1)]).areas(right);
                // 渲染时间
                let time_paragraph = Paragraph::new(self.date_time.format(" %Y/%m/%d %H:%M:%S ").to_string())
                    .style(Color::Blue)
                    .right_aligned();
                time_paragraph.render(time_area, buf);
                let [content_area, avatar_area] = Layout::horizontal([Max(width + 2), Length(3)])
                    .flex(Flex::End)
                    .areas(top);
                // 渲染头像
                let avatar_paragraph = Paragraph::new("\n👤").left_aligned();
                avatar_paragraph.render(avatar_area, buf);
                // 渲染消息内容
                let message_block = if self.success {
                    Block::default().title(title).green().borders(Borders::ALL)
                } else {
                    Block::default().title(title).red().borders(Borders::ALL)
                };
                let message_paragraph = Paragraph::new(self.message)
                    .wrap(Wrap { trim: false })
                    .style(Color::Cyan)
                    .block(message_block)
                    .left_aligned();
                message_paragraph.render(content_area, buf);
            }
            Bot => {
                // 拿到所有消息中最长一行的宽度
                let width = self
                    .message
                    .clone()
                    .lines()
                    .map(Into::into)
                    .map(s_length)
                    .max()
                    .unwrap_or_default() as u16;
                // 魔法数 5 为左右边框宽度 1 + 1 加上头像区域宽度 3
                let [left] = Layout::horizontal([Max(width + 5)]).flex(Flex::Start).areas(area);
                let [top, time_area] = Layout::vertical([Fill(1), Length(1)]).areas(left);
                // 渲染时间
                let time_paragraph = Paragraph::new(self.date_time.format(" %Y/%m/%d %H:%M:%S ").to_string())
                    .style(Color::Blue)
                    .left_aligned();
                time_paragraph.render(time_area, buf);
                let [avatar_area, content_area] = Layout::horizontal([Length(3), Max(width + 2)])
                    .flex(Flex::Start)
                    .areas(top);
                // 渲染头像
                let avatar_paragraph = Paragraph::new("\n🤖").right_aligned();
                avatar_paragraph.render(avatar_area, buf);
                // 渲染消息内容
                let message_block = Block::default().green().borders(Borders::ALL);
                let message_paragraph = Paragraph::new(self.message)
                    .wrap(Wrap { trim: false })
                    .style(Color::Yellow)
                    .block(message_block)
                    .left_aligned();
                message_paragraph.render(content_area, buf);
            }
            Split => {
                Paragraph::new("").render(area, buf);
            }
        }
    }
}
