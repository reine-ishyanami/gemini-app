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
}

impl Default for DeletePopup {
    fn default() -> Self {
        Self {
            title: "确认删除".into(),
            selected_button: Default::default(),
            width: 30,
            height: 5,
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
pub enum ButtonType {
    #[default]
    Confirm,
    Cancel,
}

impl DeletePopup {
    // 下一个按钮
    pub fn next_button(&mut self) {
        let current = self.selected_button.clone() as i32;
        let next = (current + 1) % 2;
        self.selected_button = next.try_into().unwrap();
    }

    // 按下按钮，返回是否确认删除
    pub fn press(&self) -> bool {
        self.selected_button == ButtonType::Confirm
    }
}
