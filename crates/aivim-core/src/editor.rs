use crate::buffer::{Buffer, BufferId};
use crate::cursor::Cursor;
use crate::edit::{Edit, EditResult};
use crate::mode::Mode;
use crate::motion::Motion;
use crate::register::RegisterManager;
use crate::search::{SearchDirection, SearchState};
use std::collections::HashMap;
use std::io;
use std::path::Path;

pub struct Editor {
    buffers: HashMap<BufferId, Buffer>,
    current_buffer: BufferId,
    cursor: Cursor,
    mode: Mode,
    next_buffer_id: usize,
    command_line: String,
    message: Option<String>,
    register_manager: RegisterManager,
    undo_stack: Vec<EditState>,
    redo_stack: Vec<EditState>,
    search_state: SearchState,
}

#[derive(Clone)]
struct EditState {
    buffer_content: String,
    cursor: Cursor,
}

impl Editor {
    pub fn new() -> Self {
        let mut buffers = HashMap::new();
        let initial_buffer = Buffer::new(BufferId::new(0));
        let buffer_id = initial_buffer.id();
        buffers.insert(buffer_id, initial_buffer);

        Self {
            buffers,
            current_buffer: buffer_id,
            cursor: Cursor::at_origin(),
            mode: Mode::Normal,
            next_buffer_id: 1,
            command_line: String::new(),
            message: None,
            register_manager: RegisterManager::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_state: SearchState::new(),
        }
    }

    pub fn with_file(path: &Path) -> io::Result<Self> {
        let mut editor = Self::new();
        editor.open_file(path)?;
        Ok(editor)
    }

    pub fn current_buffer(&self) -> &Buffer {
        self.buffers.get(&self.current_buffer).unwrap()
    }

