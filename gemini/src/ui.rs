use std::sync::mpsc;

use anyhow::Result;
use chrono::Local;
use gemini_api::body::GenerationConfig;
use gemini_api::model::blocking::Gemini;
use gemini_api::model::LanguageModel;
use ratatui::layout::{Alignment, Position};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{List, Paragraph};
use ratatui::Frame;
use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    layout::{
        Constraint::{Fill, Length},
        Layout,
    },
    widgets::{Block, Borders, ListItem},
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
    chat_history: Vec<ChatMessage>,
    gemini: Option<Gemini>,
    /// 指针位置，每个ASCII字符占两格，非ASCII字符占两格
    cursor_index: usize,
    /// 字符位置，光标当前坐标（这个参数比 cursor_index 的大或相等）
    charactor_index: usize,
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
                        line_width = line.chars().count();
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
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key, tx.clone(), &rx);
            };
        }
        Ok(())
    }

    /// 处理按键事件
    fn handle_key(&mut self, key: KeyEvent, tx: mpsc::Sender<String>, rx: &mpsc::Receiver<String>) {
        if self.receiving_message {
            if let Ok(request) = rx.recv() {
                let response = self.gemini.as_mut().unwrap().chat_conversation(request).unwrap();
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
                self.receiving_message = false;
            }
            return;
        }
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            event::KeyCode::Backspace => self.delete_pre_char(),
            event::KeyCode::Enter => self.submit_message(tx),
            event::KeyCode::Left => self.move_cursor_left(self.get_current_char()),
            event::KeyCode::Right => self.move_cursor_right(self.get_next_char()),
            event::KeyCode::Up => {}
            event::KeyCode::Down => {}
            event::KeyCode::Home => self.reset_cursor(),
            event::KeyCode::End => self.end_of_cursor(),
            event::KeyCode::Delete => self.delete_suf_char(),
            event::KeyCode::Char(x) => self.enter_char(x),
            event::KeyCode::Esc => {
                self.should_exit = true;
            }
            _ => {}
        }
    }

    /// 定位到字符串末尾
    fn end_of_cursor(&mut self) {
        self.cursor_index = self.input_buffer.chars().count();
        self.charactor_index = self.input_length();
    }

    /// 获取当前光标指向的字符
    fn get_current_char(&self) -> char {
        if self.cursor_index == 0 {
            '\0'
        } else {
            self.input_buffer.chars().nth(self.cursor_index - 1).unwrap()
        }
    }

    /// 获取当前光标的下一个字符
    fn get_next_char(&self) -> char {
        self.input_buffer.chars().nth(self.cursor_index).unwrap_or('\0')
    }

    /// 向左移动光标
    fn move_cursor_left(&mut self, c: char) {
        let origin_cursor_index = self.cursor_index;
        let cursor_moved_left = self.cursor_index.saturating_sub(1);
        self.cursor_index = self.clamp_cursor(cursor_moved_left);
        // 光标有变化
        if origin_cursor_index != self.cursor_index {
            self.charactor_index = if c.is_ascii() {
                self.charactor_index.saturating_sub(1)
            } else {
                self.charactor_index.saturating_sub(2)
            }
        }
    }

    /// 向右移动光标
    fn move_cursor_right(&mut self, c: char) {
        let origin_cursor_index = self.cursor_index;
        let cursor_moved_right = self.cursor_index.saturating_add(1);
        self.cursor_index = self.clamp_cursor(cursor_moved_right);
        // 光标有变化
        if origin_cursor_index != self.cursor_index {
            self.charactor_index = if c.is_ascii() {
                self.charactor_index.saturating_add(1)
            } else {
                self.charactor_index.saturating_add(2)
            }
        }
    }

    /// 输入字符
    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input_buffer.insert(index, new_char);
        self.move_cursor_right(new_char);
    }

    fn byte_index(&self) -> usize {
        self.input_buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_index)
            .unwrap_or(self.input_buffer.len())
    }

    /// 获取输入框字符长度
    fn input_length(&self) -> usize {
        self.input_buffer
            .chars()
            .map(|c| if c.is_ascii() { 1 } else { 2 })
            .sum()
    }

    /// 删除当前光标指向字符
    fn delete_pre_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_index != 0;
        if is_not_cursor_leftmost {
            let delete_char = self.get_current_char();
            let current_index = self.cursor_index;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input_buffer.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input_buffer.chars().skip(current_index);
            self.input_buffer = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left(delete_char);
        }
    }

    /// 删除当前光标位置的后一个字符
    fn delete_suf_char(&mut self) {
        let is_not_cursor_rightmost = self.cursor_index != self.input_buffer.chars().count();
        if is_not_cursor_rightmost {
            let current_index = self.cursor_index;
            let from_left_to_current_index = current_index + 1;
            let before_char_to_delete = self.input_buffer.chars().take(current_index);
            let after_char_to_delete = self.input_buffer.chars().skip(from_left_to_current_index);
            self.input_buffer = before_char_to_delete.chain(after_char_to_delete).collect();
        }
    }

    /// 限制光标位置
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input_buffer.chars().count())
    }

    /// 重置光标位置
    fn reset_cursor(&mut self) {
        self.cursor_index = 0;
        self.charactor_index = 0;
    }

    /// 提交消息
    fn submit_message(&mut self, tx: mpsc::Sender<String>) {
        if !self.input_buffer.is_empty() {
            if self.gemini.is_none() {
                self.init_gemini_api(Some(self.input_buffer.clone()));
            } else {
                self.chat_history.push(ChatMessage {
                    sender: User,
                    message: self.input_buffer.clone(),
                    date_time: Local::now(),
                });
                self.receiving_message = true;
                let _ = tx.send(self.input_buffer.clone());
                // let response = self
                //     .gemini
                //     .as_mut()
                //     .unwrap()
                //     .chat_conversation(self.input_buffer.clone())
                //     .unwrap();
                // let response = if response.ends_with("\n") {
                //     response[..response.len() - 1].to_owned()
                // } else {
                //     response
                // };
                // self.chat_history.push(ChatMessage {
                //     sender: Bot,
                //     message: response,
                //     date_time: Local::now(),
                // });
            }
            self.input_buffer.clear();
            self.reset_cursor();
        }
    }

    /// 截取 input_buffer 字符串以供UI展示
    fn sub_input_buffer(&self, start: usize, count: usize) -> String {
        let mut result = String::new();
        let mut char_count = 0;

        for (i, c) in self.input_buffer.char_indices() {
            // 当我们达到起始字符索引时开始截取
            if i >= start && char_count < count {
                result.push(c);
                char_count += 1;
            }
            // 当我们截取了足够的字符后停止
            if char_count == count {
                break;
            }
        }
        result
    }

    /// 尝试通过读取环境变量信息初始化Gemini API
    fn init_gemini_api(&mut self, key: Option<String>) {
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

    /// 绘制UI
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        // 计算显示区域宽度
        let chat_area_width = || area.width as usize - 2;
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
        let mut text = if self.input_length() > input_area_width() && self.charactor_index > input_area_width() {
            self.sub_input_buffer(self.charactor_index - input_area_width(), self.charactor_index)
        } else {
            self.input_buffer.clone()
        };

        // 如果处于等待消息接收状态，则显示等待提示
        if self.receiving_message {
            text = "Receiving message...".to_owned();
        }

        let input_paragraph = Paragraph::new(text)
            .block(input_block)
            .style(Style::default().fg(Color::Yellow));
        // 输入区域
        // input_paragraph.render(input_area_area, buf);
        frame.render_widget(input_paragraph, input_area_area);
        frame.set_cursor_position(Position::new(
            input_area_area.x + self.charactor_index as u16 + 1,
            input_area_area.y + 1,
        ));
        let items: Vec<ListItem> = self
            .chat_history
            .iter()
            .map(|m| {
                let area_width = chat_area_width();
                let mut message = String::new();
                // 对长文本进行插入换行符号
                let mut line_width = 0;
                for (_, c) in m.message.clone().char_indices() {
                    if line_width > area_width {
                        message.push('\n');
                        line_width = 0;
                    }
                    message.push(c);
                    line_width += if c.is_ascii() { 1 } else { 2 };
                    if c == '\n' {
                        line_width = 0;
                    }
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
        frame.render_widget(chat_list, chat_area);
    }
}
