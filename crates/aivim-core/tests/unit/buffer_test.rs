//! Buffer 模块单元测试
//!
//! 对应源文件: src/buffer.rs
//! 测试范围: 缓冲区创建、插入、删除、行操作

use aivim_core::buffer::{Buffer, BufferId};

#[test]
fn test_buffer_creation() {
    let buffer = Buffer::new(BufferId::new(0));
    assert!(buffer.is_empty());
    assert!(!buffer.is_modified());
}

#[test]
fn test_buffer_insert() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello, World!");
    assert_eq!(buffer.to_string(), "Hello, World!");
    assert!(buffer.is_modified());
}

#[test]
fn test_buffer_remove() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "Hello, World!");
    buffer.remove(7, 5);
    assert_eq!(buffer.to_string(), "Hello, !");
}

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
