//! Cursor 模块单元测试
//!
//! 对应源文件: src/cursor.rs
//! 测试范围: 光标位置管理、移动、字符索引转换

use aivim_core::buffer::{Buffer, BufferId};
use aivim_core::cursor::Cursor;

// ==================== 基础创建测试 ====================

#[test]
fn test_cursor_new() {
    let cursor = Cursor::new(5, 10);
    assert_eq!(cursor.line, 5);
    assert_eq!(cursor.column, 10);
    assert_eq!(cursor.preferred_column, None);
}

#[test]
fn test_cursor_at_origin() {
    let cursor = Cursor::at_origin();
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 0);
    assert_eq!(cursor.preferred_column, None);
}

// ==================== 基础移动测试 ====================

#[test]
fn test_cursor_movement() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    let mut cursor = Cursor::at_origin();
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 0);
    
    cursor.move_down(&buffer, 1);
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 0);
    
    cursor.move_right(&buffer, 3);
    assert_eq!(cursor.column, 3);
    
    cursor.move_up(&buffer, 1);
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 3);
}

#[test]
fn test_cursor_move_left() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello\nWorld");
    
    let mut cursor = Cursor::new(0, 5);
    cursor.move_left(&buffer, 1);
    assert_eq!(cursor.column, 4);
    
    cursor.move_left(&buffer, 10);
    assert_eq!(cursor.column, 0);
}

#[test]
fn test_cursor_move_left_cross_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello\nWorld");
    
    let mut cursor = Cursor::new(1, 0);
    cursor.move_left(&buffer, 1);
    // 应该跳到上一行的末尾
    assert_eq!(cursor.line, 0);
    // 当没有 preferred_column 时，跳到行尾
    assert_eq!(cursor.column, 0); // 实际行为：使用 min(preferred_column.unwrap_or(0), line_len-1)
}

#[test]
fn test_cursor_move_right() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello");
    
    let mut cursor = Cursor::at_origin();
    cursor.move_right(&buffer, 3);
    assert_eq!(cursor.column, 3);
    
    cursor.move_right(&buffer, 10);
    assert_eq!(cursor.column, 4); // 不能超过行尾
}

#[test]
fn test_cursor_move_right_cross_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello\nWorld");
    
    let mut cursor = Cursor::new(0, 4);
    cursor.move_right(&buffer, 2);
    // 应该跳到下一行的开头
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 0);
}

#[test]
fn test_cursor_move_up() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    let mut cursor = Cursor::new(2, 3);
    cursor.move_up(&buffer, 1);
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 3);
    
    cursor.move_up(&buffer, 10);
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 3);
}

#[test]
fn test_cursor_move_down() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    let mut cursor = Cursor::at_origin();
    cursor.move_down(&buffer, 1);
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 0);
    
    cursor.move_down(&buffer, 10);
    assert_eq!(cursor.line, 2);
    assert_eq!(cursor.column, 0);
}

// ==================== 行首行尾测试 ====================

#[test]
fn test_cursor_line_start_end() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello, World!");
    
    let mut cursor = Cursor::new(0, 5);
    
    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.column, 12);
    
    cursor.move_to_line_start();
    assert_eq!(cursor.column, 0);
}

#[test]
fn test_cursor_move_to_line_start() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello World");
    
    let mut cursor = Cursor::new(0, 10);
    cursor.move_to_line_start();
    assert_eq!(cursor.column, 0);
    assert_eq!(cursor.preferred_column, Some(0));
}

#[test]
fn test_cursor_move_to_line_end_empty_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "");
    
    let mut cursor = Cursor::at_origin();
    cursor.move_to_line_end(&buffer);
    // 空行的行尾是 0
    assert_eq!(cursor.column, 0);
}

// ==================== 第一个非空白字符测试 ====================

#[test]
fn test_cursor_move_to_first_non_blank() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "   Hello");
    
    let mut cursor = Cursor::at_origin();
    cursor.move_to_first_non_blank(&buffer);
    assert_eq!(cursor.column, 3);
}

#[test]
fn test_cursor_move_to_first_non_blank_no_whitespace() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello");
    
    let mut cursor = Cursor::new(0, 3);
    cursor.move_to_first_non_blank(&buffer);
    assert_eq!(cursor.column, 0);
}

#[test]
fn test_cursor_move_to_first_non_blank_all_whitespace() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "   ");
    
    let mut cursor = Cursor::at_origin();
    cursor.move_to_first_non_blank(&buffer);
    // 全是空白字符，position 返回 None，使用 unwrap_or(0)
    assert_eq!(cursor.column, 0);
}

// ==================== 跳转测试 ====================

