//! 显示宽度计算测试（宽字符支持）

use aivim_core::buffer::{Buffer, BufferId};

#[test]
fn test_ascii_display_width() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello World");
    
    // ASCII 字符显示宽度等于字符数
    assert_eq!(buffer.line_display_width(0), 11);
    assert_eq!(buffer.line_display_width_to_column(0, 5), 5);
}

#[test]
fn test_chinese_display_width() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "你好世界");
    
    // 中文字符显示宽度为 2
    assert_eq!(buffer.line_display_width(0), 8); // 4 个中文字符 × 2
    
    // 到第 2 个字符（"你好"）的显示宽度
    assert_eq!(buffer.line_display_width_to_column(0, 2), 4);
}

#[test]
fn test_mixed_display_width() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello你好");
    
    // "Hello" (5) + "你好" (4) = 9
    assert_eq!(buffer.line_display_width(0), 9);
    
    // 到 "Hello" 后的显示宽度
    assert_eq!(buffer.line_display_width_to_column(0, 5), 5);
    
    // 到 "Hello你" 后的显示宽度
    assert_eq!(buffer.line_display_width_to_column(0, 6), 7);
}

#[test]
fn test_display_width_with_punctuation() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "你好，世界！");
    
    // 中文标点符号也是宽字符（宽度为 2）
    // "你好" (4) + "，" (2) + "世界" (4) + "！" (2) = 12
    assert_eq!(buffer.line_display_width(0), 12);
}

#[test]
fn test_empty_line_display_width() {
    let buffer = Buffer::new(BufferId::new(0));
    
    assert_eq!(buffer.line_display_width(0), 0);
    assert_eq!(buffer.line_display_width_to_column(0, 0), 0);
}

#[test]
fn test_display_width_beyond_line_length() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello");
    
    // 超出行长度的列应该返回整行宽度
    assert_eq!(buffer.line_display_width_to_column(0, 10), 5);
}
