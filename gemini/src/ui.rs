use anyhow::Result;
use chrono::Local;
use gemini_api::body::GenerationConfig;
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
        Constraint::{Fill, Length},
        Layout, Rect,
    },
    widgets::{Block, Borders, ListItem, Widget},
    DefaultTerminal,
};

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
                let mut line_width = 0;
                for line in message_lines {
                    let line = if line_width == 0 {
                        let line = format!("{} 👤", line);
                        line_width = line.len();
                        line
                    } else {
                        line.to_owned()
                    };
                    lines.push(
                        Line::from(format!("{:width$}", line, width = line_width))
                            .alignment(Alignment::Right)
                            .style(Style::default().fg(Color::Green)),
                    );
                }
                lines.push(
                    Line::from(value.date_time.format("%H:%M:%S").to_string())
                        .alignment(Alignment::Right)
                        .style(Style::default().fg(Color::Cyan)),
                );
                lines
            }
            Bot => {
                let message = value.message.clone();
                let message_lines = message.split("\n");
                let mut lines = Vec::new();
                let mut line_width = 0;
                for line in message_lines {
                    let line = if line_width == 0 {
                        let line = format!("🤖 {}", line);
                        line_width = line.len();
                        line
                    } else {
                        let line = format!("   {}", line);
                        line.to_owned()
                    };
                    lines.push(
                        Line::from(line.to_string())
                            .alignment(Alignment::Left)
                            .style(Style::default().fg(Color::Blue)),
                    );
                }
                lines.push(
                    Line::from(value.date_time.format("%H:%M:%S").to_string())
                        .alignment(Alignment::Left)
                        .style(Style::default().fg(Color::Cyan)),
                );
                lines
            }
        };
        ListItem::new(lines)
    }
}

impl UI {
    /// 启动UI
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.init_gemini_api(None);
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
                    if self.gemini.is_none() {
                        self.init_gemini_api(Some(self.input_buffer.clone()));
                    } else {
                        self.chat_history.push(ChatMessage {
                            sender: User,
                            message: self.input_buffer.clone(),
                            date_time: Local::now(),
                        });
                        let response = self
                            .gemini
                            .as_mut()
                            .unwrap()
                            .chat_conversation(self.input_buffer.clone())
                            .unwrap();
                        let response = if response.ends_with("\n") {
                            response[..response.len() - 1].to_owned()
                        } else {
                            response
                        };
                        self.chat_history.push(ChatMessage {
                            sender: Bot,
                            message: response,
                            date_time: Local::now(),
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
            event::KeyCode::Up => {}
            event::KeyCode::Down => {}
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
            let mut gemini = Gemini::new(key, LanguageModel::Gemini1_5Flash);
            gemini.set_options(GenerationConfig {
                maxOutputTokens: 2048,
                ..GenerationConfig::default()
            });
            self.gemini = Some(gemini)
        } else if let Ok(key) = std::env::var("GEMINI_KEY") {
            let mut gemini = Gemini::new(key, LanguageModel::Gemini1_5Flash);
            gemini.set_options(GenerationConfig {
                maxOutputTokens: 2048,
                ..GenerationConfig::default()
            });
            self.gemini = Some(gemini)
        }
    }
}

impl Widget for &mut UI {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 计算显示区域宽度
        let chat_area_width = || area.width as usize - 2 - 5;
        // 这里 -2 的原因是因为输入框中具有两侧的的 1px 边框
        // 计算输入框区域宽度
        let input_area_width = || area.width as usize - 2;

        let input_block_title = if self.gemini.is_none() {
            "Input Key"
        } else {
            "Input Text"
        };
        let [chat_area, input_area_area] = Layout::vertical([Fill(1), Length(3)]).areas(area);
        let chat_block = Block::default()
            .title("Chat")
            .border_style(Style::default().fg(Color::Blue))
            .borders(Borders::ALL);
        let input_block = Block::default()
            .title(input_block_title)
            .border_style(Style::default().fg(Color::Green))
            .borders(Borders::ALL);
        // 输入框内容
        let text = if self.input_buffer.len() > input_area_width() && self.cursor_pos > input_area_width() {
            &self.input_buffer[self.cursor_pos - input_area_width()..self.cursor_pos]
        } else {
            self.input_buffer.as_str()
        };
        let input_paragraph = Paragraph::new(text)
            .block(input_block)
            .style(Style::default().fg(Color::Yellow));
        // 输入区域
        input_paragraph.render(input_area_area, buf);
        let items: Vec<ListItem> = self
            .chat_history
            .iter()
            .map(|m| {
                let area_width = chat_area_width();
                let mut message_max_width = m.message.len();
                // 如果是用户进行提问的消息，则将换行符改为 ',' ; 如果是机器人回答的消息，则照搬
                let mut message = match m.sender {
                    User => m.message.clone().replace("\n", ","),
                    Bot => m.message.clone(),
                };
                // 对长文本进行插入换行符号
                let mut line = 1;
                while message_max_width > area_width {
                    message.insert(area_width * line + line - 1, '\n');
                    message_max_width -= area_width;
                    line += 1;
                }
                ChatMessage {
                    sender: m.sender.clone(),
                    message,
                    date_time: m.date_time,
                }
            })
            .map(|m| (&m).into())
            .collect();
        let chat_list = List::new(items)
            .block(chat_block)
            .style(Style::default().fg(Color::White))
            .scroll_padding(10);
        // 聊天记录区域
        Widget::render(chat_list, chat_area, buf);
    }
}
