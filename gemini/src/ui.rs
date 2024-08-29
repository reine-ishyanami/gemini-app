use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::ChatMessage;
use crate::model::Sender::{Bot, User};
use anyhow::Result;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{List, Paragraph};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    layout::{
        Constraint::{Length, Min},
        Layout, Rect,
    },
    widgets::{Block, Borders, ListItem, Widget},
    DefaultTerminal,
};

/// 窗口UI
#[derive(Default)]
pub struct UI {
    receiving_message: bool,
    should_exit: bool,
    input_buffer: String,
    cursor_pos: usize,
    chat_history: Vec<ChatMessage>,
}

impl From<&ChatMessage> for ListItem<'_> {
    fn from(value: &ChatMessage) -> Self {
        let line = match value.sender {
            User => Line::styled(format!("User: {}", value.message), Style::new().yellow().italic()),
            Bot => Line::styled(format!("Model: {}", value.message), Style::new().blue().italic()),
        };
        ListItem::new(line)
    }
}

impl UI {
    /// 启动UI
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    /// 处理按键事件
    fn handle_key(&mut self, key: KeyEvent) {
        if self.receiving_message {
            return;
        }
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            event::KeyCode::Backspace => {
                if !self.input_buffer.is_empty() && self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input_buffer.remove(self.cursor_pos);
                }
            }
            event::KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    self.chat_history.push(ChatMessage {
                        sender: User,
                        message: self.input_buffer.clone(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    });
                    self.input_buffer.clear();
                    self.cursor_pos = 0;
                }
            }
            event::KeyCode::Left => {
                if !self.input_buffer.is_empty() && self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            event::KeyCode::Right => {
                if self.input_buffer.len() > self.cursor_pos {
                    self.cursor_pos += 1;
                }
            }
            event::KeyCode::Home => {
                self.cursor_pos = 0;
            }
            event::KeyCode::End => {
                self.cursor_pos = self.input_buffer.len();
            }
            event::KeyCode::Tab => {
                self.input_buffer.insert_str(self.cursor_pos, "    ");
                self.cursor_pos += 4;
            }
            event::KeyCode::Delete => {
                if !self.input_buffer.is_empty() && self.cursor_pos != self.input_buffer.len() {
                    self.input_buffer.remove(self.cursor_pos);
                }
            }
            event::KeyCode::Char(x) => {
                self.input_buffer.insert(self.cursor_pos, x);
                self.cursor_pos += 1;
            }
            event::KeyCode::Esc => {
                self.should_exit = true;
            }
            _ => {}
        }
    }
}

impl Widget for &mut UI {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [chat_area, input_area_area] = Layout::vertical([Min(5), Length(3)]).areas(area);
        let chat_block = Block::default().title("Chat").borders(Borders::ALL);
        let input_block = Block::default().title("Input").borders(Borders::ALL);
        let items: Vec<ListItem> = self.chat_history.iter().map(|m| m.into()).collect();
        let chat_list = List::new(items)
            .block(chat_block)
            .style(Style::default().fg(Color::White));
        let input_paragraph = Paragraph::new(self.input_buffer.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::Yellow));
        // 聊天记录区域
        chat_list.render(chat_area, buf);
        // 输入区域
        input_paragraph.render(input_area_area, buf);
    }
}
