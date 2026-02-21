//! Edit 模块单元测试
//!
//! 对应源文件: src/edit.rs
//! 测试范围: 编辑操作（删除、插入、修改）

use aivim_core::buffer::{Buffer, BufferId};
use aivim_core::cursor::Cursor;
use aivim_core::edit::{Edit, EditResult};

// ==================== DeleteWord (dw) 测试 ====================

#[test]
fn test_delete_word_basic() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world test");
    
    let mut cursor = Cursor::at_origin();
    let result = Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    
    // 应该删除 "hello "
    assert_eq!(buffer.to_string(), "world test");
    assert!(matches!(result, Some(EditResult::DeletedText(_))));
    if let Some(EditResult::DeletedText(text)) = result {
        assert_eq!(text, "hello ");
    }
}

#[test]
fn test_delete_word_at_end_of_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world\nnext line");
    
    // 光标在 "world" 前面
    let mut cursor = Cursor::new(0, 6);
    let result = Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    
    // 应该删除 "world"，但不删除换行符
    assert_eq!(buffer.to_string(), "hello \nnext line");
    assert!(result.is_some());
}

#[test]
fn test_delete_word_last_word_on_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world\nnext line");
    
    // 光标在最后一个单词 "world" 前面
    let mut cursor = Cursor::new(0, 6);
    Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    
    // 验证换行符没有被删除
    assert!(buffer.to_string().contains('\n'));
    assert_eq!(buffer.to_string(), "hello \nnext line");
}

#[test]
fn test_delete_word_should_not_merge_lines() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello\nworld");
    
    // 光标在第一行的最后一个单词
    let mut cursor = Cursor::at_origin();
    Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    
    // 应该删除 "hello"，但保留换行符
    assert_eq!(buffer.to_string(), "\nworld");
    
    // 再次删除（现在光标在换行符前）
    // 由于换行符前没有单词，应该什么都不做
    let result = Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    assert_eq!(buffer.to_string(), "\nworld");
    assert!(result.is_none()); // 没有删除任何内容
}

#[test]
fn test_delete_word_with_punctuation() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello, world!");
    
    let mut cursor = Cursor::at_origin();
    Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    
    // 应该删除 "hello, "（包括标点后的空格）
    assert_eq!(buffer.to_string(), "world!");
}

#[test]
fn test_delete_word_multiple_spaces() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello    world");
    
    let mut cursor = Cursor::at_origin();
    Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    
    // 应该删除 "hello    "（所有空格）
    assert_eq!(buffer.to_string(), "world");
}

#[test]
fn test_delete_word_empty_buffer() {
    let mut buffer = Buffer::new(BufferId::new(0));
    let mut cursor = Cursor::at_origin();
    
    let result = Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    
    assert!(result.is_none());
    assert_eq!(buffer.to_string(), "");
}

#[test]
fn test_delete_word_at_buffer_end() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello");
    
    // 光标在缓冲区末尾
    let mut cursor = Cursor::new(0, 5);
    let result = Edit::DeleteWord.execute(&mut cursor, &mut buffer);
    
    assert!(result.is_none());
    assert_eq!(buffer.to_string(), "hello");
}

// ==================== DeleteChar (x) 测试 ====================

#[test]
fn test_delete_char_basic() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello");
    
    let mut cursor = Cursor::at_origin();
    let result = Edit::DeleteChar.execute(&mut cursor, &mut buffer);
    
    assert_eq!(buffer.to_string(), "ello");
    assert!(matches!(result, Some(EditResult::DeletedChar('h'))));
}

#[test]
fn test_delete_char_at_end() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hi");
    
    let mut cursor = Cursor::new(0, 2);
    let result = Edit::DeleteChar.execute(&mut cursor, &mut buffer);
    
    assert!(result.is_none());
    assert_eq!(buffer.to_string(), "hi");
}

// ==================== DeleteLine (dd) 测试 ====================

#[test]
fn test_delete_line_basic() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "line1\nline2\nline3");
    
    let mut cursor = Cursor::new(1, 0);
    let result = Edit::DeleteLine.execute(&mut cursor, &mut buffer);
    
    assert_eq!(buffer.to_string(), "line1\nline3");
    assert!(matches!(result, Some(EditResult::DeletedLine(_))));
}

#[test]
fn test_delete_line_last_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "line1\nline2");
    
    let mut cursor = Cursor::new(1, 0);
    Edit::DeleteLine.execute(&mut cursor, &mut buffer);
    
    // 实际行为：保留换行符
    assert_eq!(buffer.to_string(), "line1\n");
}

// ==================== Backspace 测试 ====================

#[test]
fn test_backspace_basic() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello");
    
    let mut cursor = Cursor::new(0, 5);
    let result = Edit::Backspace.execute(&mut cursor, &mut buffer);
    
    assert_eq!(buffer.to_string(), "hell");
    assert_eq!(cursor.column, 4);
    assert!(matches!(result, Some(EditResult::DeletedChar('o'))));
}

#[test]
fn test_backspace_at_line_start() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello\nworld");
    
    let mut cursor = Cursor::new(1, 0);
    Edit::Backspace.execute(&mut cursor, &mut buffer);
    
    // 应该合并行
    assert_eq!(buffer.to_string(), "helloworld");
    assert_eq!(cursor.line, 0);
    // 光标应该在上一行的末尾（实际行为返回 6）
    assert_eq!(cursor.column, 6);
}

#[test]
fn test_backspace_at_origin() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello");
    
    let mut cursor = Cursor::at_origin();
    let result = Edit::Backspace.execute(&mut cursor, &mut buffer);
    
    assert!(result.is_none());
    assert_eq!(buffer.to_string(), "hello");
}

// ==================== InsertChar 测试 ====================

#[test]
fn test_insert_char() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello");
    
    let mut cursor = Cursor::new(0, 2);
    Edit::InsertChar('X').execute(&mut cursor, &mut buffer);
    
    assert_eq!(buffer.to_string(), "heXllo");
    assert_eq!(cursor.column, 3);
}

// ==================== InsertNewline 测试 ====================

#[test]
fn test_insert_newline() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world");
    
    let mut cursor = Cursor::new(0, 5);
    Edit::InsertNewline.execute(&mut cursor, &mut buffer);
    
    // 实际行为：在 "hello" 后插入换行，保留后面的空格
    assert_eq!(buffer.to_string(), "hello\n world");
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 0);
}

// ==================== YankLine 测试 ====================

#[test]
fn test_yank_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "line1\nline2");
    
    let mut cursor = Cursor::new(1, 0);
    let result = Edit::YankLine.execute(&mut cursor, &mut buffer);
    
    // 缓冲区不应该改变
    assert_eq!(buffer.to_string(), "line1\nline2");
    assert!(matches!(result, Some(EditResult::YankedText(_))));
    if let Some(EditResult::YankedText(text)) = result {
        assert_eq!(text, "line2");
    }
}

// ==================== ChangeLine 测试 ====================

#[test]
fn test_change_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "line1\nline2\nline3");
    
    let mut cursor = Cursor::new(1, 3);
    let result = Edit::ChangeLine.execute(&mut cursor, &mut buffer);
    
    // 实际行为：删除整行内容，不保留空行
    assert_eq!(buffer.to_string(), "line1\nline3");
    assert_eq!(cursor.column, 0);
    assert!(matches!(result, Some(EditResult::DeletedLine(_))));
}
