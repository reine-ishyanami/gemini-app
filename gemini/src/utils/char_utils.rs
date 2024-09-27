#![allow(unused)]

/// 计算字符宽度
///
/// # Examples
/// ```
/// assert_eq!(c_len('a'), 1);
/// assert_eq!(c_len('中'), 2);
/// assert_eq!(c_len('\n'), 0);
/// assert_eq!(c_len('\0'), 0);
/// ```
pub(crate) fn c_len(c: char) -> usize {
    let width_0 = ['\n', '\0'];
    if width_0.contains(&c) {
        0
    } else if c.is_ascii() {
        1
    } else {
        2
    }
}

/// 判断是否为中文字符
fn is_chinese_char(c: char) -> bool {
    // 基本汉字
    if (c as u32 >= 0x4E00) && (c as u32 <= 0x9FFF) {
        return true;
    }
    // 扩展区A
    if (c as u32 >= 0x3400) && (c as u32 <= 0x4DBF) {
        return true;
    }
    // 扩展区B
    if (c as u32 >= 0x20000) && (c as u32 <= 0x2A6DF) {
        return true;
    }
    // 扩展区C
    if (c as u32 >= 0x2A700) && (c as u32 <= 0x2B73F) {
        return true;
    }
    // 扩展区D
    if (c as u32 >= 0x2B740) && (c as u32 <= 0x2B81F) {
        return true;
    }
    // 扩展区E
    if (c as u32 >= 0x2B820) && (c as u32 <= 0x2CEAF) {
        return true;
    }
    // 兼容汉字
    if (c as u32 >= 0xF900) && (c as u32 <= 0xFAFF) {
        return true;
    }
    // 兼容扩展
    if (c as u32 >= 0x2F800) && (c as u32 <= 0x2FA1F) {
        return true;
    }
    false
}

/// 判断是否为中文标点
fn is_chinese_punctuation(c: char) -> bool {
    // CJK 符号和标点
    if (c as u32 >= 0x3000) && (c as u32 <= 0x303F) {
        return true;
    }
    // 全角ASCII、全角标点
    if (c as u32 >= 0xFF00) && (c as u32 <= 0xFFEF) {
        return true;
    }
    false
}

/// 获取输入框字符长度
///
/// # Examples
/// ```
/// let s = "你好，世界！";
/// assert_eq!(s_length(s), 12);
/// let s = "Hello, World!"
/// assert_eq!(s_length(s), 13);
/// ```
pub(crate) fn s_length(str: String) -> usize {
    str.chars().map(c_len).sum()
}
