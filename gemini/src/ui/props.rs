/// 输入区域相关属性
#[derive(Default)]
pub struct InputFieldProps {
    /// 当前指针位置，光标指向输入字符串中第几位
    pub input_buffer_index: usize,
    /// 光标坐标 x，每一个 ASCII 字符占1位，非 ASCII 字符占2位
    /// 如果输入的文本为纯 ASCII 字符，则于 input_buffer_index 相等，如果包含非 ASCII 字符，则会比 input_buffer_index
    /// 大
    pub cursor_position_x: usize,
    /// 光标坐标 y
    pub cursor_position_y: usize,
    /// 输入框内容
    pub input_buffer: String,
    /// 输入框宽度
    pub width: usize,
    /// 输入框高度
    pub height: usize,
}

/// 输入框输入相关 Trait
pub trait InputFieldCursorNeed {
    /// 应该显示在输入框中的文本
    fn should_show_text(&self) -> String;
    /// 处理回车按键事件
    fn handle_enter_key(&mut self);
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
    fn length(&self, str: String) -> usize;
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

impl InputFieldCursorNeed for InputFieldProps {
    fn end_of_cursor(&mut self) {
        self.input_buffer_index = self.input_buffer.chars().count();
        self.cursor_position_y = self.input_buffer.lines().count() - 1;
        self.cursor_position_x = self.length(self.input_buffer.lines().last().unwrap_or("").into());
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
        // 光标有变化
        if origin_cursor_index != self.input_buffer_index {
            self.cursor_position_x = if c.is_ascii() {
                self.cursor_position_x.saturating_sub(1)
            } else {
                self.cursor_position_x.saturating_sub(2)
            }
        }
    }

    fn move_cursor_right(&mut self, c: char) {
        let origin_cursor_index = self.input_buffer_index;
        let cursor_moved_right = self.input_buffer_index.saturating_add(1);
        self.input_buffer_index = self.clamp_cursor(cursor_moved_right);
        // 光标有变化
        if origin_cursor_index != self.input_buffer_index {
            self.cursor_position_x = if c.is_ascii() {
                self.cursor_position_x.saturating_add(1)
            } else {
                self.cursor_position_x.saturating_add(2)
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
            .nth(self.input_buffer_index)
            .unwrap_or(self.input_buffer.len())
    }

    fn length(&self, str: String) -> usize {
        str.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
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

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        // 限制光标位置，将光标位置限制在0到字符总长度之间
        new_cursor_pos.clamp(0, self.input_buffer.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.input_buffer_index = 0;
        self.cursor_position_x = 0;
    }

    fn sub_input_buffer(&self, start: usize, count: usize) -> String {
        let mut result = String::new();
        let mut char_count = 0;

        for (i, c) in self.input_buffer.char_indices() {
            // 当我们达到起始字符索引时开始截取
            if i >= start && char_count < count {
                result.push(c);
                char_count += if c.is_ascii() { 1 } else { 2 };
            }
            // 当我们截取了足够的字符后停止
            if char_count == count {
                break;
            }
            if char_count > count {
                result.pop();
                break;
            }
        }
        result
    }

    fn should_show_text(&self) -> String {
        match self.height.cmp(&1) {
            // 高度为 0
            std::cmp::Ordering::Less => "".into(),
            // 单行输入框
            std::cmp::Ordering::Equal => {
                if self.length(self.input_buffer.clone()) > self.width && self.cursor_position_x > self.width {
                    self.sub_input_buffer(self.cursor_position_x - self.width, self.cursor_position_x)
                } else {
                    self.input_buffer.clone()
                }
            }
            // 多行输入框
            std::cmp::Ordering::Greater => self.input_buffer.clone(),
        }
    }

    fn handle_enter_key(&mut self) {
        // 如果为不是单行输入框，则可以对字符串进行换行操作
        if self.height > 1 {
            // 插入换行符
            let index = self.byte_index();
            self.input_buffer.insert(index, '\n');
            // 指向右移
            self.move_cursor_right('\n');
            // TODO: 不应该在此处修改光标坐标
            // self.cursor_position_y += 1;
            // self.cursor_position_x = 0;
        }
    }
}