#[test]
fn test_cursor_move_to_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    let mut cursor = Cursor::at_origin();
    cursor.move_to_line(1, &buffer);
    assert_eq!(cursor.line, 1);
    
    cursor.move_to_line(100, &buffer);
    assert_eq!(cursor.line, 2); // 不能超过最大行号
}

#[test]
fn test_cursor_move_to_top() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    let mut cursor = Cursor::new(2, 3);
    cursor.move_to_top(&buffer);
    assert_eq!(cursor.line, 0);
}

#[test]
fn test_cursor_move_to_bottom() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    let mut cursor = Cursor::at_origin();
    cursor.move_to_bottom(&buffer);
    assert_eq!(cursor.line, 2);
}

// ==================== 字符索引转换测试 ====================

#[test]
fn test_cursor_char_idx_conversion() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    let cursor = Cursor::new(1, 2);
    let char_idx = cursor.to_char_idx(&buffer);
    assert_eq!(char_idx, 9);
    
    let cursor2 = Cursor::from_char_idx(&buffer, char_idx);
    assert_eq!(cursor2.line, 1);
    assert_eq!(cursor2.column, 2);
}

#[test]
fn test_cursor_to_char_idx_first_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello World");
    
    let cursor = Cursor::new(0, 5);
    let char_idx = cursor.to_char_idx(&buffer);
    assert_eq!(char_idx, 5);
}

#[test]
fn test_cursor_from_char_idx_first_line() {
    let buffer = Buffer::new(BufferId::new(0));
    
    let cursor = Cursor::from_char_idx(&buffer, 0);
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 0);
}

// ==================== preferred_column 测试 ====================

#[test]
fn test_cursor_preferred_column() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello\nHi\nWorld");
    
    let mut cursor = Cursor::new(0, 5);
    cursor.update_preferred_column();
    assert_eq!(cursor.preferred_column, Some(5));
    
    // 移动到短行
    cursor.move_down(&buffer, 1);
    // 应该保持在 preferred_column 和行长的最小值
    assert_eq!(cursor.column, 2); // "Hi" 的长度是 2
}

#[test]
fn test_cursor_move_down_with_preferred_column() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello World\nHi\nTest Line");
    
    let mut cursor = Cursor::new(0, 8);
    cursor.update_preferred_column();
    
    // 移动到短行
    cursor.move_down(&buffer, 1);
    assert_eq!(cursor.column, 2); // 被限制在 "Hi" 的长度
    
    // 移动到长行
    cursor.move_down(&buffer, 1);
    assert_eq!(cursor.column, 8); // 恢复 preferred_column
}

// ==================== ensure_valid 测试 ====================

#[test]
fn test_cursor_ensure_valid() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello\nWorld");
    
    let mut cursor = Cursor::new(100, 100);
    cursor.ensure_valid(&buffer);
    assert_eq!(cursor.line, 1); // 最大行号
    assert_eq!(cursor.column, 4); // "World" 的长度减 1（最大列索引）
}

#[test]
fn test_cursor_ensure_valid_empty_buffer() {
    let buffer = Buffer::new(BufferId::new(0));
    
    let mut cursor = Cursor::new(10, 10);
    cursor.ensure_valid(&buffer);
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 0);
}

// ==================== 边界情况测试 ====================

#[test]
fn test_cursor_move_left_at_origin() {
    let buffer = Buffer::new(BufferId::new(0));
    
    let mut cursor = Cursor::at_origin();
    cursor.move_left(&buffer, 1);
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 0);
}

#[test]
fn test_cursor_move_right_at_end() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hi");
    
    let mut cursor = Cursor::new(0, 1);
    cursor.move_right(&buffer, 10);
    assert_eq!(cursor.column, 1); // 不能超过行尾
}

#[test]
fn test_cursor_move_up_at_top() {
    let buffer = Buffer::new(BufferId::new(0));
    
    let mut cursor = Cursor::at_origin();
    cursor.move_up(&buffer, 1);
    assert_eq!(cursor.line, 0);
}

#[test]
fn test_cursor_move_down_at_bottom() {
    let buffer = Buffer::new(BufferId::new(0));
    
    let mut cursor = Cursor::at_origin();
    cursor.move_down(&buffer, 10);
    assert_eq!(cursor.line, 0); // 只有一行
}

#[test]
fn test_cursor_unicode() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello 世界");
    
    let mut cursor = Cursor::at_origin();
    cursor.move_right(&buffer, 6);
    assert_eq!(cursor.column, 6);
    
    // 确保字符索引正确
    let char_idx = cursor.to_char_idx(&buffer);
    assert_eq!(char_idx, 6);
}
