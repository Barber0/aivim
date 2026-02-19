/// 搜索功能模块
/// 
/// 实现Vim风格的搜索功能：
/// - /pattern - 向前搜索
/// - ?pattern - 向后搜索
/// - n - 下一个匹配
/// - N - 上一个匹配

use crate::buffer::Buffer;
use crate::cursor::Cursor;

#[derive(Debug, Clone)]
pub struct SearchState {
    /// 当前搜索模式
    pub pattern: String,
    /// 搜索方向
    pub direction: SearchDirection,
    /// 所有匹配位置（字符索引）
    pub matches: Vec<usize>,
    /// 当前匹配的索引
    pub current_match: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchDirection {
    Forward,  // /
    Backward, // ?
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            direction: SearchDirection::Forward,
            matches: Vec::new(),
            current_match: None,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.pattern.is_empty()
    }

    /// 设置搜索模式并执行搜索
    pub fn set_pattern(&mut self, pattern: impl Into<String>, direction: SearchDirection, buffer: &Buffer) {
        self.pattern = pattern.into();
        self.direction = direction;
        self.find_all_matches(buffer);
    }

    /// 查找所有匹配位置
    fn find_all_matches(&mut self, buffer: &Buffer) {
        self.matches.clear();
        self.current_match = None;

        if self.pattern.is_empty() {
            return;
        }

        let text = buffer.to_string();
        let pattern = &self.pattern;

        // 简单的字符串匹配（后续可以升级为正则表达式）
        let mut start = 0;
        while let Some(pos) = text[start..].find(pattern) {
            let absolute_pos = start + pos;
            self.matches.push(absolute_pos);
            start = absolute_pos + 1;
        }
    }

    /// 计算下一个匹配的索引
    pub fn calc_next_match(&self, cursor: &Cursor, buffer: &Buffer) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }

        let current_char_idx = cursor.to_char_idx(buffer);

        // 根据搜索方向找到下一个匹配
        match self.direction {
            SearchDirection::Forward => {
                self.matches.iter()
                    .position(|&m| m > current_char_idx)
                    .or_else(|| if self.matches.len() > 1 { Some(0) } else { None })
            }
            SearchDirection::Backward => {
                self.matches.iter()
                    .rposition(|&m| m < current_char_idx)
                    .or_else(|| if self.matches.len() > 1 { Some(self.matches.len() - 1) } else { None })
            }
        }
    }

    /// 计算上一个匹配的索引
    pub fn calc_prev_match(&self, cursor: &Cursor, buffer: &Buffer) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }

        let current_char_idx = cursor.to_char_idx(buffer);

        // 与 calc_next_match 相反
        match self.direction {
            SearchDirection::Forward => {
                self.matches.iter()
                    .rposition(|&m| m < current_char_idx)
                    .or_else(|| if self.matches.len() > 1 { Some(self.matches.len() - 1) } else { None })
            }
            SearchDirection::Backward => {
                self.matches.iter()
                    .position(|&m| m > current_char_idx)
                    .or_else(|| if self.matches.len() > 1 { Some(0) } else { None })
            }
        }
    }

    /// 获取指定索引匹配的位置
    pub fn get_match_pos(&self, idx: usize) -> Option<usize> {
        self.matches.get(idx).copied()
    }

    /// 设置当前匹配索引
    pub fn set_current_match(&mut self, idx: usize) {
        self.current_match = Some(idx);
    }

    /// 获取当前高亮的匹配范围（用于UI显示）
    pub fn current_match_range(&self) -> Option<(usize, usize)> {
        self.current_match.map(|idx| {
            let start = self.matches[idx];
            let end = start + self.pattern.len();
            (start, end)
        })
    }

    /// 获取所有匹配范围（用于UI高亮）
    pub fn all_match_ranges(&self) -> Vec<(usize, usize)> {
        self.matches.iter()
            .map(|&start| (start, start + self.pattern.len()))
            .collect()
    }

    pub fn clear(&mut self) {
        self.pattern.clear();
        self.matches.clear();
        self.current_match = None;
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffer, BufferId};

    #[test]
    fn test_find_matches() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "hello world hello");

        let mut search = SearchState::new();
        search.set_pattern("hello", SearchDirection::Forward, &buffer);

        assert_eq!(search.matches.len(), 2);
        assert_eq!(search.matches[0], 0);
        assert_eq!(search.matches[1], 12);
    }

    #[test]
    fn test_next_match_forward() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "hello world hello");

        let mut search = SearchState::new();
        search.set_pattern("hello", SearchDirection::Forward, &buffer);

        let mut cursor = Cursor::at_origin();
        search.next_match(&mut cursor, &buffer);
        
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 0); // 第一个 "hello"
    }
}
