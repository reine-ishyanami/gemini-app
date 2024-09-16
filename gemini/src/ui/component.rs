#![allow(unused)]

/// 输入框输入相关 Trait
pub trait InputTextComponent {
    /// 应该显示在输入框中的文本
    fn should_show_text(&self) -> String;
    /// 处理回车按键事件
    fn handle_enter_key(&mut self) {}
    /// 获取鼠标指针位置
    fn get_cursor_position(&self) -> (usize, usize);
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
    /// 向上移动光标
    fn move_cursor_up(&mut self) {}
    /// 向下移动光标
    fn move_cursor_down(&mut self) {}
    /// 输入字符
    fn enter_char(&mut self, new_char: char);
    /// 获取当前光标位置的字节索引,
    /// 如 input_buffer 为 "hello", input_buffer_index 为 1，则返回 1
    /// 如 input_buffer 为 "你好", input_buffer_index 为 2，则返回 3
    fn byte_index(&self) -> usize;
    /// 删除当前光标指向字符
    fn delete_pre_char(&mut self);
    /// 删除当前光标位置的后一个字符
    fn delete_suf_char(&mut self);
    /// 限制光标位置
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize;
    /// 重置光标位置
    fn reset_cursor(&mut self);
    /// 设置宽高
    fn set_width_height(&mut self, width: usize, height: usize);
    /// 获取输入框内容
    fn get_content(&self) -> String;
}

/// 通用方法

/// 计算字符宽度
fn c_len(c: char) -> usize {
    if c.is_ascii() {
        1
    } else {
        2
    }
}

/// 获取输入框字符长度
fn length(str: String) -> usize {
    str.chars().map(c_len).sum()
}

/// 单行输入框相关属性
#[derive(Default)]
pub struct TextField {
    /// 当前指针位置，光标指向输入字符串中第几位
    pub input_buffer_index: usize,
    /// 光标坐标 x，每一个 ASCII 字符占1位，非 ASCII 字符占2位
    /// 如果输入的文本为纯 ASCII 字符，则于 input_buffer_index 相等，
    /// 如果包含非 ASCII 字符，则会比 input_buffer_index 大
    pub cursor_position_x: usize,
    /// 输入框内容
    pub input_buffer: String,
    /// 输入框宽度
    pub width: usize,
    /// 是否已经初始化过指针位置
    pub align_right: bool,
}

impl InputTextComponent for TextField {
    fn should_show_text(&self) -> String {
        // 根据指针位置截取内容展示，输入框内容长度大于组件宽度，并且指针位置截取的宽度大于组件宽度
        if length(self.input_buffer.clone()) > self.width && self.cursor_position_x > self.width {
            self.sub_input_buffer(self.cursor_position_x - self.width, self.cursor_position_x)
        } else {
            self.input_buffer.clone()
        }
    }

    fn get_cursor_position(&self) -> (usize, usize) {
        (
            if self.cursor_position_x > self.width {
                self.width
            } else {
                self.cursor_position_x
            },
            0,
        )
    }

    fn end_of_cursor(&mut self) {
        self.input_buffer_index = self.input_buffer.chars().count();
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
            self.cursor_position_x = self.cursor_position_x.saturating_sub(c_len(c))
        }
    }

    fn move_cursor_right(&mut self, c: char) {
        let origin_cursor_index = self.input_buffer_index;
        // 指针位置指向下一位
        let cursor_moved_right = self.input_buffer_index.saturating_add(1);
        self.input_buffer_index = self.clamp_cursor(cursor_moved_right);
        // 光标有变化
        if origin_cursor_index != self.input_buffer_index {
            self.cursor_position_x = self.cursor_position_x.saturating_add(c_len(c))
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

    // 限制光标位置，将光标位置限制在0到字符总长度之间
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input_buffer.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.input_buffer_index = 0;
        self.cursor_position_x = 0;
    }

    fn set_width_height(&mut self, width: usize, _height: usize) {
        self.width = width;
        // self.height = height;
        // 调整指针位置
        if !self.align_right {
            self.end_of_cursor();
            // 指针 x 坐标
            self.cursor_position_x = length(self.input_buffer.lines().last().unwrap_or("").into());
            // 指针 y 坐标
            self.align_right = true;
        }
    }

    fn get_content(&self) -> String {
        self.input_buffer.clone()
    }
}

