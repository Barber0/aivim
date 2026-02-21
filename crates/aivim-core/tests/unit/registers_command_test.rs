//! :registers 命令单元测试
//!
//! 对应功能: Editor::format_registers() 和 RegisterManager::get_all_registers()

use aivim_core::editor::Editor;
use aivim_core::register::RegisterManager;

#[test]
fn test_get_all_registers_empty() {
    let manager = RegisterManager::new();
    let registers = manager.get_all_registers();
    
    // 新创建的 RegisterManager 应该只有空的寄存器
    assert!(registers.is_empty());
}

#[test]
fn test_get_all_registers_unnamed() {
    let mut manager = RegisterManager::new();
    manager.set_unnamed_yank("test content", false);
    
    let registers = manager.get_all_registers();
    
    // 无名寄存器和 0 号寄存器都会存储内容
    assert!(registers.len() >= 1);
    
    // 检查是否包含无名寄存器
    let has_unnamed = registers.iter().any(|r| r.name == '"');
    assert!(has_unnamed);
    
    // 检查内容
    let unnamed = registers.iter().find(|r| r.name == '"').unwrap();
    assert_eq!(unnamed.content, "test content");
}

#[test]
fn test_get_all_registers_numbered() {
    let mut manager = RegisterManager::new();
    
    // 使用删除操作填充数字寄存器
    manager.set_unnamed_delete("first", false);
    manager.set_unnamed_delete("second", false);
    
    let registers = manager.get_all_registers();
    
    // 应该有: "", 0, 1 (无名寄存器 + 2个数字寄存器)
    assert!(registers.len() >= 3);
    
    // 检查是否包含 0 号寄存器（最新的）
    let has_zero = registers.iter().any(|r| r.name == '0');
    assert!(has_zero);
    
    // 0 号应该是最新的 "second"
    let zero_reg = registers.iter().find(|r| r.name == '0').unwrap();
    assert_eq!(zero_reg.content, "second");
}

#[test]
fn test_get_all_registers_named() {
    let mut manager = RegisterManager::new();
    
    manager.set('a', "content a", false);
    manager.set('z', "content z", false);
    manager.set('m', "content m", false);
    
    let registers = manager.get_all_registers();
    
    // 检查是否包含命名寄存器
    let has_a = registers.iter().any(|r| r.name == 'a');
    let has_z = registers.iter().any(|r| r.name == 'z');
    let has_m = registers.iter().any(|r| r.name == 'm');
    
    assert!(has_a);
    assert!(has_z);
    assert!(has_m);
}

#[test]
fn test_get_all_registers_search() {
    let mut manager = RegisterManager::new();
    manager.set_search("pattern");
    
    let registers = manager.get_all_registers();
    
    let has_search = registers.iter().any(|r| r.name == '/');
    assert!(has_search);
}

#[test]
fn test_get_all_registers_readonly() {
    let mut manager = RegisterManager::new();
    
    manager.set_readonly('%', "current.txt");
    manager.set_readonly('#', "alternate.txt");
    
    let registers = manager.get_all_registers();
    
    let has_percent = registers.iter().any(|r| r.name == '%');
    let has_hash = registers.iter().any(|r| r.name == '#');
    
    assert!(has_percent);
    assert!(has_hash);
}

#[test]
fn test_format_registers_empty() {
    let editor = Editor::new();
    let output = editor.format_registers();
    
    assert_eq!(output, "No registers");
}

#[test]
fn test_execute_registers_command() {
    let mut editor = Editor::new();
    
    // 执行 :registers 命令（没有寄存器内容）
    let result = editor.execute_command("registers");
    
    assert!(result.is_ok());
    // 现在 :registers 命令显示面板而不是消息
    assert!(editor.show_registers_panel());
}

#[test]
fn test_execute_reg_command_short() {
    let mut editor = Editor::new();
    
    // 执行 :reg 命令（缩写）
    let result = editor.execute_command("reg");
    
    assert!(result.is_ok());
    // 现在 :reg 命令显示面板而不是消息
    assert!(editor.show_registers_panel());
}
