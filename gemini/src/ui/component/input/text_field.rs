use crate::utils::char_utils::{c_len, length};

use super::input_trait::InputTextComponent;

/// 单行输入框相关属性
#[derive(Default)]
pub(crate) struct TextField {
    /// 当前指针位置，光标指向输入字符串中第几位
    input_buffer_index: usize,
    /// 最左侧光标索引
    left_index: usize,
    /// 最右侧光标索引
    right_index: usize,
    /// 光标坐标 x，每一个 ASCII 字符占1位，非 ASCII 字符占2位
    /// 如果输入的文本为纯 ASCII 字符，则于 input_buffer_index 相等，
    /// 如果包含非 ASCII 字符，则会比 input_buffer_index 大
    cursor_position_x: usize,
    /// 输入框内容
    input_buffer: String,
    /// 输入框宽度
    width: usize,
    /// 是否已经初始化过指针位置
    align_right: bool,
}

impl InputTextComponent for TextField {
    fn should_show_text(&self) -> String {
        self.sub_input_buffer()
    }

    fn get_cursor_position(&self) -> (usize, usize) {
        // 计算左侧隐藏的宽度
        let left = self.left_index;
        let mut width = 0;
        for index in 0..left {
            if let Some(c) = self.input_buffer.chars().nth(index) {
                width += c_len(c);
            }
        }
        // 坐标减去左侧隐藏的宽度为真实指针坐标
        let x = self.cursor_position_x - width;
        (x.clamp(0, self.width), 0)
    }

    fn end_of_cursor(&mut self) {
        self.input_buffer_index = self.input_buffer.chars().count();
        self.cursor_position_x = length(self.input_buffer.clone());
        self.right_index = self.input_buffer_index;
        self.compact_left_index_by_right_index();
    }

    fn home_of_cursor(&mut self) {
        self.input_buffer_index = 0;
        self.cursor_position_x = 0;
        self.left_index = 0;
        self.compact_right_index_by_left_index();
    }

    fn get_current_char(&self) -> char {
        if self.input_buffer_index == 0 {
            '\0'
        } else {
            self.input_buffer.chars().nth(self.input_buffer_index - 1).unwrap()
        }
    }

    fn get_next_char(&self) -> char {
        self.input_buffer.chars().nth(self.input_buffer_index).unwrap_or('\0')
    }

    fn move_cursor_left(&mut self, c: char) {
        let origin_cursor_index = self.input_buffer_index;
        let cursor_moved_left = self.input_buffer_index.saturating_sub(1);
        self.input_buffer_index = self.clamp_cursor(cursor_moved_left);
        // 如果当前字符不在左右范围内，则左指针左移
        if self.input_buffer_index < self.left_index {
            self.left_index = self.input_buffer_index;
            self.compact_right_index_by_left_index();
        }
        // 光标有变化
        if origin_cursor_index != self.input_buffer_index {
            self.cursor_position_x = self.cursor_position_x.saturating_sub(c_len(c));
        }
    }

    fn move_cursor_right(&mut self, c: char) {
        let origin_cursor_index = self.input_buffer_index;
        // 指针位置指向下一位
        let cursor_moved_right = self.input_buffer_index.saturating_add(1);
        self.input_buffer_index = self.clamp_cursor(cursor_moved_right);
        // 如果当前字符不在左右范围内，则右指针右移
        if self.input_buffer_index > self.right_index {
            self.right_index = self.input_buffer_index;
            self.compact_left_index_by_right_index();
        }
        // 光标有变化
        if origin_cursor_index != self.input_buffer_index {
            self.cursor_position_x = self.cursor_position_x.saturating_add(c_len(c));
        }
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input_buffer.insert(index, new_char);
        self.move_cursor_right(new_char);
    }

    fn byte_index(&self) -> usize {
        self.input_buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.input_buffer_index)
            .unwrap_or(self.input_buffer.len())
    }

    fn delete_pre_char(&mut self) {
        let is_not_cursor_leftmost = self.input_buffer_index != 0;
        if is_not_cursor_leftmost {
            let delete_char = self.get_current_char();
            let current_index = self.input_buffer_index;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input_buffer.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input_buffer.chars().skip(current_index);
            self.input_buffer = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left(delete_char);
            if self.left_index != 0 {
                self.left_index -= 1;
                self.compact_right_index_by_left_index();
            }
        }
    }

    fn delete_suf_char(&mut self) {
        let is_not_cursor_rightmost = self.input_buffer_index != self.input_buffer.chars().count();
        if is_not_cursor_rightmost {
            let current_index = self.input_buffer_index;
            let from_left_to_current_index = current_index + 1;
            let before_char_to_delete = self.input_buffer.chars().take(current_index);
            let after_char_to_delete = self.input_buffer.chars().skip(from_left_to_current_index);
            self.input_buffer = before_char_to_delete.chain(after_char_to_delete).collect();
        }
    }

    fn set_width_height(&mut self, width: usize, _height: usize) {
        self.width = width;
        // 调整指针位置
        if !self.align_right {
            self.end_of_cursor();
            self.align_right = true;
        }
    }

    fn get_content(&self) -> String {
        self.input_buffer.clone()
    }

    fn clear(&mut self) {
        self.input_buffer.clear();
        self.input_buffer_index = 0;
        self.left_index = 0;
        self.right_index = 0;
        self.cursor_position_x = 0;
        self.home_of_cursor();
    }
}

impl TextField {
    // 初始化
    pub fn new(input_buffer: String) -> Self {
        TextField {
            input_buffer,
            ..Default::default()
        }
    }

    /// 截取 input_buffer 字符串以供 UI 展示
    fn sub_input_buffer(&self) -> String {
        let start = self.left_index;
        let end = self.right_index;
        let mut result = String::new();

        for (c, index) in self.input_buffer.chars().zip(0..self.input_buffer.len()) {
            // 当我们达到起始字符索引时开始截取
            if index >= start && index < end {
                result.push(c);
            }
            if index == end {
                break;
            }
        }
        result
    }

    /// 限制光标位置，将光标位置限制在0到字符总长度之间
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input_buffer.chars().count())
    }

    /// 根据右指针位置调整左指针位置
    fn compact_left_index_by_right_index(&mut self) {
        let right = self.right_index;
        if right == 0 {
            return;
        }
        let input = self.input_buffer.clone();
        self.left_index = right;
        let mut width = 0;
        for index in (0..right).rev() {
            if let Some(c) = input.chars().nth(index) {
                width += c_len(c);
                self.left_index -= 1;
                if width > self.width {
                    self.left_index += 1;
                    break;
                }
                if width == self.width || self.left_index == 0 {
                    break;
                }
            }
        }
    }

    /// 根据左指针位置调整右指针位置
    fn compact_right_index_by_left_index(&mut self) {
        // 拿到当前的左指针坐标
        let left = self.left_index;
        if self.input_buffer.chars().count() == 0 {
            return;
        }
        let input = self.input_buffer.clone();
        self.right_index = left;
        let mut width = 0;
        for index in (left..self.input_buffer.chars().count() - 1).rev() {
            if let Some(c) = input.chars().nth(index) {
                width += c_len(c);
                self.right_index += 1;
                if width == self.width {
                    break;
                }
                if width > self.width {
                    self.left_index -= 1;
                    break;
                }
            }
        }
    }
}