impl TextField {
    /// 截取 input_buffer 字符串以供 UI 展示
    fn sub_input_buffer(&self, start: usize, count: usize) -> String {
        let mut result = String::new();
        let mut char_count = 0;

        // 记录当前遍历过的字符总宽度
        let mut j = 0;
        for (_, c) in self.input_buffer.char_indices() {
            // 当我们达到起始字符索引时开始截取
            if j >= start && char_count < count {
                result.push(c);
                char_count += c_len(c);
            }
            // 每遍历完递增宽度
            j += c_len(c);
            // 当我们截取了足够的字符后停止
            if char_count == count {
                break;
            }
            // 当我们超过了字符总长度时停止
            if char_count > count {
                result.push(' ');
                break;
            }
        }
        result
    }
}

/// 多行输入框相关属性
#[derive(Default)]
pub struct TextArea {
    /// 当前指针位置，光标指向输入字符串中第几位
    pub input_buffer_index: usize,
    /// 光标坐标 x，每一个 ASCII 字符占1位，非 ASCII 字符占2位
    /// 如果输入的文本为纯 ASCII 字符，则于 input_buffer_index 相等，
    /// 如果包含非 ASCII 字符，则会比 input_buffer_index 大
    pub cursor_position_x: usize,
    /// 光标坐标 y
    pub cursor_position_y: usize,
    /// 输入框内容
    pub input_buffer: String,
    /// 输入框宽度
    pub width: usize,
    /// 输入框高度
    pub height: usize,
    /// 每行最大宽度
    pub each_line_max_width: Vec<usize>,
    /// 是否已经初始化过指针位置
    pub align_right: bool,
}

impl InputTextComponent for TextArea {
    fn should_show_text(&self) -> String {
        self.split_overflow_line_to_a_new_line()
    }

    fn handle_enter_key(&mut self) {
        // 插入换行符
        let index = self.byte_index();
        self.input_buffer.insert(index, '\n');
        // 指向右移
        self.move_cursor_right('\n');
    }

    fn get_cursor_position(&self) -> (usize, usize) {
        (
            if self.cursor_position_x > self.width {
                self.width
            } else {
                self.cursor_position_x
            },
            if self.cursor_position_y > self.height {
                self.height
            } else {
                self.cursor_position_y
            },
        )
    }

    fn end_of_cursor(&mut self) {
        self.input_buffer_index = self.input_buffer.chars().count();
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
        todo!()
    }

    fn move_cursor_right(&mut self, c: char) {
        todo!()
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
        todo!()
    }

    fn delete_suf_char(&mut self) {
        todo!()
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        todo!()
    }

    fn reset_cursor(&mut self) {
        todo!()
    }

    fn set_width_height(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        let new_str = self.split_overflow_line_to_a_new_line();
        self.each_line_max_width = new_str.lines().map(|line| length(line.into())).collect();
        // 调整指针位置
        if !self.align_right {
            self.end_of_cursor();
            // 指针 x 坐标
            self.cursor_position_x = *self.each_line_max_width.last().unwrap_or(&0);
            // 指针 y 坐标
            self.cursor_position_y = new_str.lines().count().saturating_sub(1);
            self.align_right = true;
        }
    }

    fn get_content(&self) -> String {
        self.input_buffer.clone()
    }
}

impl TextArea {
    /// 对单行长文本进行手动换行操作
    fn split_overflow_line_to_a_new_line(&self) -> String {
        let mut message = String::new();
        // 对长文本进行插入换行符号
        let mut line_width = 0;
        let width = if self.width % 2 == 0 {
            self.width
        } else {
            self.width - 1
        };
        for (_, c) in self.input_buffer.clone().char_indices() {
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
            message.push(c);
            line_width += c_len(c);
            if c == '\n' {
                line_width = 0;
            }
        }
        message
    }
}
