use ratatui::style::Color;
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
