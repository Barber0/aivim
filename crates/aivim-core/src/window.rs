//! 窗口系统模块
//!
//! 实现 Vim 风格的窗口管理：
//! - 窗口分割（水平/垂直）
//! - 窗口导航
//! - 窗口大小调整

use crate::buffer::{Buffer, BufferId};
use crate::cursor::Cursor;

/// 窗口唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(usize);

impl WindowId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

/// 窗口结构体
#[derive(Debug, Clone)]
pub struct Window {
    id: WindowId,
    /// 窗口显示的缓冲区
    buffer_id: BufferId,
    /// 窗口内的光标位置
    cursor: Cursor,
    /// 滚动偏移（显示的第一行）
    scroll_offset: usize,
}

impl Window {
    pub fn new(id: WindowId, buffer_id: BufferId) -> Self {
        Self {
            id,
            buffer_id,
            cursor: Cursor::at_origin(),
            scroll_offset: 0,
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn buffer_id(&self) -> BufferId {
        self.buffer_id
    }

    pub fn set_buffer(&mut self, buffer_id: BufferId) {
        self.buffer_id = buffer_id;
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }
}

/// 分割方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Horizontal, // :sp - 水平分割（上下）
    Vertical,   // :vsp - 垂直分割（左右）
}

/// 窗口管理器
#[derive(Debug)]
pub struct WindowManager {
    /// 所有窗口
    windows: Vec<Window>,
    /// 当前活动窗口索引
    current_window_idx: usize,
    /// 下一个窗口 ID
    next_window_id: usize,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            current_window_idx: 0,
            next_window_id: 0,
        }
    }

    /// 创建初始窗口
    pub fn create_initial_window(&mut self, buffer_id: BufferId) -> WindowId {
        let window_id = WindowId::new(self.next_window_id);
        self.next_window_id += 1;

        let window = Window::new(window_id, buffer_id);
        self.windows.push(window);
        self.current_window_idx = 0;

        window_id
    }

    /// 分割当前窗口
    pub fn split_current(&mut self, direction: SplitDirection, buffer_id: BufferId) -> Option<WindowId> {
        if self.windows.is_empty() {
            return None;
        }

        let new_window_id = WindowId::new(self.next_window_id);
        self.next_window_id += 1;

        let new_window = Window::new(new_window_id, buffer_id);

        // 在当前窗口之后插入新窗口
        let insert_idx = self.current_window_idx + 1;
        self.windows.insert(insert_idx, new_window);

        // 切换到新窗口
        self.current_window_idx = insert_idx;

        Some(new_window_id)
    }

    /// 关闭当前窗口
    pub fn close_current(&mut self) -> bool {
        if self.windows.len() <= 1 {
            // 至少保留一个窗口
            return false;
        }

        self.windows.remove(self.current_window_idx);

        // 调整当前窗口索引
        if self.current_window_idx >= self.windows.len() {
            self.current_window_idx = self.windows.len() - 1;
        }

        true
    }

    /// 获取当前窗口
    pub fn current_window(&self) -> &Window {
        &self.windows[self.current_window_idx]
    }

    /// 获取当前窗口（可变）
    pub fn current_window_mut(&mut self) -> &mut Window {
        &mut self.windows[self.current_window_idx]
    }

    /// 获取当前窗口索引
    pub fn current_window_idx(&self) -> usize {
        self.current_window_idx
    }

    /// 获取窗口数量
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// 获取指定窗口
    pub fn get_window(&self, idx: usize) -> Option<&Window> {
        self.windows.get(idx)
    }

    /// 获取指定窗口（可变）
    pub fn get_window_mut(&mut self, idx: usize) -> Option<&mut Window> {
        self.windows.get_mut(idx)
    }

    /// 切换到指定索引的窗口
    pub fn switch_to_window(&mut self, idx: usize) -> bool {
        if idx < self.windows.len() {
            self.current_window_idx = idx;
            true
        } else {
            false
        }
    }

    /// 切换到下一个窗口（循环）
    pub fn next_window(&mut self) {
        self.current_window_idx = (self.current_window_idx + 1) % self.windows.len();
    }

    /// 切换到上一个窗口（循环）
    pub fn prev_window(&mut self) {
        if self.current_window_idx == 0 {
            self.current_window_idx = self.windows.len() - 1;
        } else {
            self.current_window_idx -= 1;
        }
    }

    /// 获取所有窗口
    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    /// 获取所有窗口（可变）
    pub fn windows_mut(&mut self) -> &mut [Window] {
        &mut self.windows
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let mut wm = WindowManager::new();
        let buffer_id = BufferId::new(0);

        let window_id = wm.create_initial_window(buffer_id);
        assert_eq!(wm.window_count(), 1);
        assert_eq!(wm.current_window().id(), window_id);
    }

    #[test]
    fn test_window_split() {
        let mut wm = WindowManager::new();
        let buffer_id = BufferId::new(0);

        wm.create_initial_window(buffer_id);
        let new_window_id = wm.split_current(SplitDirection::Horizontal, buffer_id);

        assert!(new_window_id.is_some());
        assert_eq!(wm.window_count(), 2);
    }

    #[test]
    fn test_window_navigation() {
        let mut wm = WindowManager::new();
        let buffer_id = BufferId::new(0);

        wm.create_initial_window(buffer_id);
        wm.split_current(SplitDirection::Horizontal, buffer_id);
        wm.split_current(SplitDirection::Horizontal, buffer_id);

        assert_eq!(wm.window_count(), 3);

        // split_current 会切换到新窗口，所以当前应该是最后一个（索引 2）
        assert_eq!(wm.current_window_idx(), 2);

        // 测试循环切换
        wm.next_window();
        assert_eq!(wm.current_window_idx(), 0); // 循环到第一个

        wm.next_window();
        assert_eq!(wm.current_window_idx(), 1);

        wm.next_window();
        assert_eq!(wm.current_window_idx(), 2); // 回到最后一个

        // 测试 prev_window
        wm.prev_window();
        assert_eq!(wm.current_window_idx(), 1);

        wm.prev_window();
        assert_eq!(wm.current_window_idx(), 0);

        wm.prev_window();
        assert_eq!(wm.current_window_idx(), 2); // 循环到最后一个
    }

    #[test]
    fn test_window_close() {
        let mut wm = WindowManager::new();
        let buffer_id = BufferId::new(0);

        wm.create_initial_window(buffer_id);
        wm.split_current(SplitDirection::Horizontal, buffer_id);

        assert_eq!(wm.window_count(), 2);

        // 关闭当前窗口
        assert!(wm.close_current());
        assert_eq!(wm.window_count(), 1);

        // 不能再关闭最后一个窗口
        assert!(!wm.close_current());
    }
}
