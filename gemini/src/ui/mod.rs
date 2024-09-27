mod component;
mod enum_from;
mod setting;
mod widget;

use std::sync::mpsc;

use anyhow::Result;
use chrono::Local;
use component::input::{input_trait::InputTextComponent, text_field::TextField};
use gemini_api::body::request::GenerationConfig;
use gemini_api::model::blocking::Gemini;
use gemini_api::param::LanguageModel;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Position as CursorPosition, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::block::{Position as TitlePosition, Title};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget};
use ratatui::Frame;
use ratatui::{
    crossterm::event::{self, Event, KeyEventKind},
    layout::{
        Constraint::{Fill, Length},
        Layout,
    },
    widgets::{Block, Borders},
    DefaultTerminal,
};
use setting::SettingUI;

use crate::model::ChatMessage;
use crate::model::Sender::{Bot, Split, User};
use crate::utils::char_utils::c_len;
use crate::utils::store_utils::{read_config, save_config, StoreData};

/// 窗口UI
#[derive(Default)]
pub struct UI {
    /// 是否正在接收消息
    receiving_message: bool,
    /// 消息响应失败
    response_status: ResponseStatus,
    /// 是否应该退出程序
    should_exit: bool,
    /// 聊天历史记录
    chat_history: Vec<ChatMessage>,
    /// Gemini API
    gemini: Option<Gemini>,
    /// 当前聚焦的组件
    focus_component: MainFocusComponent,
    /// 输入区域组件
    input_field_component: TextField,
    scroll_props: ScrollProps,
    /// 当前窗口
    current_windows: CurrentWindows,
    /// 图片路径
    image_path: Option<String>,
}
/// 窗口枚举
#[derive(Default)]
pub enum CurrentWindows {
    #[default]
    This,
    SettingWindow(SettingUI),
}

/// 当前聚焦组件
#[derive(Default, PartialEq, Eq)]
pub enum MainFocusComponent {
    #[default]
    InputArea,
    ExitButton,
    SettingButton,
}

/// 响应状态
#[derive(Default, PartialEq, Eq)]
pub enum ResponseStatus {
    #[default]
    None,
    /// 接收响应消息失败，提供错误信息
    Failed(String),
}

#[derive(PartialEq, Eq)]
enum ChatType {
    Simple { message: String },
    Image { message: String, image_path: String },
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

impl UI {
    /// 启动UI
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        self.restore_or_new_gemini(None);
        while !self.should_exit {
            match self.current_windows {
                CurrentWindows::This => {
                    terminal.draw(|frame| self.draw(frame))?;
                    self.handle_key(tx.clone(), &rx);
                }
                CurrentWindows::SettingWindow(ref mut setting_ui) => {
                    if setting_ui.should_exit {
                        // 如果配置更新了，则重构 Gemini API
                        if setting_ui.update {
                            self.restore_or_new_gemini(None);
                        }
                        self.current_windows = CurrentWindows::This;
                    } else {
                        terminal.draw(|frame| setting_ui.draw(frame))?;
                        setting_ui.handle_key();
                    }
                }
            }
        }
        Ok(())
    }