    pub fn current_buffer_mut(&mut self) -> &mut Buffer {
        self.buffers.get_mut(&self.current_buffer).unwrap()
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn command_line(&self) -> &str {
        &self.command_line
    }

    pub fn command_line_mut(&mut self) -> &mut String {
        &mut self.command_line
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn save_state(&mut self) {
        let buffer = self.current_buffer();
        let state = EditState {
            buffer_content: buffer.to_string(),
            cursor: self.cursor,
        };
        self.undo_stack.push(state);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            let current_state = EditState {
                buffer_content: self.current_buffer().to_string(),
                cursor: self.cursor,
            };
            self.redo_stack.push(current_state);

            let current_buffer_id = self.current_buffer;
            let buffer = self.buffers.get_mut(&current_buffer_id).unwrap();
            *buffer = Buffer::new(current_buffer_id);
            buffer.insert(0, &state.buffer_content);
            self.cursor = state.cursor;
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            let current_state = EditState {
                buffer_content: self.current_buffer().to_string(),
                cursor: self.cursor,
            };
            self.undo_stack.push(current_state);

            let current_buffer_id = self.current_buffer;
            let buffer = self.buffers.get_mut(&current_buffer_id).unwrap();
            *buffer = Buffer::new(current_buffer_id);
            buffer.insert(0, &state.buffer_content);
            self.cursor = state.cursor;
        }
    }

    pub fn execute_motion(&mut self, motion: Motion) {
        let buffer = self.buffers.get(&self.current_buffer).unwrap();
        let mut cursor = self.cursor;
        motion.execute(&mut cursor, buffer);
        self.cursor = cursor;
    }

    pub fn execute_edit(&mut self, edit: Edit) -> Option<EditResult> {
        self.save_state();
        let buffer = self.buffers.get_mut(&self.current_buffer).unwrap();
        edit.execute(&mut self.cursor, buffer)
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.mode.is_insert() {
            // Get all needed info first
            let cursor_line = self.cursor.line;
            let cursor_col = self.cursor.column;

            // 计算插入位置
            // 在 Insert 模式下，光标可以在 line_len 位置（最后一个字符之后）
            let char_idx = {
                let buffer = self.current_buffer();
                let line_start = buffer.line_to_char(cursor_line);
                let line_len = buffer.line_len(cursor_line);
                // 如果光标在 line_len 位置，在最后一个字符之后插入
                // 否则，在当前光标位置插入
                line_start + cursor_col.min(line_len)
            };

            let buffer = self.current_buffer_mut();
            buffer.insert_char(char_idx, ch);

            // Update cursor position manually - move right by 1
            // After insertion, the line length increased by 1
            let line_len = buffer.line_len(cursor_line);
            // cursor_col + 1 is the new position after insertion
            // 在 Insert 模式下，光标可以在 line_len 位置（最后一个字符之后）
            self.cursor.column = (cursor_col + 1).min(line_len);
            self.cursor.preferred_column = Some(self.cursor.column);
        }
    }

    /// 在光标后插入字符（用于 'a' 命令在行尾的情况）
    /// 在光标后进入插入模式（a命令）
    pub fn enter_append_mode(&mut self) {
        // 在 Vim 中，Normal 模式的光标在字符上，Insert 模式的光标在字符之间
        // 'a' 命令应该在当前字符之后插入

        // 获取当前光标位置信息
        let buffer = self.current_buffer();
        let line_len = buffer.line_len(self.cursor.line);
        let max_col = line_len.saturating_sub(1);
        let at_end = self.cursor.column >= max_col;

        if at_end {
            // 在行尾：将光标设置为 line_len（在最后一个字符之后）
            // 这样在 Insert 模式下，to_char_idx 会返回正确的插入位置
            self.cursor.column = line_len;
            self.set_mode(Mode::Insert);
        } else {
            // 不在行尾：向右移动一位，然后进入 Insert 模式
            drop(buffer);
            self.execute_motion(Motion::Right);
            self.set_mode(Mode::Insert);
        }
    }

    pub fn insert_newline(&mut self) {
        if self.mode.is_insert() {
            let char_idx = {
                let buffer = self.current_buffer();
                self.cursor.to_char_idx(buffer)
            };
            let buffer = self.current_buffer_mut();
            buffer.insert(char_idx, "\n");
            self.cursor.line += 1;
            self.cursor.column = 0;
        }
    }

    pub fn backspace(&mut self) {
        if self.mode.is_insert() {
            let should_edit = self.cursor.column > 0 || self.cursor.line > 0;
            if !should_edit {
                return;
            }

            self.save_state();

            let column = self.cursor.column;
            let line = self.cursor.line;

            if column > 0 {
                let char_idx = {
                    let buffer = self.current_buffer();
                    self.cursor.to_char_idx(buffer)
                };
                let buffer = self.current_buffer_mut();
                buffer.remove_char(char_idx - 1);

                // Update cursor position manually
                self.cursor.column -= 1;
                self.cursor.preferred_column = Some(self.cursor.column);
            } else if line > 0 {
                let prev_line_len = {
                    let buffer = self.current_buffer();
                    buffer.line_len(line - 1)
                };
                let current_line_start = {
                    let buffer = self.current_buffer();
                    buffer.line_to_char(line)
                };
                let buffer = self.current_buffer_mut();
                buffer.remove(current_line_start - 1, 1);
                self.cursor.line -= 1;
                self.cursor.column = prev_line_len;
            }
        }
    }

    pub fn delete_char(&mut self) {
        if self.mode == Mode::Normal {
            self.save_state();

            let char_idx = {
                let buffer = self.current_buffer();
                self.cursor.to_char_idx(buffer)
            };
            let cursor_line = self.cursor.line;
            let cursor_col = self.cursor.column;

            let buffer = self.current_buffer_mut();
            buffer.remove_char(char_idx);

            // Ensure cursor is valid manually
            let max_line = buffer.len_lines().saturating_sub(1);
            let new_line = cursor_line.min(max_line);
            let line_len = buffer.line_len(new_line);
            let max_col = line_len.saturating_sub(1);
            let new_col = cursor_col.min(max_col);

            self.cursor.line = new_line;
            self.cursor.column = new_col;
        }
    }

    pub fn open_file(&mut self, path: &Path) -> io::Result<()> {
        let buffer_id = BufferId::new(self.next_buffer_id);
        self.next_buffer_id += 1;

        // 如果文件存在则打开，否则创建新缓冲区并设置文件路径
        let buffer = if path.exists() {
            Buffer::from_file(buffer_id, path)?
        } else {
            Buffer::new_with_path(buffer_id, path)
        };

        self.buffers.insert(buffer_id, buffer);
        self.current_buffer = buffer_id;
        self.cursor = Cursor::at_origin();
        self.mode = Mode::Normal;

        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        let buffer = self.current_buffer_mut();
        buffer.save()?;
        Ok(())
    }

    pub fn save_as(&mut self, path: &Path) -> io::Result<()> {
        let buffer = self.current_buffer_mut();
        buffer.save_as(path)?;
        Ok(())
    }

    pub fn execute_command(&mut self, command: &str) -> Result<(), String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "w" | "write" => {
                if parts.len() > 1 {
                    let path = Path::new(parts[1]);
                    self.save_as(path).map_err(|e| e.to_string())?;
                } else {
                    self.save().map_err(|e| e.to_string())?;
                }
                self.set_message("Saved");
            }
            "q" | "quit" => {
                if self.current_buffer().is_modified() {
                    return Err("No write since last change (add ! to override)".to_string());
                }
            }
            "q!" => {
                // 强制退出，不保存修改
            }
            "wq" => {
                self.save().map_err(|e| e.to_string())?;
            }
            "qa" => {
            }
            "set" => {
                if parts.len() > 1 {
                    self.set_option(parts[1])?;
                }
            }
            cmd if cmd.starts_with("s/") || cmd.starts_with("%s/") => {
                // 处理替换命令
                if let Some((pattern, replacement, global, full_file)) = crate::replace::parse_substitute_command(command) {
                    let line_range = if full_file {
                        None // None 表示整个文件
                    } else {
                        let current_line = self.cursor.line;
                        Some((current_line, current_line + 1))
                    };
                    
                    let buffer = self.current_buffer_mut();
                    let result = crate::replace::replace_in_buffer(
                        buffer,
                        &pattern,
                        &replacement,
                        global,
                        line_range,
                    );
                    
                    self.set_message(&format!("Replaced {} occurrence(s)", result.count));
                } else {
                    return Err("Invalid substitute command".to_string());
                }
            }
            _ => return Err(format!("Unknown command: {}", parts[0])),
        }

