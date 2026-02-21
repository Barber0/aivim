//! Editor 删除操作单元测试
//!
//! 测试 delete_to_motion_with_register 等编辑器删除功能

use aivim_core::buffer::{Buffer, BufferId};
use aivim_core::cursor::Cursor;
use aivim_core::editor::Editor;
use aivim_core::motion::Motion;

#[test]
fn test_delete_word_forward_no_merge() {
    let mut editor = Editor::new();
    
    // 插入内容到初始缓冲区
    {
        let buffer = editor.current_buffer_mut();
        buffer.insert(0, "hello world\nnext line");
    }
    
    // 光标在 "world" 前面
    *editor.cursor_mut() = Cursor::new(0, 6);
    
    // 执行 dw (delete word forward)
    let result = editor.delete_to_motion(Motion::WordForward);
    
    // 应该删除 "world"，但不删除换行符
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "world");
    
    // 验证换行符还在
    let buffer = editor.current_buffer();
    assert!(buffer.to_string().contains('\n'));
    assert_eq!(buffer.to_string(), "hello \nnext line");
}

#[test]
fn test_delete_word_forward_at_line_end() {
    let mut editor = Editor::new();
    
    {
        let buffer = editor.current_buffer_mut();
        buffer.insert(0, "hello\nworld");
    }
    
    // 光标在第一行的单词上
    *editor.cursor_mut() = Cursor::at_origin();
    
    // 执行 dw
    let result = editor.delete_to_motion(Motion::WordForward);
    
    // 应该删除 "hello"，但保留换行符
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "hello");
    
    let buffer = editor.current_buffer();
    assert_eq!(buffer.to_string(), "\nworld");
}

#[test]
fn test_delete_word_backward_no_merge() {
    let mut editor = Editor::new();
    
    {
        let buffer = editor.current_buffer_mut();
        buffer.insert(0, "hello\nworld test");
    }
    
    // 光标在第二行的 "test" 上 (列号应该是 "world " 之后，即 6)
    *editor.cursor_mut() = Cursor::new(1, 6);
    
    // 执行 db (delete word backward)
    let result = editor.delete_to_motion(Motion::WordBackward);
    
    // 应该删除 "test" 前面的 "world "，但不删除换行符
    assert!(result.is_some());
    // WordBackward 删除的是前一个单词，所以删除的是 "world "
    assert_eq!(result.unwrap(), "world ");
    
    let buffer = editor.current_buffer();
    assert!(buffer.to_string().contains('\n'));
    assert_eq!(buffer.to_string(), "hello\ntest");
}

#[test]
fn test_delete_word_forward_multiple_words() {
    let mut editor = Editor::new();
    
    {
        let buffer = editor.current_buffer_mut();
        buffer.insert(0, "one two three\nnext");
    }
    
    // 光标在开头
    *editor.cursor_mut() = Cursor::at_origin();
    
    // 删除第一个单词
    let result = editor.delete_to_motion(Motion::WordForward);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "one ");
    
    let buffer = editor.current_buffer();
    assert_eq!(buffer.to_string(), "two three\nnext");
}

#[test]
fn test_delete_word_forward_empty_result() {
    let mut editor = Editor::new();
    
    {
        let buffer = editor.current_buffer_mut();
        buffer.insert(0, "hello\n");
    }
    
    // 光标在行尾（换行符前）
    *editor.cursor_mut() = Cursor::new(0, 5);
    
    // 执行 dw - 应该删除到行尾（不包括换行符）
    let result = editor.delete_to_motion(Motion::WordForward);
    
    // 没有可删除的内容（换行符前没有单词字符）
    assert!(result.is_none());
}
