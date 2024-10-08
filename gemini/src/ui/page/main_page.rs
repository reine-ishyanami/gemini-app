use std::sync::mpsc;

use super::setting_page::SettingUI;
use anyhow::Result;
use chrono::Local;
use component::input::{input_trait::InputTextComponent, text_field::TextField};
use component::popup::delete_popup::DeletePopup;
use component::scroll::chat_item_list::ChatItemListScrollProps;
use component::scroll::chat_show::ChatShowScrollProps;
use gemini_api::body::request::GenerationConfig;
use gemini_api::body::{Content, Part, Role};
use gemini_api::model::blocking::Gemini;
use gemini_api::param::LanguageModel;
use gemini_api::utils::image::blocking::get_image_type_and_base64_string;
use ratatui::layout::Position as CursorPosition;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::block::{Position as TitlePosition, Title};
use ratatui::widgets::{Clear, Paragraph, Widget};
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
use strum::{EnumCount, FromRepr};

use crate::model::view::ChatMessage;
use crate::model::view::Sender::{Bot, Never, User};
use crate::ui::component;
use crate::utils::db_utils::{create_table, generate_unique_id, save_conversation};
use crate::utils::image_utils::{cache_image, read_image_cache};
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
    /// Gemini API
    gemini: Option<Gemini>,
    /// 当前聚焦的组件
    focus_component: MainFocusComponent,
    /// 输入区域组件
    input_field_component: TextField,
    /// 当前窗口
    current_windows: CurrentWindows,
    /// 图片路径
    image_path: Option<String>,
    /// 侧边栏是否显示
    sidebar_show: bool,
    /// 对话标题内容
    title: String,
    /// 对话 id
    conversation_id: String,
    chat_item_list: ChatItemListScrollProps,
    chat_show: ChatShowScrollProps,
}
/// 窗口枚举
#[derive(Default)]
pub enum CurrentWindows {
    #[default]
    This,
    SettingWindow(SettingUI),
}

/// 当前聚焦组件
#[derive(Default, Clone, EnumCount, FromRepr)]
pub enum MainFocusComponent {
    /// 输入框
    #[default]
    InputField,
    /// 新建聊天按钮
    NewChatButton,
    /// 聊天记录列表
    ChatItemList,
    /// 设置按钮
    SettingButton,
    /// 聊天内容显示区域
    ChatShow,
}

/// 响应状态
#[derive(Default)]
pub enum ResponseStatus {
    #[default]
    None,
    /// 接收响应消息失败，提供错误信息
    Failed(String),
}

enum ChatType {
    Simple { message: String },
    Image { message: String, image_path: String },
}

impl UI {
    /// 启动UI
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // 与数据库建立连接
        create_table()?;
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

    fn max_scroll_offset(&self) -> u16 {
        self.chat_show.chat_history_area_height
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
        let system_instruction = String::new();
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

    /// 通过纯净的 Gemini API 获取对话摘要
    fn summary_by_gemini(&self, message: String) -> String {
        let gemini = self.gemini.clone().unwrap();
        let mut pure_gemini = Gemini::new(gemini.key, LanguageModel::Gemini1_5Flash);
        pure_gemini.set_options(gemini.options.clone());
        pure_gemini.set_system_instruction("请给我概括一下这段文字内容，不包含任意标点符号，不大于15字。".into());
        pure_gemini.chat_once(message).unwrap_or_default()
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

/// 渲染 UI
impl UI {
    /// 侧边栏宽度
    const SIDEBAR_WIDTH: u16 = 30;

    /// 绘制UI
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        // 左侧宽度
        if self.sidebar_show {
            let [left_area, right_area] = Layout::horizontal([Length(Self::SIDEBAR_WIDTH), Fill(1)]).areas(area);
            self.render_left_area(frame, left_area);
            self.render_right_area(frame, right_area);
        } else {
            self.render_right_area(frame, area);
        }
        // 是否显示删除弹窗
        if let Some(popup) = self.chat_item_list.popup_delete_confirm_dialog.clone() {
            let x = (area.width - popup.width as u16) / 2;
            let y = (area.height - popup.height as u16) / 2;
            let rect = Rect::new(x, y, popup.width as u16, popup.height as u16);
            // 先清空弹窗区域内容
            frame.render_widget(Clear, rect);
            let buf = frame.buffer_mut();
            popup.render(rect, buf);
        }
    }

    /// 渲染左侧区域
    fn render_left_area(&mut self, frame: &mut Frame, left_area: Rect) {
        let [title_area, new_chat_area, list_area, setting_area] =
            Layout::vertical([Length(1), Length(3), Fill(1), Length(3)]).areas(left_area);
        // 标题
        let title_paragraph = Paragraph::new("History")
            .style(Style::default().fg(Color::LightMagenta))
            .centered();
        frame.render_widget(title_paragraph, title_area);
        // 新建聊天按钮
        let new_chat_button_block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(
            if matches!(self.focus_component, MainFocusComponent::NewChatButton) {
                Color::Green
            } else {
                Color::White
            },
        ));
        let new_chat_button_text = Paragraph::new("New Chat")
            .style(Style::default().fg(Color::LightBlue))
            .block(new_chat_button_block)
            .centered();
        frame.render_widget(new_chat_button_text, new_chat_area);
        // 聊天列表
        let is_focused = matches!(self.focus_component, MainFocusComponent::ChatItemList);
        self.chat_item_list.draw(frame, list_area, is_focused);
        // 设置按钮
        let setting_button_block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(
            if matches!(self.focus_component, MainFocusComponent::SettingButton) {
                Color::Green
            } else {
                Color::White
            },
        ));
        let setting_button_text = Paragraph::new("Setting")
            .style(Style::default().fg(Color::LightBlue))
            .block(setting_button_block)
            .centered();
        frame.render_widget(setting_button_text, setting_area);
    }

