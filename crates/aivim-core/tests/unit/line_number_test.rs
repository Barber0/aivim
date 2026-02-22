//! 行号显示功能单元测试
//!
//! 对应源文件: src/editor.rs (EditorOptions)

use aivim_core::editor::{Editor, EditorOptions};

#[test]
fn test_default_options() {
    let editor = Editor::new();
    let options = editor.options();
    
    assert!(!options.number, "默认不显示绝对行号");
    assert!(!options.relativenumber, "默认不显示相对行号");
    assert!(!options.cursorline, "默认不高亮当前行");
}

#[test]
fn test_set_number_option() {
    let mut editor = Editor::new();
    
    // 启用绝对行号
    editor.execute_command("set number").unwrap();
    assert!(editor.options().number);
    
    // 禁用绝对行号
    editor.execute_command("set nonumber").unwrap();
    assert!(!editor.options().number);
    
    // 使用缩写
    editor.execute_command("set nu").unwrap();
    assert!(editor.options().number);
    
    editor.execute_command("set nonu").unwrap();
    assert!(!editor.options().number);
}

#[test]
fn test_set_relativenumber_option() {
    let mut editor = Editor::new();
    
    // 启用相对行号
    editor.execute_command("set relativenumber").unwrap();
    assert!(editor.options().relativenumber);
    
    // 禁用相对行号
    editor.execute_command("set norelativenumber").unwrap();
    assert!(!editor.options().relativenumber);
    
    // 使用缩写
    editor.execute_command("set rnu").unwrap();
    assert!(editor.options().relativenumber);
    
    editor.execute_command("set nornu").unwrap();
    assert!(!editor.options().relativenumber);
}

#[test]
fn test_set_cursorline_option() {
    let mut editor = Editor::new();
    
    // 启用光标行高亮
    editor.execute_command("set cursorline").unwrap();
    assert!(editor.options().cursorline);
    
    // 禁用光标行高亮
    editor.execute_command("set nocursorline").unwrap();
    assert!(!editor.options().cursorline);
}

#[test]
fn test_combined_options() {
    let mut editor = Editor::new();
    
    // 同时启用绝对行号和相对行号（混合模式）
    editor.execute_command("set number").unwrap();
    editor.execute_command("set relativenumber").unwrap();
    
    assert!(editor.options().number);
    assert!(editor.options().relativenumber);
    
    // 禁用其中一个
    editor.execute_command("set nonumber").unwrap();
    assert!(!editor.options().number);
    assert!(editor.options().relativenumber);
}

#[test]
fn test_options_persistence() {
    let mut editor = Editor::new();
    
    // 设置选项
    editor.execute_command("set number").unwrap();
    editor.execute_command("set relativenumber").unwrap();
    editor.execute_command("set cursorline").unwrap();
    
    // 验证选项保持
    let options = editor.options();
    assert!(options.number);
    assert!(options.relativenumber);
    assert!(options.cursorline);
}

#[test]
fn test_unknown_option_error() {
    let mut editor = Editor::new();
    
    // 未知选项应该返回错误
    let result = editor.execute_command("set unknown_option");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown option"));
}
