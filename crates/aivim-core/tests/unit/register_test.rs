//! Register 模块单元测试
//!
//! 对应源文件: src/register.rs
//! 测试范围: 寄存器管理（无名、数字、命名、搜索、只读寄存器）

use aivim_core::register::{Register, RegisterManager};

// ==================== 寄存器基本操作测试 ====================

#[test]
fn test_register_creation() {
    let reg = Register::new('a', "content", false);
    assert_eq!(reg.name, 'a');
    assert_eq!(reg.content, "content");
    assert!(!reg.linewise);
    assert!(!reg.is_empty());
}

#[test]
fn test_register_empty() {
    let reg = Register::empty('a');
    assert!(reg.is_empty());
    assert_eq!(reg.content, "");
}

#[test]
fn test_register_lines() {
    let reg = Register::new('a', "line1\nline2\nline3", true);
    let lines = reg.lines();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "line3");
}

// ==================== 无名寄存器测试 ====================

#[test]
fn test_unnamed_register_yank() {
    let mut manager = RegisterManager::new();
    // 复制操作 - 只更新 0 号，不移动数字寄存器
    manager.set_unnamed_yank("hello", false);

    assert_eq!(manager.get('"').unwrap().content, "hello");
    assert_eq!(manager.get('0').unwrap().content, "hello");
}

#[test]
fn test_unnamed_register_delete() {
    let mut manager = RegisterManager::new();
    // 删除操作 - 更新 0 号，移动数字寄存器
    manager.set_unnamed_delete("hello", false);

    assert_eq!(manager.get('"').unwrap().content, "hello");
    assert_eq!(manager.get('0').unwrap().content, "hello");
}

#[test]
fn test_unnamed_register_linewise() {
    let mut manager = RegisterManager::new();
    manager.set_unnamed_yank("line content", true);
    
    let reg = manager.get('"').unwrap();
    assert!(reg.linewise);
}

// ==================== 数字寄存器测试 ====================

#[test]
fn test_numbered_registers_delete() {
    let mut manager = RegisterManager::new();

    // 使用删除操作，数字寄存器应该移动
    manager.set_unnamed_delete("first", false);
    manager.set_unnamed_delete("second", false);
    manager.set_unnamed_delete("third", false);

    // 0号应该是最新的
    assert_eq!(manager.get('0').unwrap().content, "third");
    // 1号应该是上一个
    assert_eq!(manager.get('1').unwrap().content, "second");
    // 2号应该是第一个
    assert_eq!(manager.get('2').unwrap().content, "first");
}

#[test]
fn test_numbered_registers_yank() {
    let mut manager = RegisterManager::new();

    // 使用复制操作，数字寄存器不应该移动
    manager.set_unnamed_yank("first", false);
    manager.set_unnamed_yank("second", false);
    manager.set_unnamed_yank("third", false);

    // 0号应该是最新的
    assert_eq!(manager.get('0').unwrap().content, "third");
    // 1-9号应该仍然是空的（因为复制不移动数字寄存器）
    assert!(manager.get('1').unwrap().content.is_empty());
    assert!(manager.get('2').unwrap().content.is_empty());
}

#[test]
fn test_numbered_registers_shift() {
    let mut manager = RegisterManager::new();

    // 填充所有数字寄存器
    for i in 0..10 {
        manager.set_unnamed_delete(&format!("content{}", i), false);
    }

    // 9号应该是最旧的（content0）
    assert_eq!(manager.get('9').unwrap().content, "content0");
    // 0号应该是最新的（content9）
    assert_eq!(manager.get('0').unwrap().content, "content9");
}

// ==================== 命名寄存器测试 ====================

#[test]
fn test_named_registers() {
    let mut manager = RegisterManager::new();

    manager.set('a', "content a", false);
    assert_eq!(manager.get('a').unwrap().content, "content a");

    // 大写表示追加
    manager.set('A', " appended", false);
    assert_eq!(manager.get('a').unwrap().content, "content a appended");
}

