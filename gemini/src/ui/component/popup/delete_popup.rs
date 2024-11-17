use ratatui::{
    layout::{
        Constraint::{Fill, Length},
        Layout, Rect,
    },
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use strum::{EnumCount, FromRepr};

#[derive(Clone)]
pub struct DeletePopup {
    // 提示文本
    pub title: String,
    // 当前选中的按钮
    pub selected_button: ButtonType,
    // 宽度
    pub width: usize,
    // 高度
    pub height: usize,
    // 边框颜色
    pub border_color: Color,
    // 按钮选中的背景色
    pub button_selected_bg_color: Color,
}

impl Default for DeletePopup {
    fn default() -> Self {
        Self {
            title: "Confirm Deletion".into(),
            selected_button: Default::default(),
            width: 30,
            height: 5,
            border_color: Color::Blue,
            button_selected_bg_color: Color::Green,
        }
    }
}

#[derive(Default, Clone, EnumCount, FromRepr)]
pub enum ButtonType {
    #[default]
    Confirm,
    Cancel,
}

impl DeletePopup {
    // 下一个按钮
    pub fn next_button(&mut self) {
        let current = self.selected_button.clone() as usize;
        let next = (current + 1) % ButtonType::COUNT;
        self.selected_button = ButtonType::from_repr(next).unwrap();
    }

    // 按下按钮，返回是否确认删除
    pub fn press(&self) -> bool {
        matches!(self.selected_button, ButtonType::Confirm)
    }
}

impl DeletePopup {
    pub fn draw(self, frame: &mut Frame, area: Rect) {
        // 先清空弹窗区域内容
        frame.render_widget(Clear, area);

        let title = self.title;
        let [_, title_area, split_area, button_area, _] =
            Layout::vertical([Length(1), Fill(1), Length(1), Length(1), Length(1)]).areas(area);
        // 渲染标题区域
        let title_paragraph = Paragraph::new(format!(" {} ", title)).centered();
        frame.render_widget(title_paragraph, title_area);
        // 渲染分割线
        let split_block = Block::default().borders(Borders::ALL).border_style(Color::Gray);
        frame.render_widget(split_block, split_area);
        // 渲染按钮区域
        let [_, confirm_area, _, cancel_area, _] =
            Layout::horizontal([Length(1), Fill(1), Length(1), Fill(1), Length(1)]).areas(button_area);
        let (confirm_button_style, cancel_button_style) = match self.selected_button {
            ButtonType::Confirm => (
                Style::default().fg(Color::White).bg(self.button_selected_bg_color),
                Style::default().fg(Color::White),
            ),
            ButtonType::Cancel => (
                Style::default().fg(Color::White),
                Style::default().fg(Color::White).bg(self.button_selected_bg_color),
            ),
        };
        // 确认按钮
        let confirm_button = Paragraph::new("Confirm").style(confirm_button_style).centered();
        frame.render_widget(confirm_button, confirm_area);
        // 取消按钮
        let cancel_button = Paragraph::new("Cancel").style(cancel_button_style).centered();
        frame.render_widget(cancel_button, cancel_area);
        // 边框
        let border_block = Block::default().style(self.border_color).borders(Borders::ALL);
        frame.render_widget(border_block, area);
    }
}
