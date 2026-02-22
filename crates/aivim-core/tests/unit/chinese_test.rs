//! 中文支持测试

use aivim_core::buffer::{Buffer, BufferId};
use aivim_core::cursor::Cursor;

#[test]
fn test_chinese_text_insert() {
    let mut buffer = Buffer::new(BufferId::new(0));
    
    // 插入中文文本
    let chinese_text = "你好世界";
    buffer.insert(0, chinese_text);
    
    // 验证内容
    let content = buffer.to_string();
    assert_eq!(content, chinese_text);
    
    // 验证字符数（不是字节数）
    assert_eq!(buffer.len_lines(), 1);
}

#[test]
fn test_chinese_cursor_movement() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "你好世界");
    
    let mut cursor = Cursor::at_origin();
    assert_eq!(cursor.column, 0);
    
    // 向右移动一个字符（应该移动一个中文字符）
    cursor.move_right(&buffer, 1);
    assert_eq!(cursor.column, 1);
    
    // 再移动
    cursor.move_right(&buffer, 1);
    assert_eq!(cursor.column, 2);
}

#[test]
fn test_chinese_line_length() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "你好，世界！");
    
    // 行长度应该是字符数，不是字节数
    let line_len = buffer.line_len(0);
    assert_eq!(line_len, 6); // "你好，世界！" = 6 个字符
}

#[test]
fn test_mixed_chinese_english() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello 你好 World 世界");
    
    let content = buffer.to_string();
    assert_eq!(content, "Hello 你好 World 世界");
    
    // 测试光标移动
    let mut cursor = Cursor::at_origin();
    cursor.move_right(&buffer, 6); // "Hello " = 6 个字符
    assert_eq!(cursor.column, 6);
}
