//! Cursor 模块单元测试
//!
//! 对应源文件: src/cursor.rs
//! 测试范围: 光标位置管理、移动、字符索引转换

use aivim_core::buffer::{Buffer, BufferId};
use aivim_core::cursor::Cursor;

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
