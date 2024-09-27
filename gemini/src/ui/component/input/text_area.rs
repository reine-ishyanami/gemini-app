use crate::utils::char_utils::{c_len, s_length};

use super::input_trait::InputTextComponent;

/// 多行输入框相关属性
#[derive(Default)]
pub(crate) struct TextArea {
    /// 当前指针位置，光标指向输入字符串中第几位
    input_buffer_index: usize,
    /// y 指向文本行的坐标，每一个 ASCII 字符占1位，非 ASCII 字符占2位
    /// 如果输入的文本为纯 ASCII 字符，则于 input_buffer_index 相等，
    /// 如果包含非 ASCII 字符，则会比 input_buffer_index 大
    cursor_position_x: usize,
    /// 第 y 行文本
    cursor_position_y: usize,
    /// 输入框内容
    input_buffer: String,
    /// 输入框宽度
    width: usize,
    /// 输入框高度
    height: usize,
    /// 每行的字符串
    each_line_string: Vec<String>,
    /// 是否已经初始化过指针位置
    align_right: bool,
    /// 指针总偏移量
    offset: isize,
    /// 是否输入字符
    is_input: bool,
}

impl InputTextComponent for TextArea {
    fn should_show_text(&self) -> String {
        let text = self.split_overflow_line_to_a_new_line();
        text.replace("\n\n", "\n")
    }

    fn handle_enter_key(&mut self) {
        if self.get_current_char() != '\n' {
            self.enter_char('\n');
        }
    }

    fn get_cursor_position(&self) -> (usize, usize) {
        let line = self.cursor_position_y;
        let mut y = 0;
        for index in 0..line {
            let size = self.get_len_of_line(index);
            // 如果本行不为 0 , 才记为有效行
            if size != 0 {
                y += 1;
            }
        }
        (self.cursor_position_x.clamp(0, self.width), y.clamp(0, self.height))
    }

    fn end_of_cursor(&mut self) {
        let y = self.cursor_position_y;
        // 需要添加的行
        let mut add_line = -1;
        for index in y..self.each_line_string.len() {
            if self.get_len_of_line(index) == 0 {
                if add_line == -1 {
                    self.cursor_position_y = y;
                    self.cursor_position_x = self.get_len_of_line(y);
                } else {
                    self.cursor_position_y += add_line as usize;
                }
                break;
            }
            add_line += 1;
            self.cursor_position_x = self.get_len_of_line(index);
        }
        self.update_offset(0);
        self.update_input_buffer_index();
    }

    fn end_of_multiline(&mut self) {
        if !self.each_line_string.is_empty() {
            // 指针 x 坐标
            self.cursor_position_x = self.get_len_of_line(self.each_line_string.len() - 1);
            // 指针 y 坐标，跳过空行
            self.cursor_position_y = self.each_line_string.len() - 1;
        } else {
            self.cursor_position_x = 0;
            self.cursor_position_y = 0;
        }
        self.update_offset(0);
        self.update_input_buffer_index();
    }

    fn home_of_cursor(&mut self) {
        let y = self.cursor_position_y;
        // 需要减少的行
        let mut sub_line = -1;
        for index in (0..=y).rev() {
            if self.get_len_of_line(index) == 0 {
                if sub_line == -1 {
                    self.cursor_position_y = y;
                    self.cursor_position_x = 0;
                } else {
                    self.cursor_position_y -= sub_line as usize;
                }
                break;
            }
            sub_line += 1;
            self.cursor_position_x = 0;
        }
        self.update_offset(0);
        self.update_input_buffer_index();
    }

    fn home_of_multiline(&mut self) {
        // self.input_buffer_index = 0;
        self.cursor_position_y = 0;
        self.cursor_position_x = 0;
        self.update_offset(0);
        self.update_input_buffer_index();
    }

    fn get_current_char(&self) -> char {
        if self.input_buffer_index == 0 {
            '\0'
        } else {
            self.input_buffer
                .chars()
                .nth(self.input_buffer_index - 1)
                .unwrap_or('\0')
        }
    }

    fn get_next_char(&self) -> char {
        self.input_buffer.chars().nth(self.input_buffer_index).unwrap_or('\0')
    }

    fn move_cursor_left(&mut self, c: char) {
        if self.cursor_position_y != 0 || self.cursor_position_x != 0 {
            self.input_buffer_index = self.input_buffer_index.saturating_sub(1);
            // 减去字符宽度
            self.update_offset(0 - c_len(c) as isize);
        }
    }

    fn move_cursor_right(&mut self, c: char) {
        // 如果为空，则直接移动
        if self.each_line_string.is_empty() {
            self.input_buffer_index = self.input_buffer_index.saturating_add(1);
            self.cursor_position_x = c_len(c);
            // 加上字符宽度
            self.update_offset(c_len(c) as isize);
            self.end_of_cursor();
            return;
        }
        // 不是最后一行允许右移，输入允许右移
        if (self.cursor_position_y != self.each_line_string.len() - 1
            || self.cursor_position_x != self.get_len_of_line(self.each_line_string.len() - 1))
            || self.is_input
        {
            self.input_buffer_index = self.input_buffer_index.saturating_add(1);
            // 加上字符宽度
            self.update_offset(c_len(c) as isize);
            self.is_input = false;
        }
    }

