use ratatui::{
    crossterm::event::{self, Event, KeyEventKind, KeyModifiers},
    layout::{
        Constraint::{self, Fill, Length, Min},
        Layout, Position, Rect,
    },
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use strum::{EnumCount, FromRepr};

use crate::utils::store_utils::{read_config, save_config, StoreData};

use super::{
    component::input::{input_trait::InputTextComponent, text_area::TextArea},
    TextField,
};

/// 窗口UI
pub struct SettingUI {
    /// 选中的输入框
    select_input_field: InputIdentifier,
    /// 组件列表，先纵向再横向排列
    components: Vec<(Constraint, Vec<SettingComponent>)>,
    /// 修改后的配置数据
    data: StoreData,
    /// 是否需要更新配置标志位
    pub update: bool,
    /// 是否应该退出程序
    pub should_exit: bool,
}

pub struct SettingComponent {
    /// 标识符
    identifier: InputIdentifier,
    /// 提示文本
    label: String,
    /// 布局属性
    layout: Constraint,
    // 输入框组件
    input_component: Box<dyn InputTextComponent>,
}
/// 组件标识符枚举
#[derive(Clone, EnumCount, FromRepr, PartialEq, Eq)]
pub enum InputIdentifier {
    Model,
    Key,
    SystemInstruction,
    ResponseMineType,
    MaxOutputTokens,
    Temperature,
    TopP,
    TopK,
}

impl SettingUI {
    /// 启动此窗口UI
    pub fn new() -> Self {
        let data = read_config().unwrap_or_default();
        Self {
            select_input_field: InputIdentifier::SystemInstruction,
            update: false,
            data: data.clone(),
            should_exit: false,
            components: vec![
                (
                    Length(3),
                    vec![
                        SettingComponent {
                            identifier: InputIdentifier::Model,
                            label: "model".into(),
                            layout: Length(30),
                            input_component: Box::new(TextField::new(data.model.to_string())),
                        },
                        SettingComponent {
                            identifier: InputIdentifier::Key,
                            label: "key".into(),
                            layout: Fill(20),
                            input_component: Box::new(TextField::new(data.key)),
                        },
                    ],
                ),
                (
                    Min(10),
                    vec![SettingComponent {
                        identifier: InputIdentifier::SystemInstruction,
                        label: "system instruction".into(),
                        layout: Fill(1),
                        input_component: Box::new(TextArea::new(data.system_instruction.unwrap_or("".into()))),
                    }],
                ),
                (
                    Length(3),
                    vec![
                        SettingComponent {
                            identifier: InputIdentifier::ResponseMineType,
                            label: "response mine type".into(),
                            layout: Fill(1),
                            input_component: Box::new(TextField::new(
                                data.options.response_mime_type.unwrap_or("".into()),
                            )),
                        },
                        SettingComponent {
                            identifier: InputIdentifier::MaxOutputTokens,
                            label: "max output tokens".into(),
                            layout: Fill(1),
                            input_component: Box::new(TextField::new(
                                data.options.max_output_tokens.unwrap_or(0).to_string(),
                            )),
                        },
                    ],
                ),
                (
                    Length(3),
                    vec![
                        SettingComponent {
                            identifier: InputIdentifier::Temperature,
                            label: "temperature".into(),
                            layout: Fill(1),
                            input_component: Box::new(TextField::new(
                                data.options.temperature.unwrap_or(0.0).to_string(),
                            )),
                        },
                        SettingComponent {
                            identifier: InputIdentifier::TopP,
                            label: "top p".into(),
                            layout: Min(5),
                            input_component: Box::new(TextField::new(data.options.top_p.unwrap_or(0.0).to_string())),
                        },
                        SettingComponent {
                            identifier: InputIdentifier::TopK,
                            label: "top k".into(),
                            layout: Min(5),
                            input_component: Box::new(TextField::new(data.options.top_k.unwrap_or(0).to_string())),
                        },
                    ],
                ),
            ],
        }
    }
    /// 处理用户输入
    pub fn handle_key(&mut self) {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind != KeyEventKind::Press {
                return;
            }
            // 获取当前选中的输入框
            let component = self.get_current_input_field().unwrap();
            match key.code {
                event::KeyCode::Enter => component.input_component.handle_enter_key(),
                event::KeyCode::Tab => self.next_input_field(),
                event::KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => self.save_config(),
                event::KeyCode::F(2) => self.save_config(),
                event::KeyCode::Esc => self.should_exit = true,
                event::KeyCode::Backspace => component.input_component.delete_pre_char(),
                event::KeyCode::Delete => component.input_component.delete_suf_char(),
                event::KeyCode::Left => component
                    .input_component
                    .move_cursor_left(component.input_component.get_current_char()),
                event::KeyCode::Right => component
                    .input_component
                    .move_cursor_right(component.input_component.get_next_char()),
                event::KeyCode::Up => component.input_component.move_cursor_up(),
                event::KeyCode::Down => component.input_component.move_cursor_down(),
                event::KeyCode::Home => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        component.input_component.home_of_multiline()
                    } else {
                        component.input_component.home_of_cursor()
                    }
                }
                event::KeyCode::End => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        component.input_component.end_of_multiline()
                    } else {
                        component.input_component.end_of_cursor()
                    }
                }
                event::KeyCode::Char(x) => component.input_component.enter_char(x),
                _ => {}
            }
        }
    }

    /// 获取当前选中的组件
    fn get_current_input_field(&mut self) -> Option<&mut SettingComponent> {
        for (_, components) in self.components.iter_mut() {
            for component in components.iter_mut() {
                if self.select_input_field == component.identifier {
                    return Some(component);
                }
            }
        }
        None
    }

    /// 切换到下一个输入组件
    fn next_input_field(&mut self) {
        let current = self.select_input_field.clone() as usize;
        let next = (current + 1) % InputIdentifier::COUNT;
        self.select_input_field = InputIdentifier::from_repr(next).unwrap();
    }

    /// 保存当前配置并退出配置窗口
    fn save_config(&mut self) {
        // 遍历所有组件，将其现在显示的值更新到配置中
        for (_, line) in self.components.iter() {
            for component in line.iter() {
                match component.identifier {
                    InputIdentifier::Model => self.data.model = component.input_component.get_content().into(),
                    InputIdentifier::Key => self.data.key = component.input_component.get_content(),
                    InputIdentifier::SystemInstruction => {
                        self.data.system_instruction = Some(component.input_component.get_content())
                    }
                    InputIdentifier::ResponseMineType => {
                        self.data.options.response_mime_type = Some(component.input_component.get_content())
                    }
                    InputIdentifier::MaxOutputTokens => {
                        self.data.options.max_output_tokens =
                            Some(component.input_component.get_content().parse().unwrap_or(0))
                    }
                    InputIdentifier::Temperature => {
                        self.data.options.temperature =
                            Some(component.input_component.get_content().parse().unwrap_or(0.0))
                    }
                    InputIdentifier::TopP => {
                        self.data.options.top_p = Some(component.input_component.get_content().parse().unwrap_or(0.0))
                    }
                    InputIdentifier::TopK => {
                        self.data.options.top_k = Some(component.input_component.get_content().parse().unwrap_or(0))
                    }
                }
            }
        }
        save_config(self.data.clone()).unwrap();
        self.update = true;
        self.should_exit = true;
    }

    /// 绘制配置窗口UI
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [header_area, content_area] = Layout::vertical([Length(1), Fill(1)]).areas(area);
        self.render_header_area(frame, header_area);
        self.render_content_area(frame, content_area);
    }

    /// 绘制配置窗口头部区域
    fn render_header_area(&mut self, frame: &mut Frame, header_area: Rect) {
        let [left, center, right] = Layout::horizontal([Length(9), Fill(1), Length(9)]).areas(header_area);

        let left_paragraph = Paragraph::new("EXIT(ESC)").style(Color::Red).left_aligned();
        frame.render_widget(left_paragraph, left);
        let right_paragraph = Paragraph::new("SAVE(F2)").style(Color::Green).right_aligned();
        frame.render_widget(right_paragraph, right);

        let center_paragraph = Paragraph::new("System Setting")
            .style(Style::default().fg(Color::LightBlue))
            .centered();
        frame.render_widget(center_paragraph, center);
    }

    /// 绘制配置窗口内容区域
    fn render_content_area(&mut self, frame: &mut Frame, content_area: Rect) {
        let v_list: Vec<Constraint> = self.components.iter().map(|x| x.0).collect();
        let areas = Layout::vertical(v_list).split(content_area);
        for (i, (_, components)) in self.components.iter_mut().enumerate() {
            let h_list: Vec<Constraint> = components.iter().map(|x| x.layout).collect();
            let area = areas.clone()[i];
            let h_areas = Layout::horizontal(h_list).split(area);
            for (j, component) in components.iter_mut().enumerate() {
                let input_area = h_areas.clone()[j];
                // 设置输入框高度
                let height = (input_area.height as usize).saturating_sub(2);
                // 设置输入框宽度
                let width = (input_area.width as usize).saturating_sub(2);
                component.input_component.set_width_height(width, height);
                // 预设输入框边框颜色，当输入框被选中时显示为绿色，否则显示为白色
                let block_style = if self.select_input_field == component.identifier {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                // 预设输入框边框
                let block = Block::default()
                    .title(component.label.as_str())
                    .style(block_style)
                    .borders(Borders::ALL);
                let input_paragraph = Paragraph::new(component.input_component.should_show_text())
                    .block(block)
                    .wrap(Wrap { trim: false })
                    .style(Style::default().fg(Color::Yellow));
                frame.render_widget(input_paragraph, input_area);
                if self.select_input_field == component.identifier {
                    let (x, y) = component.input_component.get_cursor_position();
                    frame.set_cursor_position(Position::new(input_area.x + x as u16 + 1, input_area.y + y as u16 + 1));
                }
            }
        }
    }
}
