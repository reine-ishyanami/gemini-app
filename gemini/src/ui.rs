use anyhow::Result;
use gemini_api::model::blocking::Gemini;
use gemini_api::model::LanguageModel;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
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
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::ChatMessage;
use crate::model::Sender::{Bot, User};

/// 窗口UI
#[derive(Default)]
pub struct UI {
    receiving_message: bool,
    should_exit: bool,
    input_buffer: String,
    cursor_pos: usize,
    chat_history: Vec<ChatMessage>,
    gemini: Option<Gemini>,
}

impl From<&ChatMessage> for ListItem<'_> {
    fn from(value: &ChatMessage) -> Self {
        let lines = match value.sender {
            User => {
                let message = value.message.clone();
                let message_lines = message.split("\n");
                let mut lines = Vec::new();
                for line in message_lines {
                    lines.push(
                        Line::from(line.to_string())
                            .alignment(Alignment::Right)
                            .style(Style::default().fg(Color::Green)),
                    );
                }
                lines
            }
            Bot => {
                let message = value.message.clone();
                let message_lines = message.split("\n");
                let mut lines = Vec::new();
                for line in message_lines {
                    lines.push(
                        Line::from(line.to_string())
                            .alignment(Alignment::Left)
                            .style(Style::default().fg(Color::Cyan)),
                    );
                }
                lines
            }
        };
        ListItem::new(lines)
    }
}

impl UI {
    /// 启动UI
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        self.init_gemini_api(None);
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key, tx.clone(), &rx);
            };
        }
        Ok(())
    }

    /// 处理按键事件
    fn handle_key(&mut self, key: KeyEvent, _: Sender<String>, rx: &Receiver<String>) {
        if self.receiving_message {
            if let Ok(message) = rx.try_recv() {
                self.chat_history.push(ChatMessage {
                    sender: Bot,
                    message,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                });
                self.receiving_message = false;
            }
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
                    if self.gemini.is_none() {
                        self.init_gemini_api(Some(self.input_buffer.clone()));
                    } else {
                        self.chat_history.push(ChatMessage {
                            sender: User,
                            message: self.input_buffer.clone(),
                            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        });
                    }
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

    /// 尝试通过读取环境变量信息初始化Gemini API
    pub fn init_gemini_api(&mut self, key: Option<String>) {
        if let Some(key) = key {
            self.gemini = Some(Gemini::new(key, LanguageModel::Gemini1_5Flash))
        } else if let Ok(key) = std::env::var("GEMINI_KEY") {
            self.gemini = Some(Gemini::new(key, LanguageModel::Gemini1_5Flash))
        }
    }
}

impl Widget for &mut UI {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input_block_title = if self.gemini.is_none() {
            "Input Key"
        } else {
            "Input Text"
        };
        let [chat_area, input_area_area] = Layout::vertical([Min(5), Length(3)]).areas(area);
        let chat_block = Block::default()
            .title("Chat")
            .border_style(Style::default().fg(Color::Blue))
            .borders(Borders::ALL);
        let input_block = Block::default().title(input_block_title).borders(Borders::ALL);
        let input_paragraph = Paragraph::new(self.input_buffer.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::Yellow));
        // 输入区域
        input_paragraph.render(input_area_area, buf);
        let items: Vec<ListItem> = self
            .chat_history
            .iter()
            .map(|m| {
                // 这里 -2 的原因是因为输入框中具有两侧的的 1px 边框
                let area_width = input_area_area.width as usize - 2;
                let mut message_max_width = m.message.len();
                // 对长文本进行插入换行符号
                let mut message = m.message.clone();
                while message_max_width > area_width {
                    message.insert(area_width, '\n');
                    message_max_width -= area_width;
                }
                ChatMessage {
                    sender: m.sender.clone(),
                    message,
                    timestamp: m.timestamp,
                }
            })
            .map(|m| (&m).into())
            .collect();
        let chat_list = List::new(items)
            .block(chat_block)
            .style(Style::default().fg(Color::White));
        // 聊天记录区域
        chat_list.render(chat_area, buf);
    }
}
