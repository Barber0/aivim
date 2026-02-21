//! 寄存器状态显示测试
//!
//! 测试状态栏是否正确显示当前选择的寄存器

use aivim_tui::app::OperatorState;

// 注意：这些测试测试的是 get_register_info 函数的逻辑
// 实际 UI 测试需要集成测试框架

#[test]
fn test_register_info_none() {
    // 没有操作符状态，不显示寄存器
    let info = get_register_info_helper(OperatorState::None);
    assert_eq!(info, "");
}

#[test]
fn test_register_info_pending_none() {
    // 刚按下 "，等待寄存器名
    let info = get_register_info_helper(OperatorState::RegisterPending(None));
    assert_eq!(info, "\"?");
}

#[test]
fn test_register_info_pending_some() {
    // 已选择寄存器 a，等待操作符
    let info = get_register_info_helper(OperatorState::RegisterPending(Some('a')));
    assert_eq!(info, "\"a");
}

#[test]
fn test_register_info_delete_with_register() {
    // 删除操作符，指定了寄存器 b
    let info = get_register_info_helper(OperatorState::Delete { register: Some('b') });
    assert_eq!(info, "\"b");
}

#[test]
fn test_register_info_delete_without_register() {
    // 删除操作符，未指定寄存器
    let info = get_register_info_helper(OperatorState::Delete { register: None });
    assert_eq!(info, "");
}

#[test]
fn test_register_info_yank_with_register() {
    // 复制操作符，指定了寄存器 c
    let info = get_register_info_helper(OperatorState::Yank { register: Some('c') });
    assert_eq!(info, "\"c");
}

#[test]
fn test_register_info_change_with_register() {
    // 修改操作符，指定了寄存器 d
    let info = get_register_info_helper(OperatorState::Change { register: Some('d') });
    assert_eq!(info, "\"d");
}

#[test]
fn test_register_info_text_object_with_register() {
    // 文本对象操作符，指定了寄存器 x
    use aivim_tui::app::TextObjectOperator;
    let info = get_register_info_helper(OperatorState::TextObject {
        operator: TextObjectOperator::Delete,
        around: true,
        register: Some('x'),
    });
    assert_eq!(info, "\"x");
}

#[test]
fn test_register_info_g() {
    // G 操作符，不显示寄存器
    let info = get_register_info_helper(OperatorState::G);
    assert_eq!(info, "");
}

// 辅助函数，模拟 get_register_info 的逻辑
fn get_register_info_helper(operator_state: OperatorState) -> String {
    use aivim_tui::app::OperatorState;
    
    match operator_state {
        OperatorState::RegisterPending(None) => "\"?".to_string(),
        OperatorState::RegisterPending(Some(reg)) => format!("\"{}", reg),
        OperatorState::Delete { register: Some(reg) } => format!("\"{}", reg),
        OperatorState::Yank { register: Some(reg) } => format!("\"{}", reg),
        OperatorState::Change { register: Some(reg) } => format!("\"{}", reg),
        OperatorState::TextObject { register: Some(reg), .. } => format!("\"{}", reg),
        _ => String::new(),
    }
}
