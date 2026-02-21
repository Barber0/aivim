//! 系统剪贴板集成测试
//!
//! 对应功能: RegisterManager 的剪贴板支持 (* 和 + 寄存器)
//!
//! 注意: 这些测试需要图形环境支持剪贴板访问

use aivim_core::register::RegisterManager;

#[test]
#[ignore = "Requires GUI environment for clipboard access"]
fn test_clipboard_register_star_get() {
    let mut manager = RegisterManager::new();
    
    // 设置剪贴板内容
    manager.set_clipboard("clipboard content");
    
    // 通过 * 寄存器读取
    let reg = manager.get('*');
    assert!(reg.is_some());
    assert_eq!(reg.unwrap().content, "clipboard content");
}

#[test]
#[ignore = "Requires GUI environment for clipboard access"]
fn test_clipboard_register_plus_get() {
    let mut manager = RegisterManager::new();
    
    // 设置剪贴板内容
    manager.set_clipboard("plus register content");
    
    // 通过 + 寄存器读取
    let reg = manager.get('+');
    assert!(reg.is_some());
    assert_eq!(reg.unwrap().content, "plus register content");
}

#[test]
#[ignore = "Requires GUI environment for clipboard access"]
fn test_clipboard_register_star_set() {
    let mut manager = RegisterManager::new();
    
    // 通过 * 寄存器设置
    manager.set('*', "star register content", false);
    
    // 验证可以通过剪贴板读取
    let clipboard_content = manager.get_clipboard();
    assert_eq!(clipboard_content, Some("star register content".to_string()));
}

#[test]
#[ignore = "Requires GUI environment for clipboard access"]
fn test_clipboard_register_plus_set() {
    let mut manager = RegisterManager::new();
    
    // 通过 + 寄存器设置
    manager.set('+', "plus content", false);
    
    // 验证可以通过剪贴板读取
    let clipboard_content = manager.get_clipboard();
    assert_eq!(clipboard_content, Some("plus content".to_string()));
}

#[test]
#[ignore = "Requires GUI environment for clipboard access"]
fn test_clipboard_register_mirror() {
    // * 和 + 寄存器应该指向同一个剪贴板
    let mut manager = RegisterManager::new();
    
    // 通过 * 设置
    manager.set('*', "mirrored content", false);
    
    // 通过 + 读取应该得到相同内容
    let plus_reg = manager.get('+');
    assert!(plus_reg.is_some());
    assert_eq!(plus_reg.unwrap().content, "mirrored content");
}

#[test]
#[ignore = "Requires GUI environment for clipboard access"]
fn test_clipboard_unicode() {
    let mut manager = RegisterManager::new();
    
    // 测试 Unicode 内容
    let unicode_content = "Hello 世界 🌍";
    manager.set_clipboard(unicode_content);
    
    let retrieved = manager.get_clipboard();
    assert_eq!(retrieved, Some(unicode_content.to_string()));
}

#[test]
#[ignore = "Requires GUI environment for clipboard access"]
fn test_clipboard_multiline() {
    let mut manager = RegisterManager::new();
    
    // 测试多行内容
    let multiline = "line1\nline2\nline3";
    manager.set_clipboard(multiline);
    
    let retrieved = manager.get_clipboard();
    assert_eq!(retrieved, Some(multiline.to_string()));
}