    /// 渲染右侧区域
    fn render_right_area(&mut self, frame: &mut Frame, right_area: Rect) {
        // 计算显示区域宽度
        // - 10 留出左右空白区域
        // -2 文本段落中的左右边框
        // -3 输入框左右两侧头像部分
        // -1 对齐中文文本
        // 如果没有减去这4个宽度，文本可能有显示问题，可以再减去任意宽度，以使得在输出的列表文本右侧留出对应宽度空白
        let chat_area_width = || right_area.width as usize - 10 - 2 - 3 - 1;
        let [header_area, chat_area, input_area] = Layout::vertical([Length(1), Fill(1), Length(3)]).areas(right_area);
        self.render_header_area(frame, header_area);
        // 输入区域（底部）
        self.render_input_area(frame, input_area);
        // 聊天记录区域（顶部）
        self.render_chat_area(frame, chat_area, chat_area_width);
    }

    /// 渲染头部区域
    fn render_header_area(&mut self, frame: &mut Frame, header_area: Rect) {
        let [tip_area, title_area] = Layout::horizontal([Length(5), Fill(1)]).areas(header_area);
        let tip_text = if self.sidebar_show { "< F3" } else { "> F3" };
        let tip_paragraph = Paragraph::new(tip_text)
            .style(Style::default().fg(Color::Red))
            .centered();
        frame.render_widget(tip_paragraph, tip_area);

        let title = if self.conversation_id.is_empty() {
            "Gemini Chat"
        } else {
            self.title.as_str()
        };
        let title_paragraph = Paragraph::new(title)
            .style(Style::default().fg(Color::LightBlue))
            .centered();
        frame.render_widget(title_paragraph, title_area);
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
        let input_block = Block::bordered()
            .title(
                Title::from(input_block_title)
                    .position(TitlePosition::Top)
                    .alignment(Alignment::Left),
            )
            .title(title)
            .borders(Borders::ALL)
            .border_style(
                Style::default().fg(if matches!(self.focus_component, MainFocusComponent::InputField) {
                    Color::Green
                } else {
                    Color::White
                }),
            );
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
        if matches!(self.focus_component, MainFocusComponent::InputField) {
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
        let is_focused = matches!(self.focus_component, MainFocusComponent::ChatShow);
        self.chat_show.draw(frame, chat_area, chat_area_width, is_focused);
    }
}

/// 处理输入事件
impl UI {
    /// 处理按键事件
    fn handle_key(&mut self, tx: mpsc::Sender<ChatType>, rx: &mpsc::Receiver<ChatType>) {
        // 如果接收消息位为真
        if self.receiving_message {
            // 阻塞接收消息
            if let Ok(request) = rx.recv() {
                match request {
                    ChatType::Simple { message } => {
                        match self.gemini.as_mut().unwrap().chat_conversation(message) {
                            // 成功接收响应消息后，将响应消息封装后加入到消息列表以供展示
                            Ok(response) => {
                                if self.conversation_id.is_empty() {
                                    // 总结标题
                                    let title = self.summary_by_gemini(response.clone());
                                    self.title = title;
                                    self.conversation_id = generate_unique_id();
                                }
                                // 推送用户发送的消息保存到数据库
                                let chat_message = self.chat_show.chat_history.pop().unwrap();
                                let _ = save_conversation(
                                    self.conversation_id.clone(),
                                    self.title.clone(),
                                    chat_message.clone(),
                                );
                                self.chat_show.chat_history.push(chat_message);
                                let response = response.replace("\n\n", "\n");
                                let response = if response.ends_with("\n") {
                                    response[..response.len() - 1].to_owned()
                                } else {
                                    response
                                };
                                let chat_message = ChatMessage {
                                    success: true,
                                    sender: Bot,
                                    message: response,
                                    date_time: Local::now(),
                                };
                                // 推送接收到的消息保存到数据库
                                let _ = save_conversation(
                                    self.conversation_id.clone(),
                                    self.title.clone(),
                                    chat_message.clone(),
                                );
                                self.chat_show.chat_history.push(chat_message);
                            }
                            // 接收响应消息失败，将响应状态位改为失败，并提供错误信息
                            Err(e) => {
                                if let Some(msg) = e.downcast_ref::<String>() {
                                    self.response_status = ResponseStatus::Failed(msg.clone());
                                } else {
                                    self.response_status = ResponseStatus::Failed("Unknown Error".into());
                                }
                                // 将最后一条消息状态修改为失败
                                let mut chat_message = self.chat_show.chat_history.pop().unwrap();
                                chat_message.success = false;
                                self.chat_show.chat_history.push(chat_message);
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
                                if self.conversation_id.is_empty() {
                                    // 总结标题
                                    let title = self.summary_by_gemini(response.clone());
                                    self.title = title;
                                    self.conversation_id = generate_unique_id();
                                }
                                // 推送用户发送的消息保存到数据库
                                let chat_message = self.chat_show.chat_history.pop().unwrap();
                                let _ = save_conversation(
                                    self.conversation_id.clone(),
                                    self.title.clone(),
                                    chat_message.clone(),
                                );
                                self.chat_show.chat_history.push(chat_message);
                                let response = response.replace("\n\n", "\n");
                                let response = if response.ends_with("\n") {
                                    response[..response.len() - 1].to_owned()
                                } else {
                                    response
                                };
                                let chat_message = ChatMessage {
                                    success: true,
                                    sender: Bot,
                                    message: response,
                                    date_time: Local::now(),
                                };
                                // 推送接收到的消息保存到数据库
                                let _ = save_conversation(
                                    self.conversation_id.clone(),
                                    self.title.clone(),
                                    chat_message.clone(),
                                );
                                self.chat_show.chat_history.push(chat_message);
                            }
                            // 接收响应消息失败，将响应状态位改为失败，并提供错误信息
                            Err(e) => {
                                if let Some(msg) = e.downcast_ref::<String>() {
                                    self.response_status = ResponseStatus::Failed(msg.clone());
                                } else {
                                    self.response_status = ResponseStatus::Failed("Unknown Error".into());
                                }
                                // 将最后一条消息状态修改为失败
                                let mut chat_message = self.chat_show.chat_history.pop().unwrap();
                                chat_message.success = false;
                                self.chat_show.chat_history.push(chat_message);
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
                MainFocusComponent::InputField => self.handle_input_key_event(key, tx),
                // 当聚焦于新建聊天按钮时，处理输入
                MainFocusComponent::NewChatButton => self.handle_new_chat_key_event(key),
                // 当聚焦于聊天列表时，处理输入
                MainFocusComponent::ChatItemList => self.handle_chat_list_key_event(key),
                // 当聚焦于设置按钮时，处理输入
                MainFocusComponent::SettingButton => self.handle_setting_button_key_event(key),
                // 当聚焦于聊天内容显示区域时，处理输入
                MainFocusComponent::ChatShow => self.handle_chat_show_key_event(key),
            }
        }
    }

    /// 当聚焦于输入框时，处理输入
    fn handle_input_key_event(&mut self, key: event::KeyEvent, tx: mpsc::Sender<ChatType>) {
        // 如果是除 Tab 键外其他任意按键事件，则清空错误提示消息
        if key.code != event::KeyCode::Tab && !matches!(self.response_status, ResponseStatus::None) {
            self.response_status = ResponseStatus::None;
        }
        match key.code {
            event::KeyCode::Esc => self.should_exit = true,
            event::KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.show_and_hide_sidebar()
            }
            event::KeyCode::F(3) => self.show_and_hide_sidebar(),
            event::KeyCode::Tab => self.next_component(),
            event::KeyCode::Char('i') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.set_or_clear_image_path()
            }
            event::KeyCode::F(4) => self.set_or_clear_image_path(),
            event::KeyCode::Backspace => self.input_field_component.delete_pre_char(),
            event::KeyCode::Enter => self.submit_message(tx),
            event::KeyCode::Left => self
                .input_field_component
                .move_cursor_left(self.input_field_component.get_current_char()),
            event::KeyCode::Right => self
                .input_field_component
                .move_cursor_right(self.input_field_component.get_next_char()),
            event::KeyCode::Home => self.input_field_component.home_of_cursor(),
            event::KeyCode::End => self.input_field_component.end_of_cursor(),
            event::KeyCode::Delete => self.input_field_component.delete_suf_char(),
            event::KeyCode::Char(x) => self.input_field_component.enter_char(x),
            _ => {}
        };
    }

    /// 当聚焦于新建聊天按钮时，处理输入
    fn handle_new_chat_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            event::KeyCode::Esc => self.should_exit = true,
            event::KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.show_and_hide_sidebar()
            }
            event::KeyCode::F(3) => self.show_and_hide_sidebar(),
            event::KeyCode::Tab => self.next_component(),
            event::KeyCode::Enter => self.new_conversation(),
            _ => {}
        };
    }

