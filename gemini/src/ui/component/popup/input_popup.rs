use ratatui::{
    crossterm::event,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{block::Title, Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::ui::component::input::{input_trait::InputTextComponent, text_field::TextField};
use ratatui::widgets::block::title::Position as TitlePosition;

use ratatui::layout::Position as CursorPosition;

pub struct InputPopup {
    // 提示文本
    pub input_text: TextField,
    pub width: usize,
    pub height: usize,
    // 边框颜色
    pub border_color: Color,
}

impl InputPopup {
    pub fn new(title: String, width: usize, height: usize) -> Self {
        let mut input_text = TextField::new(title);
        input_text.set_width_height(width - 2, height - 2);
        Self {
            input_text,
            width,
            height,
            border_color: Color::Blue,
        }
    }

    // 保存
    pub fn save(&mut self) -> String {
        // TODO: 保存输入内容
        self.input_text.get_content()
    }

    // 取消
    pub fn cancel(&self) -> bool {
        true
    }
}

impl InputPopup {
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        // 先清空弹窗区域内容
        frame.render_widget(Clear, area);
        let input_block = Block::bordered()
            .title(
                Title::from("Image Path Or URL")
                    .position(TitlePosition::Top)
                    .alignment(Alignment::Left),
            )
            .title(
                Title::from("Save (Ctrl+S)")
                    .position(TitlePosition::Bottom)
                    .alignment(Alignment::Left),
            )
            .title(
                Title::from("Cancel (ESC)")
                    .position(TitlePosition::Bottom)
                    .alignment(Alignment::Right),
            )
            .borders(Borders::ALL)
            .border_style(self.border_color);
        // 输入框内容
        let text = self.input_text.should_show_text();
        let input_paragraph = Paragraph::new(text)
            .block(input_block)
            .style(Style::default().fg(Color::Yellow));
        // 渲染输入框
        frame.render_widget(input_paragraph, area);
        let (x, y) = self.input_text.get_cursor_position();
        frame.set_cursor_position(CursorPosition::new(area.x + x as u16 + 1, area.y + y as u16 + 1));
    }
}

pub enum InputPopupHandleEvent {
    Save(String),
    Cancel,
    Nothing,
}

impl InputPopup {
    pub fn handle_key(&mut self, key: event::KeyEvent) -> InputPopupHandleEvent {
        match key.code {
            event::KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                InputPopupHandleEvent::Save(self.save())
            }
            event::KeyCode::Esc => {
                self.cancel();
                InputPopupHandleEvent::Cancel
            }
            event::KeyCode::Backspace => {
                self.input_text.delete_pre_char();
                InputPopupHandleEvent::Nothing
            }
            event::KeyCode::Left => {
                self.input_text.move_cursor_left(self.input_text.get_current_char());
                InputPopupHandleEvent::Nothing
            }
            event::KeyCode::Right => {
                self.input_text.move_cursor_right(self.input_text.get_next_char());
                InputPopupHandleEvent::Nothing
            }
            event::KeyCode::Home => {
                self.input_text.home_of_cursor();
                InputPopupHandleEvent::Nothing
            }
            event::KeyCode::End => {
                self.input_text.end_of_cursor();
                InputPopupHandleEvent::Nothing
            }
            event::KeyCode::Delete => {
                self.input_text.delete_suf_char();
                InputPopupHandleEvent::Nothing
            }
            event::KeyCode::Char(x) => {
                self.input_text.enter_char(x);
                InputPopupHandleEvent::Nothing
            }
            _ => InputPopupHandleEvent::Nothing,
        }
    }
}
