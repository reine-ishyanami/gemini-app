/// 输入区域相关属性
#[derive(Default)]
pub struct InputFieldProps {
    /// 指针位置，光标指向输入字符串中第几位
    pub cursor_index: usize,
    /// 字符位置，光标当前坐标，每一个 ASCII 字符占1位，非 ASCII 字符占2位
    /// 如果输入的文本为纯 ASCII 字符，则于 cursor_index 相等，如果包含非 ASCII 字符，则会比 cursor_index 大
    pub charactor_index: usize,
    /// 输入框内容
    pub input_buffer: String,
}

/// 输入框输入相关 Trait
pub trait InputFieldCursorNeed {
    /// 定位到字符串末尾
    fn end_of_cursor(&mut self);
    /// 获取当前光标指向的字符
    fn get_current_char(&self) -> char;
    /// 获取当前光标的下一个字符
    fn get_next_char(&self) -> char;
    /// 向左移动光标
    fn move_cursor_left(&mut self, c: char);
    /// 向右移动光标
    fn move_cursor_right(&mut self, c: char);
    /// 输入字符
    fn enter_char(&mut self, new_char: char);
    /// 获取当前光标位置的字节索引
    fn byte_index(&self) -> usize;
    /// 获取输入框字符长度
    fn input_length(&self) -> usize;
    /// 删除当前光标指向字符
    fn delete_pre_char(&mut self);
    /// 删除当前光标位置的后一个字符
    fn delete_suf_char(&mut self);
    /// 限制光标位置
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize;
    /// 重置光标位置
    fn reset_cursor(&mut self);
    /// 截取 input_buffer 字符串以供UI展示
    fn sub_input_buffer(&self, start: usize, count: usize) -> String;
}

/// 输入区域输入相关 Trait
#[allow(unused)]
pub trait InputAreaCursorNeed: InputFieldCursorNeed {
    /// 换行
    fn break_line(&mut self);
}

impl InputFieldCursorNeed for InputFieldProps {
    fn end_of_cursor(&mut self) {
        self.cursor_index = self.input_buffer.chars().count();
        self.charactor_index = self.input_length();
    }

    fn get_current_char(&self) -> char {
        if self.cursor_index == 0 {
            '\0'
        } else {
            self.input_buffer.chars().nth(self.cursor_index - 1).unwrap()
        }
    }

    fn get_next_char(&self) -> char {
        self.input_buffer.chars().nth(self.cursor_index).unwrap_or('\0')
    }

    fn move_cursor_left(&mut self, c: char) {
        let origin_cursor_index = self.cursor_index;
        let cursor_moved_left = self.cursor_index.saturating_sub(1);
        self.cursor_index = self.clamp_cursor(cursor_moved_left);
        // 光标有变化
        if origin_cursor_index != self.cursor_index {
            self.charactor_index = if c.is_ascii() {
                self.charactor_index.saturating_sub(1)
            } else {
                self.charactor_index.saturating_sub(2)
            }
        }
    }

    fn move_cursor_right(&mut self, c: char) {
        let origin_cursor_index = self.cursor_index;
        let cursor_moved_right = self.cursor_index.saturating_add(1);
        self.cursor_index = self.clamp_cursor(cursor_moved_right);
        // 光标有变化
        if origin_cursor_index != self.cursor_index {
            self.charactor_index = if c.is_ascii() {
                self.charactor_index.saturating_add(1)
            } else {
                self.charactor_index.saturating_add(2)
            }
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
            .nth(self.cursor_index)
            .unwrap_or(self.input_buffer.len())
    }

    fn input_length(&self) -> usize {
        self.input_buffer
            .chars()
            .map(|c| if c.is_ascii() { 1 } else { 2 })
            .sum()
    }

    fn delete_pre_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_index != 0;
        if is_not_cursor_leftmost {
            let delete_char = self.get_current_char();
            let current_index = self.cursor_index;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input_buffer.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input_buffer.chars().skip(current_index);
            self.input_buffer = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left(delete_char);
        }
    }

    fn delete_suf_char(&mut self) {
        let is_not_cursor_rightmost = self.cursor_index != self.input_buffer.chars().count();
        if is_not_cursor_rightmost {
            let current_index = self.cursor_index;
            let from_left_to_current_index = current_index + 1;
            let before_char_to_delete = self.input_buffer.chars().take(current_index);
            let after_char_to_delete = self.input_buffer.chars().skip(from_left_to_current_index);
            self.input_buffer = before_char_to_delete.chain(after_char_to_delete).collect();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input_buffer.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.cursor_index = 0;
        self.charactor_index = 0;
    }

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
}
