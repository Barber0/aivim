//! 缓冲区管理单元测试
//!
//! 对应源文件: src/editor.rs (缓冲区管理相关方法)

use aivim_core::buffer::{Buffer, BufferId};
use aivim_core::cursor::Cursor;
use aivim_core::editor::Editor;

// ==================== 基础缓冲区操作测试 ====================

#[test]
fn test_editor_new_has_one_buffer() {
    let editor = Editor::new();
    assert_eq!(editor.buffer_count(), 1);
}

#[test]
fn test_current_buffer_id() {
    let editor = Editor::new();
    let id = editor.current_buffer_id();
    assert_eq!(id.as_usize(), 0);
}

// ==================== 缓冲区列表测试 ====================

#[test]
fn test_list_buffers_single() {
    let editor = Editor::new();
    let buffers = editor.list_buffers();
    
    assert_eq!(buffers.len(), 1);
    let (id, name, is_current) = &buffers[0];
    assert_eq!(id.as_usize(), 0);
    assert!(name.contains("缓冲区") || name == "");
    assert!(*is_current);
}

#[test]
fn test_format_buffer_list() {
    let editor = Editor::new();
    let output = editor.format_buffer_list();
    
    assert!(output.contains("缓冲区列表"));
    assert!(output.contains("%")); // 当前缓冲区标记
}

// ==================== 缓冲区切换测试 ====================

#[test]
fn test_switch_buffer() {
    let mut editor = Editor::new();
    
    // 创建第二个缓冲区
    {
        let buffer = Buffer::new(BufferId::new(1));
        editor.buffers_mut().insert(BufferId::new(1), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(1), Cursor::at_origin());
    }
    
    // 切换到缓冲区 1
    let result = editor.switch_buffer(BufferId::new(1));
    assert!(result.is_ok());
    assert_eq!(editor.current_buffer_id().as_usize(), 1);
}

#[test]
fn test_switch_buffer_nonexistent() {
    let mut editor = Editor::new();
    
    let result = editor.switch_buffer(BufferId::new(999));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("不存在"));
}

#[test]
fn test_next_buffer() {
    let mut editor = Editor::new();
    
    // 创建第二个缓冲区
    {
        let buffer = Buffer::new(BufferId::new(1));
        editor.buffers_mut().insert(BufferId::new(1), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(1), Cursor::at_origin());
    }
    
    // 切换到下一个缓冲区
    let result = editor.next_buffer();
    assert!(result.is_ok());
    assert_eq!(editor.current_buffer_id().as_usize(), 1);
}

#[test]
fn test_next_buffer_single() {
    let mut editor = Editor::new();
    
    let result = editor.next_buffer();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("没有其他缓冲区"));
}

#[test]
fn test_prev_buffer() {
    let mut editor = Editor::new();
    
    // 创建第二个缓冲区并切换到它
    {
        let buffer = Buffer::new(BufferId::new(1));
        editor.buffers_mut().insert(BufferId::new(1), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(1), Cursor::at_origin());
    }
    editor.switch_buffer(BufferId::new(1)).unwrap();
    
    // 切换到上一个缓冲区
    let result = editor.prev_buffer();
    assert!(result.is_ok());
    assert_eq!(editor.current_buffer_id().as_usize(), 0);
}

#[test]
fn test_buffer_navigation_wraps() {
    let mut editor = Editor::new();
    
    // 创建多个缓冲区
    for i in 1..=3 {
        let buffer = Buffer::new(BufferId::new(i));
        editor.buffers_mut().insert(BufferId::new(i), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(i), Cursor::at_origin());
    }
    
    // 在最后一个缓冲区时执行 next 应该回到第一个
    editor.switch_buffer(BufferId::new(3)).unwrap();
    editor.next_buffer().unwrap();
    assert_eq!(editor.current_buffer_id().as_usize(), 0);
    
    // 在第一个缓冲区时执行 prev 应该到最后一个
    editor.prev_buffer().unwrap();
    assert_eq!(editor.current_buffer_id().as_usize(), 3);
}

// ==================== 缓冲区删除测试 ====================