    fn enter_char(&mut self, new_char: char) {
        self.is_input = true;
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
            if delete_char != '\n' {
                self.move_cursor_left(delete_char);
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

    fn set_width_height(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        // 将长文本分行
        let new_str = self.split_overflow_line_to_a_new_line();
        // 计算每行
        self.each_line_string = new_str.lines().map(Into::into).collect();
        // 调整指针位置，如果是第一次进入菜单绘制组件，则重新调整指针位置
        if !self.align_right {
            self.end_of_multiline();
            self.update_offset(0);
            self.align_right = true;
            return;
        }
        // 拿到当前坐标总位置
        let cursor_pos = self.offset as usize;
        self.adjust_x_y(cursor_pos);
    }

    fn get_content(&self) -> String {
        self.input_buffer.clone()
    }

    fn clear(&mut self) {
        self.input_buffer.clear();
        self.input_buffer_index = 0;
        self.cursor_position_x = 0;
        self.cursor_position_y = 0;
        self.each_line_string.clear();
        self.offset = 0;
        self.align_right = true;
    }
}

impl TextArea {
    // 初始化
    pub fn new(input_buffer: String) -> Self {
        TextArea {
            input_buffer,
            ..Default::default()
        }
    }
    /// 对单行长文本进行手动换行操作
    fn split_overflow_line_to_a_new_line(&self) -> String {
        let mut message = String::new();
        // 对长文本进行插入换行符号
        let mut line_width = 0;
        let width = self.width;
        // 如果以换行符结尾，则在最后一个换行符之前插入换行符，使行数正确
        let mut input_buffer = self.input_buffer.clone();
        if input_buffer.ends_with('\n') {
            input_buffer.push('\n');
        }
        // 每一行最后的字符是换行符
        for (_, c) in input_buffer.char_indices() {
            if c == '\n' {
                message.push('\n');
                line_width = 0;
            }
            message.push(c);
            line_width += c_len(c);
            // 如果当前行宽度正好为组件宽度，则插入换行符
            if line_width == width {
                message.push('\n');
                line_width = 0;
            }
            // 如果当前字符宽度大于组件宽度，则在最后一个字符之前插入换行符插入换行符
            if line_width > width {
                let c = message.pop().unwrap();
                message.push('\n');
                message.push(c);
                line_width = c_len(c);
            }
        }
        message
    }

    /// 指针总指向的长度
    fn update_offset(&mut self, offset: isize) {
        self.offset = 0;
        for (i, line) in self.each_line_string.clone().iter().enumerate() {
            let ele = s_length(line.clone());
            if i == self.cursor_position_y {
                break;
            }
            self.offset += ele as isize;
        }
        self.offset += self.cursor_position_x as isize;
        self.offset += offset;
    }

    /// 拿到指定行的长度
    fn get_len_of_line(&self, index: usize) -> usize {
        let str = self.each_line_string.get(index);
        if let Some(str) = str {
            s_length(str.into())
        } else {
            0
        }
    }

    /// 根据 x y 坐标更新 input_buffer_index
    fn update_input_buffer_index(&mut self) {
        // 先拿到 y 坐标
        let y = self.cursor_position_y; // 0
        if self.each_line_string.is_empty() {
            return;
        }
        self.input_buffer_index = 0;
        for (index, line) in self.each_line_string.clone().iter().enumerate() {
            // 如果已经到当前行
            if index == y {
                let mut x = self.cursor_position_x;
                // 遍历本行所有字符并计算其宽度
                for (_, len) in line.chars().map(|c| (c, c_len(c))) {
                    if x == 0 {
                        break;
                    }
                    self.input_buffer_index += 1;
                    x -= len;
                }
                break;
            }
            // 获得每一行的的字符数
            let chars_count = line.chars().count();
            // 空行表示一个换行符， +1
            if chars_count == 0 {
                self.input_buffer_index += 1;
            } else {
                // 非空行，则 + 字符数
                self.input_buffer_index += chars_count;
            }
        }
    }

    /// 调整 x y 位置
    fn adjust_x_y(&mut self, cursor_pos: usize) {
        let mut cursor_pos = cursor_pos;
        // 调整指针位置
        self.cursor_position_x = 0;
        self.cursor_position_y = 0;
        let mut is_end_of_text = false;
        // 如果当前指向总坐标为 0 则不用调整
        if cursor_pos == 0 {
            return;
        }
        // 遍历每一行
        for line in self.each_line_string.clone() {
            let len = s_length(line);
            // // 如果本行宽度为 0，则表示换行，如果遍历完文本后还有换行，则继续换行
            if len == 0 && self.get_current_char() == '\n' {
                self.cursor_position_y += 1;
                self.cursor_position_x = 0;
                continue;
            }
            // 是否已经遍历完文本
            if is_end_of_text {
                break;
            }
            // 如果坐标还有余
            if cursor_pos > len {
                // 减去本行宽度
                cursor_pos -= len;
                // 加 1 行
                self.cursor_position_y += 1;
                self.cursor_position_x = 0;
                continue;
            }
            // 偏移量不足以减去一行宽度时，则将该偏移量赋予 x，退出循环
            if cursor_pos <= len {
                self.cursor_position_x = cursor_pos;
                is_end_of_text = true;
            }
        }
    }
}
