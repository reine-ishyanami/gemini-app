/// 输入框输入相关 Trait
pub(crate) trait InputTextComponent {
    /// 应该显示在输入框中的文本
    fn should_show_text(&self) -> String;
    /// 处理回车按键事件
    fn handle_enter_key(&mut self) {}
    /// 获取鼠标指针位置
    fn get_cursor_position(&self) -> (usize, usize);
    /// 定位到字符串末尾
    fn end_of_cursor(&mut self);
    /// 将光标多行文本的最末端
    fn end_of_multiline(&mut self) {
        self.end_of_cursor();
    }
    /// 定位到字符串开头
    fn home_of_cursor(&mut self);
    /// 将光标多行文本的最前端
    fn home_of_multiline(&mut self) {
        self.home_of_cursor();
    }
    /// 获取当前光标指向的字符
    fn get_current_char(&self) -> char;
    /// 获取当前光标的下一个字符
    fn get_next_char(&self) -> char;
    /// 向左移动光标
    /// 返回是否光标位置有变化
    fn move_cursor_left(&mut self, c: char);
    /// 向右移动光标
    /// 返回是否光标位置有变化
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
    /// 设置宽高
    fn set_width_height(&mut self, width: usize, height: usize);
    /// 获取输入框内容
    fn get_content(&self) -> String;
    /// 清空文本
    fn clear(&mut self);
}