#[test]
fn test_delete_buffer() {
    let mut editor = Editor::new();
    
    // 创建第二个缓冲区
    {
        let buffer = Buffer::new(BufferId::new(1));
        editor.buffers_mut().insert(BufferId::new(1), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(1), Cursor::at_origin());
    }
    
    assert_eq!(editor.buffer_count(), 2);
    
    // 删除第二个缓冲区
    let result = editor.delete_buffer(BufferId::new(1));
    assert!(result.is_ok());
    assert_eq!(editor.buffer_count(), 1);
}

#[test]
fn test_delete_current_buffer_switches() {
    let mut editor = Editor::new();
    
    // 创建第二个缓冲区并切换到它
    {
        let buffer = Buffer::new(BufferId::new(1));
        editor.buffers_mut().insert(BufferId::new(1), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(1), Cursor::at_origin());
    }
    editor.switch_buffer(BufferId::new(1)).unwrap();
    
    // 删除当前缓冲区
    let result = editor.delete_buffer(BufferId::new(1));
    assert!(result.is_ok());
    
    // 应该自动切换到另一个缓冲区
    assert_eq!(editor.current_buffer_id().as_usize(), 0);
}

#[test]
fn test_delete_buffer_modified() {
    let mut editor = Editor::new();
    
    // 创建第二个缓冲区并修改它
    {
        let mut buffer = Buffer::new(BufferId::new(1));
        buffer.insert(0, "modified content");
        editor.buffers_mut().insert(BufferId::new(1), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(1), Cursor::at_origin());
    }
    
    // 尝试删除修改过的缓冲区应该失败
    let result = editor.delete_buffer(BufferId::new(1));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("未保存"));
}

#[test]
fn test_delete_buffer_force() {
    let mut editor = Editor::new();
    
    // 创建第二个缓冲区并修改它
    {
        let mut buffer = Buffer::new(BufferId::new(1));
        buffer.insert(0, "modified content");
        editor.buffers_mut().insert(BufferId::new(1), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(1), Cursor::at_origin());
    }
    
    // 强制删除应该成功
    let result = editor.delete_buffer_force(BufferId::new(1));
    assert!(result.is_ok());
    assert_eq!(editor.buffer_count(), 1);
}

#[test]
fn test_delete_last_buffer_creates_new() {
    let mut editor = Editor::new();
    
    // 删除唯一的缓冲区
    let current_id = editor.current_buffer_id();
    let result = editor.delete_buffer(current_id);
    assert!(result.is_ok());
    
    // 应该创建一个新的空缓冲区
    assert_eq!(editor.buffer_count(), 1);
}

#[test]
fn test_delete_nonexistent_buffer() {
    let mut editor = Editor::new();
    
    let result = editor.delete_buffer(BufferId::new(999));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("不存在"));
}

// ==================== 光标位置保存测试 ====================

#[test]
fn test_cursor_saved_on_buffer_switch() {
    let mut editor = Editor::new();
    
    // 在第一个缓冲区设置光标位置
    *editor.cursor_mut() = Cursor::new(0, 5);
    
    // 创建第二个缓冲区
    {
        let buffer = Buffer::new(BufferId::new(1));
        editor.buffers_mut().insert(BufferId::new(1), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(1), Cursor::at_origin());
    }
    
    // 切换到第二个缓冲区
    editor.switch_buffer(BufferId::new(1)).unwrap();
    
    // 设置不同的光标位置
    *editor.cursor_mut() = Cursor::new(0, 3);
    
    // 切换回第一个缓冲区
    editor.switch_buffer(BufferId::new(0)).unwrap();
    
    // 光标应该恢复到之前的位置
    assert_eq!(editor.cursor().column, 5);
}

// ==================== 缓冲区计数测试 ====================

#[test]
fn test_buffer_count() {
    let mut editor = Editor::new();
    assert_eq!(editor.buffer_count(), 1);
    
    // 添加更多缓冲区
    for i in 1..=5 {
        let buffer = Buffer::new(BufferId::new(i));
        editor.buffers_mut().insert(BufferId::new(i), buffer);
        editor.buffer_cursors_mut().insert(BufferId::new(i), Cursor::at_origin());
    }
    
    assert_eq!(editor.buffer_count(), 6);
}
