use crate::utils::char_utils::{c_len, s_length};

use super::input_trait::InputTextComponent;

/// 单行输入框相关属性
#[derive(Default)]
pub struct TextField {
    /// 当前指针位置，光标指向输入字符串中第几位
    input_buffer_index: usize,
    /// 最左侧光标索引
    left_index: usize,
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
    /// 是否为插入新字符
    insert_char: bool,
}

impl InputTextComponent for TextField {
    fn should_show_text(&self) -> String {
        let start = self.left_index;
        let mut result = String::new();
        let input = self.input_buffer.clone();
        let mut width = 0;

        for index in start..input.chars().count() {
            if let Some(c) = input.chars().nth(index) {
                result.push(c);
                width += c_len(c);
                if width > self.width {
                    break;
                }
            }
        }
        result
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
        let x = self.cursor_position_x.saturating_sub(width);
        (x.clamp(0, self.width), 0)
    }

    fn end_of_cursor(&mut self) {
        let input = self.input_buffer.clone();
        self.input_buffer_index = input.chars().count();
        self.cursor_position_x = s_length(input);
        let mut width = 0;
        self.left_index = self.input_buffer_index;
        for index in (0..self.input_buffer_index).rev() {
            if let Some(c) = self.input_buffer.chars().nth(index) {
                // 计算当前宽度
                width += c_len(c);
                if width <= self.width {
                    self.left_index -= 1;
                } else {
                    break;
                }
            }
        }
    }

    fn home_of_cursor(&mut self) {
        self.input_buffer_index = 0;
        self.cursor_position_x = 0;
        self.left_index = 0;
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
            self.left_index -= 1;
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
        // 如果当前字符不在左右范围内，则左指针右移
        if self.is_overflow_width() {
            self.left_index += 1;
        }
        // 光标有变化
        if origin_cursor_index != self.input_buffer_index {
            self.cursor_position_x = self.cursor_position_x.saturating_add(c_len(c));
        }
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input_buffer.insert(index, new_char);
        self.insert_char = true;
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
        self.cursor_position_x = 0;
    }
}

impl TextField {
    /// 初始化
    pub fn new(input_buffer: String) -> Self {
        Self {
            input_buffer,
            ..Default::default()
        }
    }

    /// 限制光标位置，将光标位置限制在0到字符总长度之间
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input_buffer.chars().count())
    }

    /// 判断从左到右的字符宽度是否大于输入框宽度
    fn is_overflow_width(&self) -> bool {
        let left = self.left_index;
        let current_index = self.input_buffer_index;
        let input = self.input_buffer.clone();
        let mut width = 0;
        for i in left..current_index {
            if let Some(c) = input.chars().nth(i) {
                width += c_len(c);
            }
        }
        width > self.width
    }
}