        Ok(())
    }

    fn set_option(&mut self, option: &str) -> Result<(), String> {
        match option {
            "nu" | "number" => {
            }
            "nonu" | "nonumber" => {
            }
            _ => return Err(format!("Unknown option: {}", option)),
        }
        Ok(())
    }

    pub fn get_line_text(&self, line_idx: usize) -> Option<String> {
        self.current_buffer().line(line_idx).map(|l| {
            let text = l.to_string();
            text.strip_suffix('\n').map(String::from).unwrap_or(text)
        })
    }

    pub fn visible_lines(&self, viewport_height: usize, scroll_offset: usize) -> Vec<(usize, String)> {
        let buffer = self.current_buffer();
        let start_line = scroll_offset;
        let end_line = (scroll_offset + viewport_height).min(buffer.len_lines());

        (start_line..end_line)
            .filter_map(|line_idx| {
                self.get_line_text(line_idx).map(|text| (line_idx, text))
            })
            .collect()
    }

    // ==================== 寄存器操作 ====================

    pub fn register_manager(&self) -> &RegisterManager {
        &self.register_manager
    }

    pub fn register_manager_mut(&mut self) -> &mut RegisterManager {
        &mut self.register_manager
    }

    /// 复制当前行到寄存器 (yy)
    pub fn yank_line(&mut self, register: Option<char>) {
        let line_idx = self.cursor.line;
        if let Some(line_text) = self.get_line_text(line_idx) {
            let content = format!("{}\n", line_text);
            let reg = register.unwrap_or('"');
            self.register_manager.set(reg, content, true);
        }
    }

    /// 复制选中区域到寄存器 (y)
    pub fn yank(&mut self, start: usize, end: usize, register: Option<char>, linewise: bool) {
        let buffer = self.current_buffer();
        let content = buffer.slice(start..end).to_string();
        let reg = register.unwrap_or('"');
        self.register_manager.set(reg, content, linewise);
    }

    /// 粘贴寄存器内容 (p/P)
    pub fn paste(&mut self, register: Option<char>, before_cursor: bool) {
        let reg = register.unwrap_or('"');

        // 先获取寄存器内容，避免借用冲突
        let (content, linewise) = {
            if let Some(register) = self.register_manager.get(reg) {
                if register.is_empty() {
                    return;
                }
                (register.content.clone(), register.linewise)
            } else {
                return;
            }
        };

        self.save_state();

        if linewise {
            // 整行粘贴
            let insert_line = if before_cursor {
                self.cursor.line
            } else {
                self.cursor.line + 1
            };

            let char_idx = self.current_buffer().line_to_char(insert_line);
            let buffer = self.current_buffer_mut();
            buffer.insert(char_idx, &content);

            // 移动光标到新粘贴的第一行
            self.cursor.line = insert_line;
            self.cursor.column = 0;
        } else {
            // 字符粘贴
            let cursor_line = self.cursor.line;
            let cursor_col = self.cursor.column;

            let char_idx = {
                let buffer = self.current_buffer();
                let idx = self.cursor.to_char_idx(buffer);
                if before_cursor { idx } else { idx + 1 }
            };

            let buffer = self.current_buffer_mut();
            buffer.insert(char_idx, &content);

            // 移动光标到粘贴内容之后
            let content_len = content.chars().count();
            let line_len = buffer.line_len(cursor_line);
            self.cursor.column = (cursor_col + content_len).min(line_len.saturating_sub(1));
        }
    }

    /// 删除当前行并放入寄存器 (dd)
    pub fn delete_line(&mut self, register: Option<char>) -> Option<String> {
        let line_idx = self.cursor.line;
        let line_text = self.get_line_text(line_idx)?;
        let content = format!("{}\n", line_text);

        self.save_state();

        // 保存到寄存器
        let reg = register.unwrap_or('"');
        self.register_manager.set(reg, &content, true);

        // 删除行
        let line_start = self.current_buffer().line_to_char(line_idx);
        let line_len = self.current_buffer().line_len(line_idx);
        let buffer = self.current_buffer_mut();
        buffer.remove(line_start, line_len);

        // 确保至少有一行
        if buffer.len_lines() == 0 {
            buffer.insert(0, "\n");
        }

        // 调整光标位置
        let max_line = buffer.len_lines().saturating_sub(1);
        self.cursor.line = self.cursor.line.min(max_line);
        self.cursor.column = 0;

        Some(content)
    }

    /// 删除字符并放入寄存器 (x/d)
    pub fn delete_char_to_register(&mut self, register: Option<char>) -> Option<char> {
        self.save_state();

        // 先获取光标位置信息
        let char_idx = {
            let buffer = self.current_buffer();
            self.cursor.to_char_idx(buffer)
        };
        let cursor_line = self.cursor.line;
        let cursor_col = self.cursor.column;

        let buffer = self.current_buffer_mut();
        let ch = buffer.remove_char(char_idx)?;

        // 确保光标有效
        let max_line = buffer.len_lines().saturating_sub(1);
        let new_line = cursor_line.min(max_line);
        let line_len = buffer.line_len(new_line);
        let max_col = line_len.saturating_sub(1);
        let new_col = cursor_col.min(max_col);

        self.cursor.line = new_line;
        self.cursor.column = new_col;

        // 最后设置寄存器
        let reg = register.unwrap_or('"');
        self.register_manager.set(reg, ch.to_string(), false);

        Some(ch)
    }

    // ==================== 搜索功能 ====================

    pub fn search_state(&self) -> &SearchState {
        &self.search_state
    }

    /// 开始搜索（/ 或 ?）
    pub fn start_search(&mut self, direction: SearchDirection, pattern: impl Into<String>) {
        let pattern = pattern.into();
        if !pattern.is_empty() {
            // 先设置搜索模式
            let buffer_clone = {
                let buffer = self.current_buffer();
                buffer.clone()
            };
            self.search_state.set_pattern(&pattern, direction, &buffer_clone);
            
            // 保存到搜索寄存器
            self.register_manager.set_search(&pattern);
            
            // 跳转到第一个匹配
            let (idx, pos) = {
                let buffer = self.current_buffer();
                let idx = self.search_state.calc_next_match(&self.cursor, buffer);
                let pos = idx.and_then(|i| self.search_state.get_match_pos(i));
                (idx, pos)
            };
            if let (Some(i), Some(p)) = (idx, pos) {
                self.search_state.set_current_match(i);
                let buffer = self.current_buffer();
                self.cursor = Cursor::from_char_idx(buffer, p);
            }
        }
    }

    /// 搜索下一个（n）
    pub fn search_next(&mut self) {
        let (idx, pos) = {
            let buffer = self.current_buffer();
            let idx = self.search_state.calc_next_match(&self.cursor, buffer);
            let pos = idx.and_then(|i| self.search_state.get_match_pos(i));
            (idx, pos)
        };
        if let (Some(i), Some(p)) = (idx, pos) {
            self.search_state.set_current_match(i);
            let buffer = self.current_buffer();
            self.cursor = Cursor::from_char_idx(buffer, p);
        }
    }

    /// 搜索上一个（N）
    pub fn search_prev(&mut self) {
        let (idx, pos) = {
            let buffer = self.current_buffer();
            let idx = self.search_state.calc_prev_match(&self.cursor, buffer);
            let pos = idx.and_then(|i| self.search_state.get_match_pos(i));
            (idx, pos)
        };
        if let (Some(i), Some(p)) = (idx, pos) {
            self.search_state.set_current_match(i);
            let buffer = self.current_buffer();
            self.cursor = Cursor::from_char_idx(buffer, p);
        }
    }

    /// 清除搜索高亮
    pub fn clear_search(&mut self) {
        self.search_state.clear();
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
