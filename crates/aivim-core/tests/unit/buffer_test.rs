//! Buffer 模块单元测试
//!
//! 对应源文件: src/buffer.rs
//! 测试范围: 缓冲区创建、插入、删除、行操作、文件操作

use aivim_core::buffer::{Buffer, BufferId};
use std::io::Write;
use std::path::PathBuf;

// ==================== 基本操作测试 ====================

#[test]
fn test_buffer_creation() {
    let buffer = Buffer::new(BufferId::new(0));
    assert!(buffer.is_empty());
    assert!(!buffer.is_modified());
    assert_eq!(buffer.len_chars(), 0);
    assert_eq!(buffer.len_lines(), 1); // 空缓冲区也有一行
}

#[test]
fn test_buffer_id() {
    let buffer = Buffer::new(BufferId::new(42));
    assert_eq!(buffer.id().as_usize(), 42);
}

#[test]
fn test_buffer_insert() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello, World!");
    assert_eq!(buffer.to_string(), "Hello, World!");
    assert!(buffer.is_modified());
    assert_eq!(buffer.len_chars(), 13);
}

#[test]
fn test_buffer_insert_at_middle() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello World!");
    buffer.insert(6, "Beautiful ");
    assert_eq!(buffer.to_string(), "Hello Beautiful World!");
}

#[test]
fn test_buffer_insert_char() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hell World!");
    buffer.insert_char(4, 'o');
    assert_eq!(buffer.to_string(), "Hello World!");
}

#[test]
fn test_buffer_remove() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello, World!");
    buffer.remove(7, 5);
    assert_eq!(buffer.to_string(), "Hello, !");
}

#[test]
fn test_buffer_remove_char() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Helo World!");
    let removed = buffer.remove_char(2);
    assert_eq!(removed, Some('l'));
    assert_eq!(buffer.to_string(), "Heo World!");
}

#[test]
fn test_buffer_remove_char_out_of_bounds() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello");
    let removed = buffer.remove_char(10);
    assert_eq!(removed, None);
}

// ==================== 行操作测试 ====================

#[test]
fn test_buffer_line_operations() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    assert_eq!(buffer.len_lines(), 3);
    
    let line0 = buffer.line(0).unwrap();
    assert_eq!(line0.to_string(), "Line 1\n");
    
    let line1 = buffer.line(1).unwrap();
    assert_eq!(line1.to_string(), "Line 2\n");
    
    let line2 = buffer.line(2).unwrap();
    assert_eq!(line2.to_string(), "Line 3");
}

#[test]
fn test_buffer_line_out_of_bounds() {
    let buffer = Buffer::new(BufferId::new(0));
    assert!(buffer.line(100).is_none());
}

#[test]
fn test_buffer_line_len() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello\nWorld!");
    assert_eq!(buffer.line_len(0), 6); // "Hello\n"
    assert_eq!(buffer.line_len(1), 6); // "World!"
}

// ==================== 字符索引转换测试 ====================

#[test]
fn test_buffer_line_to_char() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    assert_eq!(buffer.line_to_char(0), 0);
    assert_eq!(buffer.line_to_char(1), 7); // "Line 1\n"
    assert_eq!(buffer.line_to_char(2), 14); // "Line 1\nLine 2\n"
}

#[test]
fn test_buffer_char_to_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Line 1\nLine 2\nLine 3");
    
    assert_eq!(buffer.char_to_line(0), 0);
    assert_eq!(buffer.char_to_line(6), 0);
    assert_eq!(buffer.char_to_line(7), 1);
    assert_eq!(buffer.char_to_line(10), 1);
}

// ==================== 字符访问测试 ====================

#[test]
fn test_buffer_char_access() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello");
    
    assert_eq!(buffer.char(0), 'H');
    assert_eq!(buffer.char(4), 'o');
}

#[test]
fn test_buffer_slice() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello, World!");
    
    let slice = buffer.slice(0..5);
    assert_eq!(slice.to_string(), "Hello");
    
    let slice = buffer.slice(7..12);
    assert_eq!(slice.to_string(), "World");
}

// ==================== 只读模式测试 ====================

