//! Buffer Snapshot 模块单元测试
//!
//! 对应源文件: src/buffer_snapshot.rs
//! 测试范围: 快照创建、应用、撤销/重做管理

use aivim_core::buffer::{Buffer, BufferId};
use aivim_core::buffer_snapshot::{AutoSnapshot, BufferSnapshot, SnapshotManager};
use aivim_core::cursor::Cursor;
use std::path::PathBuf;

// ==================== BufferSnapshot 测试 ====================

#[test]
fn test_snapshot_basic() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world\n");

    let cursor = Cursor::at_origin();
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    assert_eq!(snapshot.content, "hello world\n");
    assert_eq!(snapshot.cursor.line, 0);
    assert_eq!(snapshot.cursor.column, 0);
    assert!(snapshot.file_path.is_none());
    assert!(snapshot.modified);
}

#[test]
fn test_snapshot_with_cursor() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello world\n");

    let cursor = Cursor::new(0, 5);
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    assert_eq!(snapshot.cursor.line, 0);
    assert_eq!(snapshot.cursor.column, 5);
}

#[test]
fn test_snapshot_file_path() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "test");
    buffer.set_file_path(PathBuf::from("/tmp/test.txt"));

    let cursor = Cursor::at_origin();
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    assert_eq!(
        snapshot.file_path,
        Some(PathBuf::from("/tmp/test.txt"))
    );
}

#[test]
fn test_snapshot_modified_flag() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "test");

    let cursor = Cursor::at_origin();
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    assert!(snapshot.modified);
}

#[test]
fn test_snapshot_empty_buffer() {
    let buffer = Buffer::new(BufferId::new(0));
    let cursor = Cursor::at_origin();
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    assert_eq!(snapshot.content, "");
    assert!(!snapshot.modified);
}

// ==================== apply_to_buffer 测试 ====================

#[test]
fn test_apply_to_buffer() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "original");

    let cursor = Cursor::at_origin();
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    // 修改缓冲区
    buffer.remove(0, 8);
    buffer.insert(0, "modified");

    // 应用快照恢复
    let mut cursor = Cursor::new(0, 5);
    snapshot.apply_to_buffer(&mut buffer, &mut cursor);

    assert_eq!(buffer.to_string(), "original");
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 0);
}

#[test]
fn test_apply_to_buffer_with_path() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "content");
    buffer.set_file_path(PathBuf::from("/tmp/original.txt"));

    let cursor = Cursor::at_origin();
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    // 修改缓冲区
    buffer.set_file_path(PathBuf::from("/tmp/modified.txt"));

    // 应用快照恢复
    let mut cursor = Cursor::at_origin();
    snapshot.apply_to_buffer(&mut buffer, &mut cursor);

    assert_eq!(
        buffer.file_path(),
        Some(PathBuf::from("/tmp/original.txt").as_path())
    );
}

// ==================== SnapshotManager 基本测试 ====================

#[test]
fn test_manager_new() {
    let manager = SnapshotManager::new();
    assert!(!manager.can_undo());
    assert!(!manager.can_redo());
}

#[test]
fn test_manager_default() {
    let manager: SnapshotManager = Default::default();
    assert!(!manager.can_undo());
    assert!(!manager.can_redo());
}

#[test]
fn test_manager_save() {
    let mut manager = SnapshotManager::new();
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "hello");
    let cursor = Cursor::at_origin();

    manager.save(&buffer, &cursor);

    assert!(manager.can_undo());
    assert!(!manager.can_redo());
}

#[test]
fn test_manager_undo_basic() {
    let mut manager = SnapshotManager::new();
    let mut buffer = Buffer::new(BufferId::new(0));
    let cursor = Cursor::at_origin();

    // 保存空状态
    manager.save(&buffer, &cursor);
    
    // 修改并保存
    buffer.insert(0, "test");
    manager.save(&buffer, &cursor);

    // 撤销
    let mut cursor = Cursor::at_origin();
    let result = manager.undo(&mut buffer, &mut cursor);

    assert!(result);
    assert!(manager.can_redo());
}

#[test]
fn test_manager_undo_empty() {
    let mut manager = SnapshotManager::new();
    let mut buffer = Buffer::new(BufferId::new(0));
    let mut cursor = Cursor::at_origin();

    let result = manager.undo(&mut buffer, &mut cursor);

    assert!(!result);
}

#[test]
fn test_manager_redo_empty() {
    let mut manager = SnapshotManager::new();
    let mut buffer = Buffer::new(BufferId::new(0));
    let mut cursor = Cursor::at_origin();

    let result = manager.redo(&mut buffer, &mut cursor);

    assert!(!result);
}

#[test]
fn test_manager_clear() {
    let mut manager = SnapshotManager::new();
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "test");
    let cursor = Cursor::at_origin();

    manager.save(&buffer, &cursor);
    manager.save(&buffer, &cursor);

    assert!(manager.can_undo());

    manager.clear();

    assert!(!manager.can_undo());
    assert!(!manager.can_redo());
}

// ==================== AutoSnapshot 测试 ====================

#[test]
fn test_auto_snapshot_commit() {
    let mut manager = SnapshotManager::new();
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "test");
    let mut cursor = Cursor::at_origin();

    {
        let auto = AutoSnapshot::new(&mut manager, &mut buffer, &mut cursor);
        auto.commit();
    }

    // 提交了，所以应该可以撤销
    assert!(manager.can_undo());
}

#[test]
fn test_auto_snapshot_rollback() {
    let mut manager = SnapshotManager::new();
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "test");
    let mut cursor = Cursor::at_origin();

    {
        let _auto = AutoSnapshot::new(&mut manager, &mut buffer, &mut cursor);
        // 不调用 commit，会自动回滚（从 undo 栈中移除）
    }

    // Drop 实现会从 undo 栈中弹出快照
    // 所以此时 undo 栈应该是空的
    assert!(!manager.can_undo());
}

// ==================== 边界情况测试 ====================

#[test]
fn test_snapshot_multiline_content() {
    let mut buffer = Buffer::new(BufferId::new(0));
    buffer.insert(0, "line1\nline2\nline3");

    let cursor = Cursor::new(1, 2);
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    assert_eq!(snapshot.content, "line1\nline2\nline3");
    assert_eq!(snapshot.cursor.line, 1);
    assert_eq!(snapshot.cursor.column, 2);
}

#[test]
fn test_apply_to_buffer_preserves_id() {
    let mut buffer = Buffer::new(BufferId::new(42));
    buffer.insert(0, "content");

    let cursor = Cursor::at_origin();
    let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);

    // 应用快照
    let mut cursor = Cursor::at_origin();
    snapshot.apply_to_buffer(&mut buffer, &mut cursor);

    // 缓冲区 ID 应该保持不变
    assert_eq!(buffer.id().as_usize(), 42);
}

#[test]
fn test_manager_save_clears_redo() {
    let mut manager = SnapshotManager::new();
    let mut buffer = Buffer::new(BufferId::new(0));
    let cursor = Cursor::at_origin();

    // 保存初始状态
    manager.save(&buffer, &cursor);
    
    // 修改并保存
    buffer.insert(0, "modified");
    manager.save(&buffer, &cursor);

    // 撤销
    let mut cursor = Cursor::at_origin();
    manager.undo(&mut buffer, &mut cursor);
    assert!(manager.can_redo());

    // 新操作会清除 redo 栈
    buffer.insert(0, "new");
    manager.save(&buffer, &cursor);

    assert!(!manager.can_redo());
}
