use std::sync::mpsc;

use anyhow::Result;
use chrono::Local;
use gemini_api::body::GenerationConfig;
use gemini_api::model::blocking::Gemini;
use gemini_api::model::LanguageModel;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{List, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget};
use ratatui::Frame;
use ratatui::{
    crossterm::event::{self, Event, KeyEventKind},
    layout::{
        Constraint::{Fill, Length},
        Layout,
    },
    widgets::{Block, Borders, ListItem},
    DefaultTerminal,
};

use crate::model::ChatMessage;
use crate::model::Sender::{Bot, Split, User};
use crate::store::{read_config, save_config};

/// 窗口UI
#[derive(Default)]
pub struct UI {
    /// 是否正在接收消息
    receiving_message: bool,
    /// 消息响应失败
    response_status: ResponseStatus,
    /// 是否应该退出程序
    should_exit: bool,
    /// 输入框内容
    input_buffer: String,
    /// 聊天历史记录
    chat_history: Vec<ChatMessage>,
    /// Gemini API
    gemini: Option<Gemini>,

    cursor_props: CursorProps,
    scroll_props: ScrollProps,
}

/// 响应状态
#[derive(Default)]
pub enum ResponseStatus {
    #[default]
    None,
    /// 接收响应消息失败，提供错误信息
    Failed(String),
}

/// 指针位置相关属性
#[derive(Default)]
pub struct CursorProps {
    /// 指针位置，光标指向输入字符串中第几位
    cursor_index: usize,
    /// 字符位置，光标当前坐标，每一个 ASCII 字符占1位，非 ASCII 字符占2位
    /// 如果输入的文本为纯 ASCII 字符，则于 cursor_index 相等，如果包含非 ASCII 字符，则会比 cursor_index 大
    charactor_index: usize,
}

