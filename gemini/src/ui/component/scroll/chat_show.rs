use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
    Frame,
};

use crate::{model::view::ChatMessage, utils::char_utils::c_len};

use ratatui::layout::{Constraint::Length, Layout};

use crate::model::view::Sender::Never;

/// 滚动条相关属性
#[derive(Default)]
pub struct ChatShowScrollProps {
    /// 聊天历史记录
    pub chat_history: Vec<ChatMessage>,
    /// 滚动条偏移量
    pub scroll_offset: u16,
    /// 聊天历史记录区域高度
    pub chat_history_area_height: u16,
}
// 顶部底部边框以及时间区域高度
static TOP_BOTTOM_BORDER_TIME_HEIGHT: u16 = 3;

impl ChatShowScrollProps {
    pub fn draw<F>(&mut self, frame: &mut Frame, area: Rect, chat_area_width: F, is_focused: bool)
    where
        F: Fn() -> usize,
    {
        let chat_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if is_focused { Color::Green } else { Color::White }));
        let items: Vec<ChatMessage> = self
            .chat_history
            .iter()
            .map(|m| {
                let area_width = chat_area_width();
                let mut message = String::new();
                // 对长文本进行插入换行符号
                let mut line_width = 0;
                for (_, c) in m.message.clone().char_indices() {
                    // 如果当前行宽度正好为组件宽度，则插入换行符
                    if line_width == area_width {
                        message.push('\n');
                        line_width = 0;
                    }
                    // 如果当前字符宽度大于组件宽度，则在最后一个字符之前插入换行符插入换行符
                    if line_width > area_width {
                        let c = message.pop().unwrap();
                        message.push('\n');
                        message.push(c);
                        line_width = c_len(c);
                    }
                    message.push(c);
                    line_width += c_len(c);
                    if c == '\n' {
                        line_width = 0;
                    }
                }
                ChatMessage { message, ..m.clone() }
            })
            .collect();
        // 计算当前聊天记录区域高度
        self.chat_history_area_height = items
            .clone()
            .iter()
            .map(|item| {
                if item.sender == Never {
                    0
                } else {
                    item.message.lines().count() as u16 + TOP_BOTTOM_BORDER_TIME_HEIGHT
                }
            })
            .sum();

        let layouts: Vec<Constraint> = items
            .clone()
            .iter()
            .map(|item| {
                if let Never = item.sender {
                    Length(0)
                } else {
                    Length(item.message.lines().count() as u16 + TOP_BOTTOM_BORDER_TIME_HEIGHT)
                }
            })
            .collect();

        let chat_area_x = area.x;
        let chat_area_y = area.y;
        let chat_area_width = area.width;
        let chat_area_height = area.height;

        // 聊天区域高度，如果大于聊天记录区域高度，则显示聊天记录区域高度（可能有问题）
        let height = if chat_area_height > self.chat_history_area_height {
            chat_area_height
        } else {
            // 整个聊天区域的高度
            self.chat_history_area_height
        };
        // 这块区域将不会被实际渲染，此处 y + 1 为去掉上边框
        let chat_list_full_area = Rect::new(chat_area_x, chat_area_y + 1, chat_area_width, height);
        let mut chat_list_full_area_buf = Buffer::empty(chat_list_full_area);
        // 计算每一条聊天消息的位置
        let areas = Layout::vertical(layouts).split(chat_list_full_area);
        for (area, chat_message) in areas.iter().zip(items.iter()) {
            chat_message.clone().render(*area, &mut chat_list_full_area_buf);
        }

        // 将要被展示的聊天记录区域
        let visible_content = chat_list_full_area_buf
            .content
            .into_iter()
            .skip((chat_area_width * self.scroll_offset) as usize) // 跳过滚动条滚动位置头部的区域
            .take((chat_area_width * (chat_area_height - 2)) as usize); // 取出可见区域的内容，此处 -2 为去掉上边框和下边框

        let buf = frame.buffer_mut();
        for (i, cell) in visible_content.enumerate() {
            let x = i as u16 % chat_area_width;
            let y = i as u16 / chat_area_width;
            buf[(chat_list_full_area.x + x, chat_list_full_area.y + y)] = cell;
        }
        // 计算可见区域
        let show_chat_item_area = chat_list_full_area.intersection(buf.area);
        // 赋予该区域可滚动属性
        let mut state = ScrollbarState::new(0).position(self.scroll_offset as usize);
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(show_chat_item_area, buf, &mut state);
        // 给聊天记录区域渲染边框
        chat_block.render(area, buf);
    }
}