#[test]
fn test_named_registers_all_letters() {
    let mut manager = RegisterManager::new();

    // 测试所有小写字母
    for c in 'a'..='z' {
        manager.set(c, &format!("content {}", c), false);
        assert_eq!(manager.get(c).unwrap().content, format!("content {}", c));
    }
}

#[test]
fn test_named_registers_append_multiple() {
    let mut manager = RegisterManager::new();

    manager.set('a', "first", false);
    manager.set('A', " second", false);
    manager.set('A', " third", false);
    
    assert_eq!(manager.get('a').unwrap().content, "first second third");
}

#[test]
fn test_named_registers_overwrite() {
    let mut manager = RegisterManager::new();

    manager.set('a', "original", false);
    manager.set('a', "new", false); // 小写覆盖
    
    assert_eq!(manager.get('a').unwrap().content, "new");
}

// ==================== 小删除寄存器测试 ====================

#[test]
fn test_small_delete_register() {
    let mut manager = RegisterManager::new();
    
    manager.set_small_delete("small");
    assert_eq!(manager.get('-').unwrap().content, "small");
}

// ==================== 搜索寄存器测试 ====================

#[test]
fn test_search_register() {
    let mut manager = RegisterManager::new();

    manager.set_search("pattern");
    assert_eq!(manager.get_search(), "pattern");
    assert_eq!(manager.get('/').unwrap().content, "pattern");
}

#[test]
fn test_search_register_update() {
    let mut manager = RegisterManager::new();

    manager.set_search("first");
    assert_eq!(manager.get_search(), "first");
    
    manager.set_search("second");
    assert_eq!(manager.get_search(), "second");
}

// ==================== 只读寄存器测试 ====================

#[test]
fn test_readonly_registers_exist() {
    let manager = RegisterManager::new();

    // 只读寄存器应该存在
    assert!(manager.get('%').is_some());
    assert!(manager.get('#').is_some());
    assert!(manager.get(':').is_some());
    assert!(manager.get('.').is_some());
}

#[test]
fn test_readonly_registers_set() {
    let mut manager = RegisterManager::new();

    manager.set_readonly('%', "current_file.txt");
    assert_eq!(manager.get('%').unwrap().content, "current_file.txt");
    
    manager.set_readonly('#', "alternate_file.txt");
    assert_eq!(manager.get('#').unwrap().content, "alternate_file.txt");
}

#[test]
fn test_readonly_registers_invalid() {
    let mut manager = RegisterManager::new();

    // 尝试设置不存在的只读寄存器
    manager.set_readonly('x', "invalid");
    // 不应该创建新的只读寄存器
    assert!(manager.get('x').is_none());
}

// ==================== 边界情况测试 ====================

#[test]
fn test_invalid_register_name() {
    let manager = RegisterManager::new();

    // 无效寄存器名应该返回 None
    assert!(manager.get('!').is_none());
    assert!(manager.get('@').is_none());
    assert!(manager.get(' ').is_none());
}

#[test]
fn test_empty_content() {
    let mut manager = RegisterManager::new();

    manager.set('a', "", false);
    assert!(manager.get('a').unwrap().is_empty());
}

#[test]
fn test_unicode_content() {
    let mut manager = RegisterManager::new();

    manager.set('a', "Hello 世界 🌍", false);
    assert_eq!(manager.get('a').unwrap().content, "Hello 世界 🌍");
}

#[test]
fn test_multiline_content() {
    let mut manager = RegisterManager::new();

    let content = "line1\nline2\nline3";
    manager.set('a', content, true);
    
    let reg = manager.get('a').unwrap();
    assert!(reg.linewise);
    assert_eq!(reg.content, content);
    assert_eq!(reg.lines().len(), 3);
}

// ==================== 默认实现测试 ====================

#[test]
fn test_default_implementation() {
    let manager: RegisterManager = Default::default();
    
    // 默认应该创建空的无名寄存器
    assert!(manager.get('"').unwrap().is_empty());
    // 数字寄存器应该存在
    assert!(manager.get('0').is_some());
    assert!(manager.get('9').is_some());
}
