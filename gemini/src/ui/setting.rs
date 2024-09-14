use gemini_api::model::blocking::Gemini;
use ratatui::{
    crossterm::event::{self, Event, KeyEventKind},
    layout::{
        Constraint::{self, Fill, Length, Min},
        Layout, Position, Rect,
    },
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::store::{read_config, save_config};

use super::{component::AllSettingComponents, props::InputFieldCursorNeed, InputFieldProps};

/// 窗口UI
pub struct SettingUI {
    /// 选中的输入框
    select_input_field: AllSettingComponents,
    /// 组件列表，先纵向再横向排列
    components: Vec<(Constraint, Vec<SettingComponent>)>,
    conifg: Gemini,
    /// 是否需要更新配置标志位
    pub update: bool,
    /// 是否应该退出程序
    pub should_exit: bool,
}

pub struct SettingComponent {
    /// 标识符
    identifiers: AllSettingComponents,
    /// 提示文本
    label: String,
    /// 布局属性
    layout: Constraint,
    /// 指针位置，光标指向输入字符串中第几位
    input_area_props: InputFieldProps,
}

impl SettingUI {
    /// 启动此窗口UI
    pub fn new() -> Self {
        let config = read_config().unwrap_or_default();
        let mut ui = Self {
            select_input_field: AllSettingComponents::SystemInstruction,
            update: false,
            conifg: config.clone(),
            should_exit: false,
            components: vec![
                (
                    Length(3),
                    vec![
                        SettingComponent {
                            identifiers: AllSettingComponents::Model,
                            label: "model".into(),
                            layout: Length(30),
                            input_area_props: InputFieldProps {
                                input_buffer: config.model.to_string(),
                                ..Default::default()
                            },
                        },
                        SettingComponent {
                            identifiers: AllSettingComponents::Key,
                            label: "key".into(),
                            layout: Fill(20),
                            input_area_props: InputFieldProps {
                                input_buffer: config.key,
                                ..Default::default()
                            },
                        },
                    ],
                ),
                (
                    Min(10),
                    vec![SettingComponent {
                        identifiers: AllSettingComponents::SystemInstruction,
                        label: "system instruction".into(),
                        layout: Fill(1),
                        input_area_props: InputFieldProps {
                            input_buffer: config.system_instruction.unwrap_or("".into()),
                            ..Default::default()
                        },
                    }],
                ),
                (
                    Length(3),
                    vec![
                        SettingComponent {
                            identifiers: AllSettingComponents::ResponseMineType,
                            label: "response mine type".into(),
                            layout: Fill(1),
                            input_area_props: InputFieldProps {
                                input_buffer: config.options.response_mime_type.unwrap_or("".into()),
                                ..Default::default()
                            },
                        },
                        SettingComponent {
                            identifiers: AllSettingComponents::MaxOutputTokens,
                            label: "max output tokens".into(),
                            layout: Fill(1),
                            input_area_props: InputFieldProps {
                                input_buffer: config.options.max_output_tokens.unwrap_or(0).to_string(),
                                ..Default::default()
                            },
                        },
                    ],
                ),
                (
                    Length(3),
                    vec![
                        SettingComponent {
                            identifiers: AllSettingComponents::Temperature,
                            label: "temperature".into(),
                            layout: Fill(1),
                            input_area_props: InputFieldProps {
                                input_buffer: config.options.temperature.unwrap_or(0.0).to_string(),
                                ..Default::default()
                            },
                        },
                        SettingComponent {
                            identifiers: AllSettingComponents::TopP,
                            label: "top p".into(),
                            layout: Min(5),
                            input_area_props: InputFieldProps {
                                input_buffer: config.options.top_p.unwrap_or(0.0).to_string(),
                                ..Default::default()
                            },
                        },
                        SettingComponent {
                            identifiers: AllSettingComponents::TopK,
                            label: "top k".into(),
                            layout: Min(5),
                            input_area_props: InputFieldProps {
                                input_buffer: config.options.top_k.unwrap_or(0).to_string(),
                                ..Default::default()
                            },
                        },
                    ],
                ),
            ],
        };

        // 设置所有输入框组件光标位置
        for (_, components) in ui.components.iter_mut() {
            for component in components.iter_mut() {
                component.input_area_props.end_of_cursor();
            }
        }

        ui
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
                event::KeyCode::Backspace => component.input_area_props.delete_pre_char(),
                event::KeyCode::Delete => component.input_area_props.delete_suf_char(),
                event::KeyCode::Left => component
                    .input_area_props
                    .move_cursor_left(component.input_area_props.get_current_char()),
                event::KeyCode::Right => component
                    .input_area_props
                    .move_cursor_right(component.input_area_props.get_next_char()),
                event::KeyCode::Home => component.input_area_props.reset_cursor(),
                event::KeyCode::End => component.input_area_props.end_of_cursor(),
                event::KeyCode::Char(x) => component.input_area_props.enter_char(x),
                event::KeyCode::Enter => component.input_area_props.handle_enter_key(),
                event::KeyCode::Tab => self.next_input_field(),
                event::KeyCode::F(2) => self.save_config(),
                event::KeyCode::Esc => self.should_exit = true,
                _ => {}
            }
        }
    }

    /// 获取当前选中的组件
    fn get_current_input_field(&mut self) -> Option<&mut SettingComponent> {
        for (_, components) in self.components.iter_mut() {
            for component in components.iter_mut() {
                if self.select_input_field == component.identifiers {
                    return Some(component);
                }
            }
        }
        None
    }

    /// 切换到下一个输入框
    fn next_input_field(&mut self) {
        let current = self.select_input_field.clone() as i32;
        let next = (current + 1) % 8;
        self.select_input_field = next.try_into().unwrap();
    }

    /// 保存配置并退出窗口UI
    fn save_config(&mut self) {
        // 遍历所有组件，将其现在显示的值更新到配置中
        for (_, line) in self.components.iter() {
            for component in line.iter() {
                match component.identifiers {
                    AllSettingComponents::Model => {
                        self.conifg.model = component.input_area_props.input_buffer.clone().into()
                    }
                    AllSettingComponents::Key => self.conifg.key = component.input_area_props.input_buffer.clone(),
                    AllSettingComponents::SystemInstruction => {
                        self.conifg.system_instruction = Some(component.input_area_props.input_buffer.clone())
                    }
                    AllSettingComponents::ResponseMineType => {
                        self.conifg.options.response_mime_type = Some(component.input_area_props.input_buffer.clone())
                    }
                    AllSettingComponents::MaxOutputTokens => {
                        self.conifg.options.max_output_tokens =
                            Some(component.input_area_props.input_buffer.clone().parse().unwrap_or(0))
                    }
                    AllSettingComponents::Temperature => {
                        self.conifg.options.temperature =
                            Some(component.input_area_props.input_buffer.clone().parse().unwrap_or(0.0))
                    }
                    AllSettingComponents::TopP => {
                        self.conifg.options.top_p =
                            Some(component.input_area_props.input_buffer.clone().parse().unwrap_or(0.0))
                    }
                    AllSettingComponents::TopK => {
                        self.conifg.options.top_k =
                            Some(component.input_area_props.input_buffer.clone().parse().unwrap_or(0))
                    }
                }
            }
        }
        save_config(self.conifg.to_owned()).unwrap();
        self.update = true;
        self.should_exit = true;
    }

    /// 绘制窗口UI
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [header_area, content_area] = Layout::vertical([Length(1), Fill(1)]).areas(area);
        self.render_header_area(frame, header_area);
        self.render_content_area(frame, content_area);
    }

    /// 绘制头部区域
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

    /// 绘制内容区域
    fn render_content_area(&mut self, frame: &mut Frame, content_area: Rect) {
        let v_list: Vec<Constraint> = self.components.iter().map(|x| x.0).collect();
        let areas = Layout::vertical(v_list).split(content_area);
        for (i, (_, components)) in self.components.iter_mut().enumerate() {
            let h_list: Vec<Constraint> = components.iter().map(|x| x.layout).collect();
            let area = areas.clone()[i];
            let h_areas = Layout::horizontal(h_list).split(area);
            for (j, component) in components.iter_mut().enumerate() {
                let input_area = h_areas.clone()[j];
                component.input_area_props.height = (input_area.height as usize).saturating_sub(2);
                component.input_area_props.width = (input_area.width as usize).saturating_sub(2);
                let block_style = if self.select_input_field == component.identifiers {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                let block = Block::default()
                    .title(component.label.as_str())
                    .style(block_style)
                    .borders(Borders::ALL);
                let input_paragraph = Paragraph::new(component.input_area_props.should_show_text())
                    .block(block)
                    .style(Style::default().fg(Color::Yellow));
                frame.render_widget(input_paragraph, input_area);
                if self.select_input_field == component.identifiers {
                    frame.set_cursor_position(Position::new(
                        input_area.x + component.input_area_props.cursor_position_x as u16 + 1,
                        input_area.y + component.input_area_props.cursor_position_y as u16 + 1,
                    ));
                }
            }
        }
    }
}
