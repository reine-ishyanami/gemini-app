#![allow(dead_code)]

use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint::{self, Length},
        Layout, Rect,
    },
    style::{Color, Style},
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
    Frame,
};

use crate::{
    model::db::Conversation,
    ui::component::popup::delete_popup::DeletePopup,
    utils::db_utils::{delete_one, query_all, query_detail_by_id},
};

/// 滚动条相关属性
#[derive(Default)]
pub struct ChatItemListScrollProps {
    /// 聊天历史记录
    pub chat_history: Vec<SelectableConversation>,
    /// 滚动条偏移量
    pub scroll_offset: u16,
    /// 聊天历史记录区域高度
    pub chat_history_area_height: u16,
    /// 最后一条记录的高度
    pub last_chat_history_height: u16,
    /// 是否需要添加一条空记录
    pub add_a_blank_line: bool,
    /// 选中的会话
    pub selected_conversation: usize,
    /// 是否展示确认删除弹窗
    pub popup_delete_confirm_dialog: Option<DeletePopup>,
}
/// 可一被选中的会话
#[derive(Clone, Debug)]
pub struct SelectableConversation {
    /// 选中的会话
    pub conversation: Conversation,
    /// 是否选中
    pub selected: bool,
    /// 是否聚焦
    pub focused: bool,
}

impl ChatItemListScrollProps {
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, is_focused: bool) {
        // 查询所有会话
        self.chat_history = self.query_all(is_focused);
        // 最外侧的边框
        let chat_list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if is_focused { Color::Green } else { Color::White }));

        let heights: Vec<u16> = (0..self.chat_history.len()).map(|_| 3).collect();

        let height_sum: u16 = heights.iter().sum();
        let layouts: Vec<Constraint> = heights.iter().map(|x| Length(*x)).collect();

        let item_list_x = area.x;
        let item_list_y = area.y;
        let item_list_width = area.width - 2;
        let item_list_height = area.height;

        // 这块区域将不会被实际渲染
        let item_list_full_area = Rect::new(item_list_x + 1, item_list_y + 1, item_list_width, height_sum);
        let mut item_list_full_area_buf = Buffer::empty(item_list_full_area);

        let areas = Layout::vertical(layouts).split(item_list_full_area);
        for (area, chat_message) in areas.iter().zip(self.chat_history.iter()) {
            chat_message.clone().render(*area, &mut item_list_full_area_buf);
        }

        let visible_content = item_list_full_area_buf
            .content
            .into_iter()
            .skip((item_list_width * self.scroll_offset) as usize) // 跳过滚动条滚动位置头部的区域
            .take((item_list_width * (item_list_height - 1)) as usize); // 取出可见区域的内容，此处 -1 为去掉上边框和下边框（受上面的 y + 1 影响，此处必须如此）

        let buf = frame.buffer_mut();

        for (i, cell) in visible_content.enumerate() {
            let x = i as u16 % item_list_width;
            let y = i as u16 / item_list_width;
            buf[(item_list_full_area.x + x, item_list_full_area.y + y)] = cell;
        }

        let show_chat_item_area = item_list_full_area.intersection(buf.area);
        let mut state = ScrollbarState::new(0).position(self.scroll_offset as usize);
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(show_chat_item_area, buf, &mut state);

        chat_list_block.render(area, buf);
    }

    /// 重建聊天记录
    pub fn rebuild(&self) -> Option<Conversation> {
        let selected_conversation = self.chat_history.get(self.selected_conversation)?;
        if let Ok(conversation) = query_detail_by_id(selected_conversation.conversation.clone()) {
            Some(conversation)
        } else {
            None
        }
    }

    /// 选中下一个会话
    pub fn next_item(&mut self) {
        if self.selected_conversation < self.chat_history.len() - 1 {
            self.selected_conversation += 1;
        }
    }
    /// 选中上一个会话
    pub fn prev_item(&mut self) {
        if self.selected_conversation > 0 {
            self.selected_conversation -= 1;
        }
    }
    /// 删除选中的会话
    pub fn delete_item(&mut self) {
        if let Some(selected_conversation) = self.chat_history.get(self.selected_conversation) {
            let _ = delete_one(selected_conversation.conversation.conversation_id.clone());
        }
    }

    /// 查询所有会话
    fn query_all(&self, focused: bool) -> Vec<SelectableConversation> {
        let mut conversations = Vec::new();
        for (index, conversation) in query_all().unwrap_or_default().iter().enumerate() {
            let conversation = conversation.clone();
            if index == self.selected_conversation {
                conversations.push(SelectableConversation {
                    conversation,
                    selected: true,
                    focused,
                });
            } else {
                conversations.push(SelectableConversation {
                    conversation,
                    selected: false,
                    focused,
                });
            }
        }
        conversations
    }
}
