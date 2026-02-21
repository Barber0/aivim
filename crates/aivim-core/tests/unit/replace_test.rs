//! Replace 模块单元测试
//!
//! 对应源文件: src/replace.rs
//! 测试范围: 替换操作

use aivim_core::buffer::{Buffer, BufferId};
use aivim_core::replace::{parse_substitute_command, replace_in_buffer};

#[test]
fn test_replace_first() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world hello\n");

    let result = replace_in_buffer(&mut buffer, "hello", "hi", false, None);

    assert_eq!(result.count, 1);
    assert!(result.new_text.contains("hi world hello"));
}

#[test]
fn test_replace_global() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world hello\n");

    let result = replace_in_buffer(&mut buffer, "hello", "hi", true, None);

    assert_eq!(result.count, 2);
    assert!(result.new_text.contains("hi world hi"));
}

#[test]
fn test_replace_empty_pattern() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world\n");

    let result = replace_in_buffer(&mut buffer, "", "hi", true, None);

    assert_eq!(result.count, 0);
    assert_eq!(result.new_text, buffer.to_string());
}

#[test]
fn test_replace_multiline() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello\nhello\nhello\n");

    let result = replace_in_buffer(&mut buffer, "hello", "hi", true, None);

    assert_eq!(result.count, 3);
    assert_eq!(result.new_text, "hi\nhi\nhi\n");
}

#[test]
fn test_parse_substitute_basic() {
    let result = parse_substitute_command(":s/old/new");
    assert_eq!(result, Some(("old".to_string(), "new".to_string(), false, false)));
}

#[test]
fn test_parse_substitute_global() {
    let result = parse_substitute_command(":s/old/new/g");
    assert_eq!(result, Some(("old".to_string(), "new".to_string(), true, false)));
}

#[test]
fn test_parse_substitute_full_file() {
    let result = parse_substitute_command(":%s/old/new/g");
    assert_eq!(result, Some(("old".to_string(), "new".to_string(), true, true)));
}

#[test]
fn test_parse_substitute_no_colon() {
    let result = parse_substitute_command("s/old/new");
    assert_eq!(result, Some(("old".to_string(), "new".to_string(), false, false)));
}

#[test]
fn test_parse_substitute_invalid() {
    let result = parse_substitute_command(":invalid/command");
    assert_eq!(result, None);
}

#[test]
fn test_parse_substitute_no_slash() {
    let result = parse_substitute_command(":soldnew");
    assert_eq!(result, None);
}