#[test]
fn test_buffer_read_only() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello");
    buffer.save_as(std::path::PathBuf::from("/tmp/test_ro.txt").as_path()).ok(); // 重置 modified 状态
    
    buffer.set_read_only(true);
    assert!(buffer.is_read_only());
    assert!(!buffer.is_modified()); // 确保初始状态是未修改
    
    // 尝试在只读模式下修改
    buffer.insert(5, " World");
    assert_eq!(buffer.to_string(), "Hello"); // 不应该改变
    assert!(!buffer.is_modified()); // 不应该标记为修改
    
    // 清理
    std::fs::remove_file("/tmp/test_ro.txt").ok();
}

#[test]
fn test_buffer_read_only_remove() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello World");
    buffer.set_read_only(true);
    
    buffer.remove(0, 5);
    assert_eq!(buffer.to_string(), "Hello World"); // 不应该改变
}

// ==================== 文件路径测试 ====================

#[test]
fn test_buffer_file_path() {
    let mut buffer = Buffer::new(BufferId::new(0));
    assert!(buffer.file_path().is_none());
    
    let path = PathBuf::from("/tmp/test.txt");
    buffer.set_file_path(path.clone());
    assert_eq!(buffer.file_path(), Some(path.as_path()));
}

#[test]
fn test_buffer_new_with_path() {
    let path = PathBuf::from("/tmp/test.txt");
    let buffer = Buffer::new_with_path(BufferId::new(0), &path);
    assert_eq!(buffer.file_path(), Some(path.as_path()));
    assert!(buffer.is_empty());
}

// ==================== 文件操作测试 ====================

#[test]
fn test_buffer_save_and_load() {
    let temp_path = "/tmp/aivim_test_buffer.txt";
    
    // 创建并保存缓冲区
    {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "Hello, World!");
        buffer.set_file_path(PathBuf::from(temp_path));
        buffer.save().unwrap();
        assert!(!buffer.is_modified());
    }
    
    // 从文件加载
    {
        let buffer = Buffer::from_file(BufferId::new(1), PathBuf::from(temp_path).as_path()).unwrap();
        assert_eq!(buffer.to_string(), "Hello, World!\n"); // 保存时自动添加换行
        assert!(!buffer.is_modified());
    }
    
    // 清理
    std::fs::remove_file(temp_path).unwrap();
}

#[test]
fn test_buffer_save_as() {
    let temp_path1 = "/tmp/aivim_test_buffer1.txt";
    let temp_path2 = "/tmp/aivim_test_buffer2.txt";
    
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Test content");
    buffer.set_file_path(PathBuf::from(temp_path1));
    buffer.save().unwrap();
    
    // 另存为
    buffer.save_as(PathBuf::from(temp_path2).as_path()).unwrap();
    
    // 验证新文件
    let content = std::fs::read_to_string(temp_path2).unwrap();
    assert_eq!(content, "Test content\n");
    
    // 清理
    std::fs::remove_file(temp_path1).unwrap();
    std::fs::remove_file(temp_path2).unwrap();
}

#[test]
fn test_buffer_save_no_path() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Test");
    
    let result = buffer.save();
    assert!(result.is_err());
}

// ==================== 边界情况测试 ====================

#[test]
fn test_buffer_empty_operations() {
    let mut buffer = Buffer::new(BufferId::new(0));
    
    // 空缓冲区操作
    assert!(buffer.is_empty());
    assert_eq!(buffer.len_chars(), 0);
    assert_eq!(buffer.len_lines(), 1);
    
    // 在空缓冲区中删除
    buffer.remove(0, 10);
    assert!(buffer.is_empty());
}

#[test]
fn test_buffer_single_line() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Single line");
    
    assert_eq!(buffer.len_lines(), 1);
    assert_eq!(buffer.line(0).unwrap().to_string(), "Single line");
}

#[test]
fn test_buffer_multiple_inserts() {
    let mut buffer = Buffer::new(BufferId::new(0));
    
    buffer.insert(0, "Hello");
    buffer.insert(5, " ");
    buffer.insert(6, "World");
    buffer.insert(11, "!");
    
    assert_eq!(buffer.to_string(), "Hello World!");
}

#[test]
fn test_buffer_large_content() {
    let mut buffer = Buffer::new(BufferId::new(0));
    let large_text = "a".repeat(10000);
    buffer.insert(0, &large_text);
    
    assert_eq!(buffer.len_chars(), 10000);
    assert!(!buffer.is_empty());
}

#[test]
fn test_buffer_unicode() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello 世界! 🌍");
    
    assert_eq!(buffer.to_string(), "Hello 世界! 🌍");
    assert_eq!(buffer.len_chars(), 11); // 字符数，不是字节数
}