    /// 处理按键事件
    fn handle_key(&mut self, tx: mpsc::Sender<ChatType>, rx: &mpsc::Receiver<ChatType>) {
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
                match request {
                    ChatType::Simple { message } => {
                        match self.gemini.as_mut().unwrap().chat_conversation(message) {
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
                                    self.response_status = ResponseStatus::Failed("Unknown Error".into());
                                }
                                // 将最后一条消息状态修改为失败
                                let mut chat_message = self.chat_history.pop().unwrap();
                                chat_message.success = false;
                                self.chat_history.push(chat_message);
                            }
                        }
                    }
                    ChatType::Image { message, image_path } => {
                        match self
                            .gemini
                            .as_mut()
                            .unwrap()
                            .image_analysis_conversation(image_path, message)
                        {
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
                                    self.response_status = ResponseStatus::Failed("Unknown Error".into());
                                }
                                // 将最后一条消息状态修改为失败
                                let mut chat_message = self.chat_history.pop().unwrap();
                                chat_message.success = false;
                                self.chat_history.push(chat_message);
                            }
                        }
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
            match self.focus_component {
                // 当聚焦于输入框时，处理输入
                MainFocusComponent::InputArea => self.handle_input_key_evnet(key, tx),
                // 当聚焦于退出按钮时，处理退出
                MainFocusComponent::ExitButton => self.handle_exit_button_key_event(key),
                // 当聚焦于退出按钮时，处理进入设置菜单
                MainFocusComponent::SettingButton => self.handle_setting_button_key_event(key),
            }
        }
    }

    /// 当聚焦于输入框时，处理输入
    fn handle_input_key_evnet(&mut self, key: event::KeyEvent, tx: mpsc::Sender<ChatType>) {
        // 如果是除 Tab 键外其他任意按键事件，则清空错误提示消息
        if key.code != event::KeyCode::Tab && self.response_status != ResponseStatus::None {
            self.response_status = ResponseStatus::None;
        }
        match key.code {
            event::KeyCode::Backspace => self.input_field_component.delete_pre_char(),
            event::KeyCode::Enter => self.submit_message(tx),
            event::KeyCode::Left => self
                .input_field_component
                .move_cursor_left(self.input_field_component.get_current_char()),
            event::KeyCode::Right => self
                .input_field_component
                .move_cursor_right(self.input_field_component.get_next_char()),
            event::KeyCode::Up => self.up(),
            event::KeyCode::Down => self.down(),
            event::KeyCode::Home => self.input_field_component.home_of_cursor(),
            event::KeyCode::End => self.input_field_component.end_of_cursor(),
            event::KeyCode::Delete => self.input_field_component.delete_suf_char(),
            event::KeyCode::Char(x) => self.input_field_component.enter_char(x),
            event::KeyCode::Tab => self.focus_component = MainFocusComponent::ExitButton,
            event::KeyCode::Esc => self.should_exit = true,
            event::KeyCode::F(4) => self.set_or_clear_image_path(),
            _ => {}
        };
    }

    /// 当聚焦于退出按钮时，处理退出
    fn handle_exit_button_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            event::KeyCode::Enter | event::KeyCode::Esc => self.should_exit = true,
            event::KeyCode::Tab => self.focus_component = MainFocusComponent::SettingButton,
            _ => {}
        };
    }

    /// 当聚焦于退出按钮时，处理进入设置菜单
    fn handle_setting_button_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            event::KeyCode::Esc => self.should_exit = true,
            event::KeyCode::Enter => self.open_setting_menu(),
            event::KeyCode::Tab => self.focus_component = MainFocusComponent::InputArea,
            _ => {}
        };
    }

    /// 进入设置菜单
    fn open_setting_menu(&mut self) {
        self.current_windows = CurrentWindows::SettingWindow(SettingUI::new());
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

    /// 提交消息
    fn submit_message(&mut self, tx: mpsc::Sender<ChatType>) {
        let image_path = self.image_path.clone().unwrap_or_default();
        if !self.input_field_component.get_content().is_empty() {
            if self.gemini.is_none() {
                self.restore_or_new_gemini(Some(self.input_field_component.get_content()));
            } else {
                self.chat_history.push(ChatMessage {
                    success: true,
                    sender: User(image_path.clone()),
                    message: self.input_field_component.get_content(),
                    date_time: Local::now(),
                });
                // 将获取消息标志位置真，发送消息给下一次循环使用
                self.receiving_message = true;
                if image_path.is_empty() {
                    let _ = tx.send(ChatType::Simple {
                        message: self.input_field_component.get_content(),
                    });
                } else {
                    let _ = tx.send(ChatType::Image {
                        message: self.input_field_component.get_content(),
                        image_path,
                    });
                    self.image_path = None;
                }
            }
            self.input_field_component.clear();
            // 滚动到最新的一条消息
            self.scroll_props.scroll_offset = self.max_scroll_offset();
        }
    }

    /// 尝试通过读取环境变量信息初始化 Gemini API
    fn restore_or_new_gemini(&mut self, key: Option<String>) {
        // 尝试读取配置文件
        match read_config() {
            Ok(gemini) => {
                match self.gemini.clone() {
                    Some(gemini_origin) => {
                        // gemini 已经存在，则更新配置信息
                        let mut gemini_new =
                            Gemini::rebuild(gemini.key, gemini.model, gemini_origin.contents, gemini.options);
                        gemini_new.set_system_instruction(gemini.system_instruction.unwrap_or("".into()));
                        self.gemini = Some(gemini_new)
                    }
                    None => {
                        // 读取到配置文件则直接使用配置文件中的 Gemini API
                        let mut gemini_new = Gemini::rebuild(gemini.key, gemini.model, Vec::new(), gemini.options);
                        gemini_new.set_system_instruction(gemini.system_instruction.unwrap_or("".into()));
                        self.gemini = Some(gemini_new)
                    }
                }
            }
            Err(_) => {
                if let Some(key) = key {
                    // 尝试从 key 构造 Gemini API
                    self.init_gemini(key);
                } else if let Ok(key) = std::env::var("GEMINI_KEY") {
                    // 尝试从环境变量中读取密钥
                    self.init_gemini(key);
                }
            }
        }
    }

    /// 设置图片或清除图片路径
    fn set_or_clear_image_path(&mut self) {
        match self.image_path.clone() {
            Some(_) => self.image_path = None,
            None => {
                self.image_path = if self.input_field_component.get_content().is_empty() {
                    None
                } else {
                    let res = Some(self.input_field_component.get_content());
                    self.input_field_component.clear();
                    res
                }
            }
        }
    }

    /// 初始化 Gemini API
    fn init_gemini(&mut self, key: String) {
        let system_instruction = "你是一只猫娘，你每次说话都会在句尾加上喵~ ".to_owned();
        let mut gemini = Gemini::new(key, LanguageModel::Gemini1_5Flash);
        gemini.set_options(GenerationConfig::default());
        gemini.set_system_instruction(system_instruction.clone());
        let data = StoreData {
            key: gemini.key.clone(),
            model: gemini.model.clone(),
            system_instruction: Some(system_instruction),
            options: gemini.options.clone(),
        };
        let _ = save_config(data);
        self.gemini = Some(gemini)
    }

    /// 绘制UI
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        // 计算显示区域宽度
        // - 10 留出左右空白区域
        // -2 文本段落中的左右边框
        // -3 输入框左右两侧头像部分
        // -1 对齐中文文本
        // 如果没有减去这4个宽度，文本可能有显示问题，可以再减去任意宽度，以使得在输出的列表文本右侧留出对应宽度空白
        let chat_area_width = || area.width as usize - 10 - 2 - 3 - 1;
        let [header_area, chat_area, input_area] = Layout::vertical([Length(1), Fill(1), Length(3)]).areas(area);
        self.render_header_area(frame, header_area);
        // 输入区域（底部）
        self.render_input_area(frame, input_area);
        // 聊天记录区域（顶部）
        self.render_chat_area(frame, chat_area, chat_area_width);
    }

    /// 渲染头部区域
    fn render_header_area(&mut self, frame: &mut Frame, header_area: Rect) {
        let [left, center, right] = Layout::horizontal([Length(4), Fill(1), Length(4)]).areas(header_area);

        // 根据是否选中组件变色
        let left_color = if self.focus_component == MainFocusComponent::ExitButton {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        // 根据是否选中组件变色
        let right_color = if self.focus_component == MainFocusComponent::SettingButton {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let left_paragraph = Paragraph::new("EXIT").style(left_color).left_aligned();
        frame.render_widget(left_paragraph, left);
        let right_paragraph = Paragraph::new("SET").style(right_color).right_aligned();
        frame.render_widget(right_paragraph, right);

        let center_paragraph = Paragraph::new("Gemini Chat")
            .style(Style::default().fg(Color::LightBlue))
            .centered();
        frame.render_widget(center_paragraph, center);
    }

    /// 渲染输入区域
    fn render_input_area(&mut self, frame: &mut Frame, input_area: Rect) {
        // 调整输入框宽度
        self.input_field_component
            .set_width_height(input_area.width as usize - 2, 1);
        // 输入区域（底部）
        let input_block_title = if self.gemini.is_none() {
            "Input Key"
        } else {
            "Input Text"
        };
        // 根据图片是否为空设置文本
        let title = if self.blank_image() {
            Title::from("Press F4 Set Image Path")
                .position(TitlePosition::Top)
                .alignment(Alignment::Right)
        } else {
            Title::from(format!(
                "[{}] Press F4 To Clear",
                self.image_path.clone().unwrap_or_default()
            ))
            .position(TitlePosition::Top)
            .alignment(Alignment::Right)
        };
        // 根据是否选中组件变色
        let input_block = if self.focus_component == MainFocusComponent::InputArea {
            Block::default()
                .title(
                    Title::from(input_block_title)
                        .position(TitlePosition::Top)
                        .alignment(Alignment::Left),
                )
                .title(title)
                .border_style(Style::default().fg(Color::Green))
                .borders(Borders::ALL)
        } else {
            Block::bordered()
                .title(
                    Title::from(input_block_title)
                        .position(TitlePosition::Top)
                        .alignment(Alignment::Left),
                )
                .title(title)
                .border_style(Style::default().fg(Color::White))
                .borders(Borders::ALL)
        };
        // 输入框内容
        let text = self.input_field_component.should_show_text();

        let input_paragraph = if self.receiving_message {
            // 如果处于等待消息接收状态，则显示等待提示
            Paragraph::new("Receiving message...")
                .block(input_block)
                .style(Style::default().fg(Color::Cyan))
        } else if let ResponseStatus::Failed(msg) = &self.response_status {
            // 接收响应消息失败
            let text = msg.clone();
            Paragraph::new(text)
                .block(input_block)
                .style(Style::default().fg(Color::Red))
        } else {
            Paragraph::new(text)
                .block(input_block)
                .style(Style::default().fg(Color::Yellow))
        };

        frame.render_widget(input_paragraph, input_area);
        if self.focus_component == MainFocusComponent::InputArea {
            let (x, y) = self.input_field_component.get_cursor_position();
            frame.set_cursor_position(CursorPosition::new(
                input_area.x + x as u16 + 1,
                input_area.y + y as u16 + 1,
            ));
        }
    }

    /// 渲染聊天记录区域
    fn render_chat_area<F>(&mut self, frame: &mut Frame, chat_area: Rect, chat_area_width: F)
    where
        F: Fn() -> usize,
    {
        let chat_block = Block::default()
            .border_style(Style::default().fg(Color::DarkGray))
            .borders(Borders::ALL);
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
        // 保存最后一条记录的高度，用于计算滚动条位置
        self.scroll_props.last_chat_history_height = items
            .clone()
            .iter()
            .last()
            .map_or(0, |item| item.message.lines().count() + 3)
            as u16;
        // 计算当前聊天记录区域高度
        self.scroll_props.chat_history_area_height = items
            .clone()
            .iter()
            .map(|item| item.message.lines().count() as u16 + 3)
            .sum();

        let layouts: Vec<Constraint> = items
            .clone()
            .iter()
            .map(|item| {
                if let Split = item.sender {
                    Length(1)
                } else {
                    Length(item.message.lines().count() as u16 + 3)
                }
            })
            .collect();

        let chat_area_x = chat_area.x;
        let chat_area_y = chat_area.y;
        let chat_area_width = chat_area.width;
        let chat_area_height = chat_area.height;

        // 聊天区域高度，如果大于聊天记录区域高度，则显示聊天记录区域高度（可能有问题）TODO
        let height = if chat_area_height > self.scroll_props.chat_history_area_height {
            chat_area_height
        } else {
            // 滚动到最新的一条消息
            self.scroll_props.chat_history_area_height
        };
        // 这块区域将不会被实际渲染，此处 y + 1 为去掉上边框, height - 1 为去掉下边框
        let chat_list_full_area = Rect::new(chat_area_x, chat_area_y + 1, chat_area_width, height - 1);
        let mut chat_list_full_area_buf = Buffer::empty(chat_list_full_area);

        let areas = Layout::vertical(layouts).split(chat_list_full_area);
        for (area, chat_message) in areas.iter().zip(items.iter()) {
            chat_message.clone().render(*area, &mut chat_list_full_area_buf);
        }

        let visible_content = chat_list_full_area_buf
            .content
            .into_iter()
            .skip((chat_area_width * self.scroll_props.scroll_offset) as usize) // 跳过滚动条滚动位置头部的区域
            .take((chat_area_width * (chat_area_height - 2)) as usize); // 取出可见区域的内容，此处 -2 为去掉上边框和下边框（受上面的 y + 1 和 height - 1 影响，此处必须如此）

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

    /// 判断图片路径是否为空
    fn blank_image(&self) -> bool {
        if let Some(image_path) = self.image_path.clone() {
            image_path.is_empty()
        } else {
            true
        }
    }
}