/// 滚动条相关属性
#[derive(Default)]
pub struct ScrollProps {
    /// 滚动条偏移量
    scroll_offset: u16,
    /// 聊天历史记录区域高度
    chat_history_area_height: u16,
    /// 最后一条记录的高度
    last_chat_history_height: u16,
    /// 是否需要添加一条空记录
    add_a_blank_line: bool,
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
                        let line = if value.success {
                            format!("{}  👤", line)
                        } else {
                            format!("{}❌👤", line)
                        };
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
            Split => {
                vec![Line::from(String::new()).alignment(Alignment::Center)]
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
            self.handle_key(tx.clone(), &rx);
        }
        Ok(())
    }

    /// 处理按键事件
    fn handle_key(&mut self, tx: mpsc::Sender<String>, rx: &mpsc::Receiver<String>) {
        if self.scroll_props.add_a_blank_line {
            self.scroll_props.add_a_blank_line = false;
            self.chat_history.push(ChatMessage {
                success: true,
                sender: Split,
                message: String::new(),
                date_time: Local::now(),
            });
        }
        // 如果接收消息位为真
        if self.receiving_message {
            // 阻塞接收消息
            if let Ok(request) = rx.recv() {
                match self.gemini.as_mut().unwrap().chat_conversation(request) {
                    // 成功接收响应消息后，将响应消息封装后加入到消息列表以供展示
                    Ok(response) => {
                        let response = response.replace("\n\n", "\n");
                        let response = if response.ends_with("\n") {
                            response[..response.len() - 1].to_owned()
                        } else {
                            response
                        };
                        self.chat_history.push(ChatMessage {
                            success: true,
                            sender: Bot,
                            message: response,
                            date_time: Local::now(),
                        });
                        self.scroll_props.add_a_blank_line = true;
                    }
                    // 接收响应消息失败，将响应状态位改为失败，并提供错误信息
                    Err(e) => {
                        if let Some(msg) = e.downcast_ref::<String>() {
                            self.response_status = ResponseStatus::Failed(msg.clone());
                        } else {
                            self.response_status = ResponseStatus::Failed("Unknown Error".to_owned());
                        }
                        // 将最后一条消息状态修改为失败
                        let mut chat_message = self.chat_history.pop().unwrap();
                        chat_message.success = false;
                        self.chat_history.push(chat_message);
                    }
                }
                self.receiving_message = false;
            }
            return;
        }

        // 接收键盘事件
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind != KeyEventKind::Press {
                return;
            }
            match key.code {
                event::KeyCode::Backspace => self.delete_pre_char(),
                event::KeyCode::Enter => self.submit_message(tx),
                event::KeyCode::Left => self.move_cursor_left(self.get_current_char()),
                event::KeyCode::Right => self.move_cursor_right(self.get_next_char()),
                event::KeyCode::Up => self.up(),
                event::KeyCode::Down => self.down(),
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
    }

    /// 聊天区域向上滚动
    fn up(&mut self) {
        self.scroll_props.scroll_offset = self.scroll_props.scroll_offset.saturating_sub(1);
    }

    /// 聊天区域向下滚动
    fn down(&mut self) {
        self.scroll_props.scroll_offset = self
            .scroll_props
            .scroll_offset
            .saturating_add(1)
            .min(self.max_scroll_offset());
    }

    fn max_scroll_offset(&self) -> u16 {
        self.scroll_props.chat_history_area_height - self.scroll_props.last_chat_history_height
    }

    /// 定位到字符串末尾
    fn end_of_cursor(&mut self) {
        self.cursor_props.cursor_index = self.input_buffer.chars().count();
        self.cursor_props.charactor_index = self.input_length();
    }

    /// 获取当前光标指向的字符
    fn get_current_char(&self) -> char {
        if self.cursor_props.cursor_index == 0 {
            '\0'
        } else {
            self.input_buffer
                .chars()
                .nth(self.cursor_props.cursor_index - 1)
                .unwrap()
        }
    }

    /// 获取当前光标的下一个字符
    fn get_next_char(&self) -> char {
        self.input_buffer
            .chars()
            .nth(self.cursor_props.cursor_index)
            .unwrap_or('\0')
    }

    /// 向左移动光标
    fn move_cursor_left(&mut self, c: char) {
        let origin_cursor_index = self.cursor_props.cursor_index;
        let cursor_moved_left = self.cursor_props.cursor_index.saturating_sub(1);
        self.cursor_props.cursor_index = self.clamp_cursor(cursor_moved_left);
        // 光标有变化
        if origin_cursor_index != self.cursor_props.cursor_index {
            self.cursor_props.charactor_index = if c.is_ascii() {
                self.cursor_props.charactor_index.saturating_sub(1)
            } else {
                self.cursor_props.charactor_index.saturating_sub(2)
            }
        }
    }

    /// 向右移动光标
    fn move_cursor_right(&mut self, c: char) {
        let origin_cursor_index = self.cursor_props.cursor_index;
        let cursor_moved_right = self.cursor_props.cursor_index.saturating_add(1);
        self.cursor_props.cursor_index = self.clamp_cursor(cursor_moved_right);
        // 光标有变化
        if origin_cursor_index != self.cursor_props.cursor_index {
            self.cursor_props.charactor_index = if c.is_ascii() {
                self.cursor_props.charactor_index.saturating_add(1)
            } else {
                self.cursor_props.charactor_index.saturating_add(2)
            }
        }
    }

    /// 输入字符
    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input_buffer.insert(index, new_char);
        self.move_cursor_right(new_char);
    }

    /// 获取当前光标位置的字节索引
    fn byte_index(&self) -> usize {
        self.input_buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_props.cursor_index)
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
        let is_not_cursor_leftmost = self.cursor_props.cursor_index != 0;
        if is_not_cursor_leftmost {
            let delete_char = self.get_current_char();
            let current_index = self.cursor_props.cursor_index;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input_buffer.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input_buffer.chars().skip(current_index);
            self.input_buffer = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left(delete_char);
        }
    }

    /// 删除当前光标位置的后一个字符
    fn delete_suf_char(&mut self) {
        let is_not_cursor_rightmost = self.cursor_props.cursor_index != self.input_buffer.chars().count();
        if is_not_cursor_rightmost {
            let current_index = self.cursor_props.cursor_index;
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
        self.cursor_props.cursor_index = 0;
        self.cursor_props.charactor_index = 0;
    }

    /// 提交消息
    fn submit_message(&mut self, tx: mpsc::Sender<String>) {
        if !self.input_buffer.is_empty() {
            if self.gemini.is_none() {
                self.init_gemini_api(Some(self.input_buffer.clone()));
            } else {
                self.chat_history.push(ChatMessage {
                    success: true,
                    sender: User,
                    message: self.input_buffer.clone(),
                    date_time: Local::now(),
                });
                // 将获取消息标志位置真，发送消息给下一次循环使用
                self.receiving_message = true;
                let _ = tx.send(self.input_buffer.clone());
            }
            self.input_buffer.clear();
            self.reset_cursor();
            // 滚动到最新的一条消息
            self.scroll_props.scroll_offset = self.max_scroll_offset();
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

    /// 尝试通过读取环境变量信息初始化 Gemini API
    fn init_gemini_api(&mut self, key: Option<String>) {
        match read_config() {
            Ok(gemini) => {
                // 读取到配置文件则直接使用配置文件中的 Gemini API
                self.gemini = Some(gemini)
            }
            Err(_) => {
                if let Some(key) = key {
                    // 尝试从 key 构造 Gemini API
                    let mut gemini = Gemini::new(key, LanguageModel::Gemini1_5Flash);
                    gemini.set_options(GenerationConfig {
                        maxOutputTokens: 2048,
                        ..GenerationConfig::default()
                    });
                    let _ = save_config(gemini.clone());
                    self.gemini = Some(gemini)
                } else if let Ok(key) = std::env::var("GEMINI_KEY") {
                    // 尝试从环境变量中读取密钥
                    let mut gemini = Gemini::new(key, LanguageModel::Gemini1_5Flash);
                    gemini.set_options(GenerationConfig {
                        maxOutputTokens: 2048,
                        ..GenerationConfig::default()
                    });
                    let _ = save_config(gemini.clone());
                    self.gemini = Some(gemini)
                }
            }
        }
    }

    /// 绘制UI
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        // 计算显示区域宽度
        // 这里 -2 的原因是输入框中具有两侧的的 1px 边框，此闭包用于限制每一行文本的最大宽度，
        // 如果大于这个数值，可能在文本需要换行时产生丢失文本的情况
        let chat_area_width = || area.width as usize - 2 - 5;
        // 计算输入框区域宽度
        // 这里 -2 的原因是因为输入框中具有两侧的的 1px 边框
        let input_area_width = || area.width as usize - 2;
        let [chat_area, input_area] = Layout::vertical([Fill(1), Length(3)]).areas(area);
        // 输入区域（底部）
        self.render_input_area(frame, input_area, input_area_width);
        // 聊天记录区域（顶部）
        self.render_chat_area(frame, chat_area, chat_area_width);
    }

    /// 渲染输入区域
    fn render_input_area<F>(&mut self, frame: &mut Frame, input_area: Rect, input_area_width: F)
    where
        F: Fn() -> usize,
    {
        // 输入区域（底部）
        let input_block_title = if self.gemini.is_none() {
            "Input Key"
        } else {
            "Input Text"
        };
        let input_block = Block::default()
            .title(input_block_title)
            .border_style(Style::default().fg(Color::Green))
            .borders(Borders::ALL);
        // 输入框内容
        let text = if self.input_length() > input_area_width() && self.cursor_props.charactor_index > input_area_width()
        {
            self.sub_input_buffer(
                self.cursor_props.charactor_index - input_area_width(),
                self.cursor_props.charactor_index,
            )
        } else {
            self.input_buffer.clone()
        };

        let input_paragraph = if self.receiving_message {
            // 如果处于等待消息接收状态，则显示等待提示
            let text = "Receiving message...".to_owned();
            Paragraph::new(text)
                .block(input_block)
                .style(Style::default().fg(Color::Cyan))
        } else if let ResponseStatus::Failed(msg) = &self.response_status {
            // 接收响应消息失败
            let text = msg.clone();
            self.response_status = ResponseStatus::None;
            Paragraph::new(text)
                .block(input_block)
                .style(Style::default().fg(Color::Red))
        } else {
            Paragraph::new(text)
                .block(input_block)
                .style(Style::default().fg(Color::Yellow))
        };

        frame.render_widget(input_paragraph, input_area);
        frame.set_cursor_position(Position::new(
            input_area.x + self.cursor_props.charactor_index as u16 + 1,
            input_area.y + 1,
        ));
    }

    /// 渲染聊天记录区域
    fn render_chat_area<F>(&mut self, frame: &mut Frame, chat_area: Rect, chat_area_width: F)
    where
        F: Fn() -> usize,
    {
        let chat_block = Block::default()
            .title("Chat")
            .border_style(Style::default().fg(Color::Blue))
            .borders(Borders::ALL);
        let items: Vec<ListItem> = self
            .chat_history
            .iter()
            .map(|m| {
                let area_width = chat_area_width();
                let mut message = String::new();
                // 对长文本进行插入换行符号
                let mut line_width = 0;
                for (_, c) in m.message.clone().char_indices() {
                    if line_width >= area_width {
                        message.push('\n');
                        line_width = 0;
                    }
                    message.push(c);
                    line_width += if c.is_ascii() { 1 } else { 2 };
                    if c == '\n' {
                        line_width = 0;
                    }
                }
                ChatMessage { message, ..m.clone() }
            })
            .map(|m| (&m).into())
            .collect();
        // 保存最后一条记录的高度，用于计算滚动条位置
        self.scroll_props.last_chat_history_height = items.clone().iter().last().map_or(0, |item| item.height()) as u16;
        // 计算当前聊天记录区域高度
        self.scroll_props.chat_history_area_height = items.clone().iter().map(|item| item.height() as u16).sum();
        let chat_list = List::new(items)
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::TOP))
            .style(Style::default().fg(Color::White));

        let chat_area_width = chat_area.width;
        let chat_area_height = chat_area.height;

        // 聊天区域高度，如果大于聊天记录区域高度，则显示聊天记录区域高度（可能有问题）TODO
        let height = if chat_area_height > self.scroll_props.chat_history_area_height {
            chat_area_height
        } else {
            // 滚动到最新的一条消息
            self.scroll_props.chat_history_area_height
        };
        // 这块区域将不会被实际渲染
        let chat_list_full_area = Rect::new(0, 0, chat_area_width, height);
        let mut chat_list_full_area_buf = Buffer::empty(chat_list_full_area);

        // 将所有列表内容渲染到这块区域中
        Widget::render(chat_list, chat_list_full_area, &mut chat_list_full_area_buf);

        let visible_content = chat_list_full_area_buf
            .content
            .into_iter()
            .skip((chat_area_width * self.scroll_props.scroll_offset) as usize) // 跳过滚动条滚动位置头部的区域
            .take((chat_area_width * chat_area_height) as usize); // 取出可见区域的内容

        let buf = frame.buffer_mut();
        for (i, cell) in visible_content.enumerate() {
            let x = i as u16 % chat_area_width;
            let y = i as u16 / chat_area_width;
            buf[(chat_list_full_area.x + x, chat_list_full_area.y + y)] = cell;
        }

        let show_chat_item_area = chat_list_full_area.intersection(buf.area);
        let mut state = ScrollbarState::new(0).position(self.scroll_props.scroll_offset as usize);
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(show_chat_item_area, buf, &mut state);
        // 给聊天记录区域渲染边框
        chat_block.render(chat_area, buf);
    }
}