    /// 创建一个新的对话
    fn new_conversation(&mut self) {
        self.receiving_message = false;
        self.response_status = ResponseStatus::None;
        if let Some(gemini) = self.gemini.clone() {
            let mut gemini_new = Gemini::rebuild(gemini.key, gemini.model, Vec::new(), gemini.options);
            gemini_new.set_system_instruction(gemini.system_instruction.unwrap_or("".into()));
            self.gemini = Some(gemini_new);
        };
        self.focus_component = MainFocusComponent::InputField;
        self.input_field_component.clear();
        self.image_path = None;
        self.title = "".into();
        self.conversation_id = "".into();
        self.chat_show = ChatShowScrollProps::default();
    }

    /// 当聚焦于聊天列表时，处理输入
    fn handle_chat_list_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            event::KeyCode::Esc => self.should_exit = true,
            event::KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.show_and_hide_sidebar()
            }
            event::KeyCode::F(3) => self.show_and_hide_sidebar(),
            event::KeyCode::Enter => {
                // 如果此时有确认删除的弹窗，则处理弹窗
                if let Some(popup) = self.chat_item_list.popup_delete_confirm_dialog.clone() {
                    let confirm = popup.press();
                    // 如果确认删除，则删除
                    if confirm {
                        let deleted_id = self.chat_item_list.delete_item();
                        // 如果删除的是当前聊天，则重新创建新的聊天
                        if deleted_id == self.conversation_id {
                            self.new_conversation();
                        }
                    }
                    self.chat_item_list.popup_delete_confirm_dialog = None;
                    return;
                }
                // 否则加载对应选中项的聊天内容列表
                if let Some(conversation) = self.chat_item_list.rebuild() {
                    self.conversation_id = conversation.conversation_id;
                    self.title = conversation.conversation_title;
                    let contents: Vec<Content> = conversation
                        .conversation_records
                        .clone()
                        .iter()
                        .map(|record| {
                            let role = match record.record_sender {
                                User(_) => Some(Role::User),
                                Bot => Some(Role::Model),
                                Never => None,
                            };
                            let mut parts = Vec::new();
                            parts.push(Part::Text(record.record_content.clone()));
                            // 如果包含了图片数据，则添加到 parts 中
                            if let Some(image_record) = record.image_record.clone() {
                                let image_record_id = image_record.image_record_id;
                                // 读取图片缓存数据
                                Self::read_image_data(image_record_id, image_record.image_path, &mut parts);
                            }
                            Content { parts, role }
                        })
                        .collect();
                    // 重新加载 gemini 客户端
                    if let Some(gemini) = self.gemini.clone() {
                        let mut gemini_new = Gemini::rebuild(gemini.key, gemini.model, contents, gemini.options);
                        gemini_new.set_system_instruction(gemini.system_instruction.unwrap_or("".into()));
                        self.gemini = Some(gemini_new);
                    }
                    // 加载聊天记录
                    let chat_history: Vec<ChatMessage> = conversation
                        .conversation_records
                        .clone()
                        .iter()
                        .map(|record| ChatMessage {
                            success: true,
                            message: record.record_content.clone(),
                            sender: record.record_sender.clone(),
                            date_time: record.record_time,
                        })
                        .collect();
                    self.chat_show.chat_history = chat_history;
                    self.focus_component = MainFocusComponent::ChatShow;
                    self.input_field_component.clear();
                    self.image_path = None;
                }
            }
            event::KeyCode::Up => self.chat_item_list.prev_item(),
            event::KeyCode::Down => self.chat_item_list.next_item(),
            event::KeyCode::Delete => {
                // 弹窗提示
                self.chat_item_list.popup_delete_confirm_dialog = Some(DeletePopup::default());
            }
            event::KeyCode::Tab => {
                // 如果此时有确认删除的弹窗，则处理弹窗
                if let Some(ref mut popup) = self.chat_item_list.popup_delete_confirm_dialog {
                    popup.next_button();
                } else {
                    self.next_component();
                }
            }
            _ => {}
        };
    }

    /// 读取图片数据
    fn read_image_data(image_record_id: String, image_path: String, parts: &mut Vec<Part>) {
        // 读取图片缓存数据
        if let Ok((image_type, image_data)) = read_image_cache(image_record_id.clone()) {
            parts.push(Part::InlineData {
                mime_type: image_type,
                data: image_data,
            });
            // 如果读不到缓存的数据，则重新加载图片数据
        } else if let Ok((image_type, image_data)) = get_image_type_and_base64_string(image_path.clone()) {
            // 将图片数据缓存到本地
            let _ = cache_image(image_path, image_record_id);
            parts.push(Part::InlineData {
                mime_type: image_type,
                data: image_data,
            });
        }
    }

    /// 当聚焦于退出按钮时，处理进入设置菜单
    fn handle_setting_button_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            event::KeyCode::Esc => self.should_exit = true,
            event::KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.show_and_hide_sidebar()
            }
            event::KeyCode::F(3) => self.show_and_hide_sidebar(),
            event::KeyCode::Tab => self.next_component(),
            event::KeyCode::Enter => self.open_setting_menu(),
            _ => {}
        };
    }

    /// 当聚焦于聊天内容显示区域时，处理输入
    fn handle_chat_show_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            event::KeyCode::Esc => self.should_exit = true,
            event::KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.show_and_hide_sidebar()
            }
            event::KeyCode::F(3) => self.show_and_hide_sidebar(),
            event::KeyCode::Tab => self.next_component(),
            event::KeyCode::Up => self.up(),
            event::KeyCode::Down => self.down(),
            _ => {}
        }
    }

    /// 展示或隐藏侧边栏
    fn show_and_hide_sidebar(&mut self) {
        // 如果侧边栏已经显示，且当前聚焦组件为侧边栏组件，则聚焦到输入框，否则不变
        if self.sidebar_show {
            self.focus_component = match self.focus_component.clone() {
                MainFocusComponent::ChatItemList
                | MainFocusComponent::NewChatButton
                | MainFocusComponent::SettingButton => MainFocusComponent::InputField,
                other => other,
            }
        }
        self.sidebar_show = !self.sidebar_show;
    }

    /// 切换到下一个组件
    fn next_component(&mut self) {
        if self.sidebar_show {
            let current = self.focus_component.clone() as usize;
            let next = (current + 1) % MainFocusComponent::COUNT;
            self.focus_component = MainFocusComponent::from_repr(next).unwrap();
        } else {
            self.focus_component = match self.focus_component {
                MainFocusComponent::InputField => MainFocusComponent::ChatShow,
                MainFocusComponent::ChatShow => MainFocusComponent::InputField,
                _ => MainFocusComponent::InputField,
            }
        }
    }

    /// 进入设置菜单
    fn open_setting_menu(&mut self) {
        self.current_windows = CurrentWindows::SettingWindow(SettingUI::new());
    }

    /// 聊天区域向上滚动
    fn up(&mut self) {
        self.chat_show.scroll_offset = self.chat_show.scroll_offset.saturating_sub(1);
    }

    /// 聊天区域向下滚动
    fn down(&mut self) {
        self.chat_show.scroll_offset = self
            .chat_show
            .scroll_offset
            .saturating_add(1)
            .min(self.max_scroll_offset());
    }

    /// 提交消息
    fn submit_message(&mut self, tx: mpsc::Sender<ChatType>) {
        let image_path = self.image_path.clone().unwrap_or_default();
        if !self.input_field_component.get_content().is_empty() {
            if self.gemini.is_none() {
                self.restore_or_new_gemini(Some(self.input_field_component.get_content()));
            } else {
                self.chat_show.chat_history.push(ChatMessage {
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
            self.chat_show.scroll_offset = self.chat_show.chat_history_area_height;
        }
    }
}
