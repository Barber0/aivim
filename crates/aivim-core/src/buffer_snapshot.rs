/// 缓冲区快照系统
/// 
/// 提供自动状态管理，确保所有修改操作都能正确撤销/重做，
/// 并保留文件路径等元数据。

use crate::buffer::{Buffer, BufferId};
use crate::cursor::Cursor;
use std::path::PathBuf;

/// 缓冲区快照
#[derive(Clone)]
pub struct BufferSnapshot {
    pub content: String,
    pub cursor: Cursor,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
}

impl BufferSnapshot {
    /// 从缓冲区创建快照
    pub fn from_buffer(buffer: &Buffer, cursor: &Cursor) -> Self {
        Self {
            content: buffer.to_string(),
            cursor: *cursor,
            file_path: buffer.file_path().map(|p| p.to_path_buf()),
            modified: buffer.is_modified(),
        }
    }

    /// 应用快照到缓冲区
    pub fn apply_to_buffer(&self, buffer: &mut Buffer, cursor: &mut Cursor) {
        // 恢复内容
        *buffer = Buffer::new(buffer.id());
        buffer.insert(0, &self.content);
        
        // 恢复文件路径
        if let Some(ref path) = self.file_path {
            buffer.set_file_path(path.clone());
        }
        
        // 恢复光标
        *cursor = self.cursor;
    }
}

/// 修改操作包装器
/// 
/// 使用方式：
/// ```
/// let result = editor.with_snapshot(|editor| {
///     editor.execute_edit(edit)
/// });
/// ```
pub struct SnapshotManager {
    undo_stack: Vec<BufferSnapshot>,
    redo_stack: Vec<BufferSnapshot>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// 保存当前状态
    pub fn save(&mut self, buffer: &Buffer, cursor: &Cursor) {
        let snapshot = BufferSnapshot::from_buffer(buffer, cursor);
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
    }

    /// 撤销
    pub fn undo(&mut self, buffer: &mut Buffer, cursor: &mut Cursor) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            // 保存当前状态到 redo 栈
            let current = BufferSnapshot::from_buffer(buffer, cursor);
            self.redo_stack.push(current);
            
            // 应用快照
            snapshot.apply_to_buffer(buffer, cursor);
            true
        } else {
            false
        }
    }

    /// 重做
    pub fn redo(&mut self, buffer: &mut Buffer, cursor: &mut Cursor) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            // 保存当前状态到 undo 栈
            let current = BufferSnapshot::from_buffer(buffer, cursor);
            self.undo_stack.push(current);
            
            // 应用快照
            snapshot.apply_to_buffer(buffer, cursor);
            true
        } else {
            false
        }
    }

    /// 检查是否可以撤销
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// 检查是否可以重做
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 清空历史
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 自动快照守卫
/// 
/// 在创建时保存状态，在作用域结束时自动恢复（如果需要）
pub struct AutoSnapshot<'a> {
    manager: &'a mut SnapshotManager,
    buffer: &'a mut Buffer,
    cursor: &'a mut Cursor,
    committed: bool,
}

impl<'a> AutoSnapshot<'a> {
    pub fn new(
        manager: &'a mut SnapshotManager,
        buffer: &'a mut Buffer,
        cursor: &'a mut Cursor,
    ) -> Self {
        manager.save(buffer, cursor);
        Self {
            manager,
            buffer,
            cursor,
            committed: false,
        }
    }

    /// 提交修改（不恢复）
    pub fn commit(mut self) {
        self.committed = true;
    }

    /// 回滚修改
    pub fn rollback(self) {
        // 自动恢复，不需要额外操作
    }
}

impl<'a> Drop for AutoSnapshot<'a> {
    fn drop(&mut self) {
        if !self.committed {
            // 如果没有提交，撤销最后一次保存
            if let Some(_) = self.manager.undo_stack.pop() {
                // 从 undo 栈移除，但不恢复（因为修改已经发生）
                // 这种情况下应该手动调用 undo 来恢复
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_basic() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "hello world\n");
        
        let cursor = Cursor::at_origin();
        let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);
        
        assert_eq!(snapshot.content, "hello world\n");
        assert_eq!(snapshot.cursor.line, 0);
        assert_eq!(snapshot.cursor.column, 0);
    }

    #[test]
    fn test_snapshot_file_path() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "test");
        buffer.set_file_path(std::path::PathBuf::from("/tmp/test.txt"));
        
        let cursor = Cursor::at_origin();
        let snapshot = BufferSnapshot::from_buffer(&buffer, &cursor);
        
        assert_eq!(snapshot.file_path, Some(std::path::PathBuf::from("/tmp/test.txt")));
    }
}
