//! Register 模块单元测试
//!
//! 对应源文件: src/register.rs
//! 测试范围: 寄存器管理（无名、数字、命名、搜索寄存器）

use aivim_core::register::RegisterManager;

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
fn test_named_registers() {
    let mut manager = RegisterManager::new();

    manager.set('a', "content a", false);
    assert_eq!(manager.get('a').unwrap().content, "content a");

    // 大写表示追加
    manager.set('A', " appended", false);
    assert_eq!(manager.get('a').unwrap().content, "content a appended");
}

#[test]
fn test_search_register() {
    let mut manager = RegisterManager::new();

    manager.set_search("pattern");
    assert_eq!(manager.get_search(), "pattern");
    assert_eq!(manager.get('/').unwrap().content, "pattern");
}
