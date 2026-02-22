use crate::buffer::{Buffer, BufferId};
use crate::cursor::Cursor;
use crate::edit::{Edit, EditResult};
use crate::mode::Mode;
use crate::motion::Motion;
use crate::register::RegisterManager;
use crate::search::{SearchDirection, SearchState};
use crate::text_object::TextObject;
use crate::with_save_state;
use std::collections::HashMap;
use std::io;
use std::path::Path;

/// 编辑器配置选项
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorOptions {
    /// 显示绝对行号
    pub number: bool,
    /// 显示相对行号
    pub relativenumber: bool,
    /// 高亮当前行
    pub cursorline: bool,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            number: false,
            relativenumber: false,
            cursorline: false,
        }
    }
}

pub struct Editor {
    buffers: HashMap<BufferId, Buffer>,
    current_buffer: BufferId,
    cursor: Cursor,
    // 为每个缓冲区存储独立的光标位置
    buffer_cursors: HashMap<BufferId, Cursor>,
    mode: Mode,
    next_buffer_id: usize,
    command_line: String,
    message: Option<String>,
    register_manager: RegisterManager,
    undo_stack: Vec<EditState>,
    redo_stack: Vec<EditState>,
    search_state: SearchState,
    // UI 状态
    show_buffer_list: bool,
    show_registers_panel: bool,
    // 编辑器配置
    options: EditorOptions,
}

#[derive(Clone)]
struct EditState {
    buffer_content: String,
    cursor: Cursor,
    file_path: Option<std::path::PathBuf>,
}

impl Editor {
    pub fn new() -> Self {
        let mut buffers = HashMap::new();
        let mut buffer_cursors = HashMap::new();
        let initial_buffer = Buffer::new(BufferId::new(0));
        let buffer_id = initial_buffer.id();
        buffers.insert(buffer_id, initial_buffer);
        buffer_cursors.insert(buffer_id, Cursor::at_origin());

        Self {
            buffers,
            current_buffer: buffer_id,
            cursor: Cursor::at_origin(),
            buffer_cursors,
            mode: Mode::Normal,
            next_buffer_id: 1,
            command_line: String::new(),
            message: None,
            register_manager: RegisterManager::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_state: SearchState::new(),
            show_buffer_list: false,
            show_registers_panel: false,
            options: EditorOptions::default(),
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

    pub fn buffers_mut(&mut self) -> &mut HashMap<BufferId, Buffer> {
        &mut self.buffers
    }

    pub fn buffer_cursors_mut(&mut self) -> &mut HashMap<BufferId, Cursor> {
        &mut self.buffer_cursors
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
        // 如果从 Normal 模式进入 Insert 模式，保存状态用于撤销
        // 这样整个 Insert 会话可以作为一个单元撤销
        if self.mode == Mode::Normal && mode == Mode::Insert {
            self.save_state();
        }
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

    pub fn options(&self) -> &EditorOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut EditorOptions {
        &mut self.options
    }

    pub fn show_buffer_list(&self) -> bool {
        self.show_buffer_list
    }

    pub fn set_show_buffer_list(&mut self, show: bool) {
        self.show_buffer_list = show;
    }

    pub fn show_registers_panel(&self) -> bool {
        self.show_registers_panel
    }

    pub fn set_show_registers_panel(&mut self, show: bool) {
        self.show_registers_panel = show;
    }

    pub fn save_state(&mut self) {
        let buffer = self.current_buffer();
        let state = EditState {
            buffer_content: buffer.to_string(),
            cursor: self.cursor,
            file_path: buffer.file_path().map(|p| p.to_path_buf()),
        };
        self.undo_stack.push(state);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            let current_buffer = self.current_buffer();
            let current_state = EditState {
                buffer_content: current_buffer.to_string(),
                cursor: self.cursor,
                file_path: current_buffer.file_path().map(|p| p.to_path_buf()),
            };
            self.redo_stack.push(current_state);

            let current_buffer_id = self.current_buffer;
            let buffer = self.buffers.get_mut(&current_buffer_id).unwrap();
            *buffer = Buffer::new(current_buffer_id);
            buffer.insert(0, &state.buffer_content);
            // 恢复文件路径
            if let Some(path) = state.file_path {
                buffer.set_file_path(path);
            }
            self.cursor = state.cursor;
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            let current_buffer = self.current_buffer();
            let current_state = EditState {
                buffer_content: current_buffer.to_string(),
                cursor: self.cursor,
                file_path: current_buffer.file_path().map(|p| p.to_path_buf()),
            };
            self.undo_stack.push(current_state);

            let current_buffer_id = self.current_buffer;
            let buffer = self.buffers.get_mut(&current_buffer_id).unwrap();
            *buffer = Buffer::new(current_buffer_id);
            buffer.insert(0, &state.buffer_content);
            // 恢复文件路径
            if let Some(path) = state.file_path {
                buffer.set_file_path(path);
            }
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
        with_save_state!(self, {
            let buffer = self.buffers.get_mut(&self.current_buffer).unwrap();
            edit.execute(&mut self.cursor, buffer)
        })
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
        // text_len 是实际文本长度（不包括换行符）
        let text_len = line_len.saturating_sub(1);
        let max_col = text_len.saturating_sub(1);
        let at_end = self.cursor.column >= max_col;

        if at_end {
            // 在行尾：将光标设置为 text_len（在最后一个字符之后，但在换行符之前）
            // 这样在 Insert 模式下，to_char_idx 会返回正确的插入位置
            self.cursor.column = text_len;
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

            with_save_state!(self, {
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
            });
        }
    }

    pub fn delete_char(&mut self) {
        if self.mode == Mode::Normal {
            with_save_state!(self, {
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
            });
        }
    }

    pub fn open_file(&mut self, path: &Path) -> io::Result<()> {
        // 保存当前缓冲区的光标位置
        self.buffer_cursors.insert(self.current_buffer, self.cursor.clone());

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
        
        // 恢复该缓冲区的光标位置，如果没有则使用默认位置
        self.cursor = self.buffer_cursors.get(&buffer_id).cloned().unwrap_or_else(Cursor::at_origin);
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
            "reg" | "registers" => {
                // 显示寄存器列表面板而不是消息
                self.show_registers_panel = true;
            }
            "ls" | "buffers" => {
                // 显示缓冲区列表面板而不是消息
                self.set_show_buffer_list(true);
            }
            "b" | "buffer" => {
                if parts.len() > 1 {
                    if let Ok(id) = parts[1].parse::<usize>() {
                        let buffer_id = BufferId::new(id);
                        self.switch_buffer(buffer_id)?;
                        self.set_message(&format!("Switched to buffer {}", id));
                        // 切换缓冲区后关闭缓冲区列表面板
                        self.show_buffer_list = false;
                    } else {
                        return Err("Invalid buffer ID".to_string());
                    }
                } else {
                    return Err("Buffer ID required".to_string());
                }
            }
            "bn" | "bnext" | "next" => {
                match self.next_buffer() {
                    Ok(_) => {
                        let id = self.current_buffer_id().as_usize();
                        self.set_message(&format!("Switched to buffer {}", id));
                        // 切换缓冲区后关闭缓冲区列表面板
                        self.show_buffer_list = false;
                    }
                    Err(e) => self.set_message(&e),
                }
            }
            "bp" | "bprev" | "bprevious" | "prev" => {
                match self.prev_buffer() {
                    Ok(_) => {
                        let id = self.current_buffer_id().as_usize();
                        self.set_message(&format!("Switched to buffer {}", id));
                        // 切换缓冲区后关闭缓冲区列表面板
                        self.show_buffer_list = false;
                    }
                    Err(e) => self.set_message(&e),
                }
            }
            "bd" | "bdelete" => {
                let buffer_id = if parts.len() > 1 {
                    if let Ok(id) = parts[1].parse::<usize>() {
                        BufferId::new(id)
                    } else {
                        return Err("Invalid buffer ID".to_string());
                    }
                } else {
                    self.current_buffer_id()
                };
                
                match self.delete_buffer(buffer_id) {
                    Ok(_) => {
                        self.set_message(&format!("Deleted buffer {}", buffer_id.as_usize()));
                    }
                    Err(e) => return Err(e),
                }
            }
            "bd!" | "bdelete!" => {
                let buffer_id = if parts.len() > 1 {
                    if let Ok(id) = parts[1].parse::<usize>() {
                        BufferId::new(id)
                    } else {
                        return Err("Invalid buffer ID".to_string());
                    }
                } else {
                    self.current_buffer_id()
                };
                
                match self.delete_buffer_force(buffer_id) {
                    Ok(_) => {
                        self.set_message(&format!("Deleted buffer {}", buffer_id.as_usize()));
                    }
                    Err(e) => return Err(e),
                }
            }
            "new" => {
                // 创建新的空缓冲区
                self.create_new_buffer();
                let id = self.current_buffer_id().as_usize();
                self.set_message(&format!("Created new buffer {}", id));
            }
            "e" | "edit" => {
                if parts.len() > 1 {
                    let path = Path::new(parts[1]);
                    match self.open_file(path) {
                        Ok(_) => {
                            self.set_message(&format!("Opened {}", parts[1]));
                        }
                        Err(e) => {
                            return Err(format!("Failed to open {}: {}", parts[1], e));
                        }
                    }
                } else {
                    return Err("Filename required".to_string());
                }
            }
            cmd if cmd.starts_with("s/") || cmd.starts_with("%s/") => {
                // 处理替换命令
                if let Some((pattern, replacement, global, full_file)) = crate::replace::parse_substitute_command(command) {
                    with_save_state!(self, {
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
                    });
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
                self.options.number = true;
                self.set_message("Enabled line numbers");
            }
            "nonu" | "nonumber" => {
                self.options.number = false;
                self.set_message("Disabled line numbers");
            }
            "rnu" | "relativenumber" => {
                self.options.relativenumber = true;
                self.set_message("Enabled relative line numbers");
            }
            "nornu" | "norelativenumber" => {
                self.options.relativenumber = false;
                self.set_message("Disabled relative line numbers");
            }
            "cursorline" => {
                self.options.cursorline = true;
                self.set_message("Enabled cursor line highlighting");
            }
            "nocursorline" => {
                self.options.cursorline = false;
                self.set_message("Disabled cursor line highlighting");
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

        with_save_state!(self, {
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

                let insert_idx = {
                    let buffer = self.current_buffer();
                    let line_start = buffer.line_to_char(cursor_line);
                    let line_len = buffer.line_len(cursor_line);
                    // line_len 包括换行符，实际文本长度是 line_len - 1
                    let text_len = line_len.saturating_sub(1);
                    
                    if before_cursor {
                        // P - 在光标位置之前粘贴
                        // cursor_col 应该限制在文本范围内
                        line_start + cursor_col.min(text_len)
                    } else {
                        // p - 在光标位置之后粘贴
                        // 如果 cursor_col >= text_len，说明在行尾，在文本末尾粘贴
                        // 否则在 cursor_col + 1 位置粘贴
                        if cursor_col >= text_len {
                            // 在行尾，在文本末尾（换行符前）粘贴
                            line_start + text_len
                        } else {
                            // 在行中间，在当前字符后粘贴
                            line_start + cursor_col + 1
                        }
                    }
                };

                let buffer = self.current_buffer_mut();
                buffer.insert(insert_idx, &content);

                // 移动光标到粘贴内容之后
                let content_len = content.chars().count();
                let new_line_len = buffer.line_len(cursor_line);
                self.cursor.column = (cursor_col + content_len).min(new_line_len.saturating_sub(1));
            }
        });
    }

    /// 删除当前行并放入寄存器 (dd)
    pub fn delete_line(&mut self, register: Option<char>) -> Option<String> {
        let line_idx = self.cursor.line;
        let line_text = self.get_line_text(line_idx)?;
        let content = format!("{}\n", line_text);

        with_save_state!(self, {
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
        })
    }

    /// 删除字符并放入寄存器 (x/d)
    pub fn delete_char_to_register(&mut self, register: Option<char>) -> Option<char> {
        with_save_state!(self, {
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
        })
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
            
            // 跳转到第一个匹配（从光标位置开始）
            let (idx, pos) = {
                let buffer = self.current_buffer();
                // 使用 calc_first_match 找当前位置或之后的第一个匹配
                let idx = self.search_state.calc_first_match(&self.cursor, buffer);
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

    // ==================== 范围操作 ====================

    /// 删除从当前位置到目标位置的文本
    pub fn delete_to_motion(&mut self, motion: Motion) -> Option<String> {
        self.delete_to_motion_with_register(motion, None)
    }

    /// 删除从当前位置到目标位置的文本（支持指定寄存器）
    pub fn delete_to_motion_with_register(&mut self, motion: Motion, register: Option<char>) -> Option<String> {
        with_save_state!(self, {
            let start_cursor = self.cursor;
            let start_idx = {
                let buffer = self.current_buffer();
                self.cursor.to_char_idx(buffer)
            };

            // 临时执行移动来计算终点（不改变实际光标）
            let mut temp_cursor = self.cursor;
            motion.execute(&mut temp_cursor, self.current_buffer());

            let mut end_idx = {
                let buffer = self.current_buffer();
                temp_cursor.to_char_idx(buffer)
            };

            if start_idx == end_idx {
                return None;
            }

            // 对于单词移动，检查是否跨越了换行符
            // 如果是，截断到当前行的末尾，不删除换行符
            if matches!(motion, Motion::WordForward | Motion::WordBackward) {
                let buffer = self.current_buffer();
                let text = buffer.rope().to_string();
                
                // 确保 start_idx < end_idx
                let (start, end) = if start_idx < end_idx {
                    (start_idx, end_idx)
                } else {
                    (end_idx, start_idx)
                };
                
                // 检查删除范围内是否有换行符
                if let Some(newline_pos) = text[start..end].find('\n') {
                    // 有换行符，截断到换行符之前
                    end_idx = start + newline_pos;
                }
            }

            if start_idx == end_idx {
                return None;
            }

            let (start, end, is_forward) = if start_idx < end_idx {
                (start_idx, end_idx, true)
            } else {
                (end_idx, start_idx, false)
            };

            let deleted = {
                let buffer = self.current_buffer();
                let text = buffer.rope().to_string();
                text[start..end].to_string()
            };

            // 删除文本
            {
                let buffer = self.current_buffer_mut();
                buffer.remove(start, end - start);
            }

            // 如果指定了寄存器，存入命名寄存器
            if let Some(reg) = register {
                self.register_manager.set(reg, &deleted, false);
            }

            // 同时存入无名寄存器（删除操作）
            self.register_manager.set_unnamed_delete(&deleted, false);

            // 设置光标位置：
            // - 向前删除（dw, dl）：保持在原位置
            // - 向后删除（db, dh）：移动到删除区域的开头
            if is_forward {
                self.cursor = start_cursor;
            } else {
                // 向后删除，光标已经在正确的位置（start）
                self.cursor = Cursor::from_char_idx(self.current_buffer(), start);
            }

            Some(deleted)
        })
    }

    /// 复制从当前位置到目标位置的文本
    pub fn yank_to_motion(&mut self, motion: Motion) {
        self.yank_to_motion_with_register(motion, None);
    }

    /// 复制从当前位置到目标位置的文本（支持指定寄存器）
    pub fn yank_to_motion_with_register(&mut self, motion: Motion, register: Option<char>) {
        let start_idx = {
            let buffer = self.current_buffer();
            self.cursor.to_char_idx(buffer)
        };

        // 临时执行移动来计算终点
        let mut temp_cursor = self.cursor;
        motion.execute(&mut temp_cursor, self.current_buffer());

        let end_idx = {
            let buffer = self.current_buffer();
            temp_cursor.to_char_idx(buffer)
        };

        if start_idx == end_idx {
            return;
        }

        let (start, end) = if start_idx < end_idx {
            (start_idx, end_idx)
        } else {
            (end_idx, start_idx)
        };

        let yanked = {
            let buffer = self.current_buffer();
            let text = buffer.rope().to_string();
            text[start..end].to_string()
        };

        // 如果指定了寄存器，存入命名寄存器
        if let Some(reg) = register {
            self.register_manager.set(reg, &yanked, false);
        }

        // 同时存入无名寄存器（复制操作）
        self.register_manager.set_unnamed_yank(&yanked, false);
    }

    // ==================== 文本对象操作 ====================

    /// 删除文本对象（如 daw, diw）
    pub fn delete_text_object(&mut self, obj: TextObject) -> Option<String> {
        with_save_state!(self, {
            let buffer = self.current_buffer();
            let (start, end) = obj.get_range(&self.cursor, buffer)?;

            let deleted = {
                let buffer = self.current_buffer();
                let text = buffer.rope().to_string();
                text[start..end].to_string()
            };

            // 删除文本
            let buffer = self.current_buffer_mut();
            buffer.remove(start, end - start);

            // 将删除的内容放入无名寄存器（删除操作）
            let is_linewise = false;
            self.register_manager.set_unnamed_delete(&deleted, is_linewise);

            // 更新光标位置
            self.cursor = Cursor::from_char_idx(self.current_buffer(), start);

            Some(deleted)
        })
    }

    /// 复制文本对象（如 yaw, yiw）
    pub fn yank_text_object(&mut self, obj: TextObject) -> Option<String> {
        let buffer = self.current_buffer();
        let (start, end) = obj.get_range(&self.cursor, buffer)?;

        let yanked = {
            let buffer = self.current_buffer();
            let text = buffer.rope().to_string();
            text[start..end].to_string()
        };

        // 将复制的内容放入无名寄存器（复制操作）
        let is_linewise = false;
        self.register_manager.set_unnamed_yank(&yanked, is_linewise);

        Some(yanked)
    }

    // ==================== 寄存器显示 ====================

    /// 格式化所有寄存器内容用于显示
    ///
    /// 返回格式化的字符串，每行一个寄存器：
    /// "a   第一行内容
    ///      第二行内容"
    pub fn format_registers(&self) -> String {
        let registers = self.register_manager.get_all_registers();

        if registers.is_empty() {
            return "No registers".to_string();
        }

        let mut output = String::new();
        output.push_str("Registers:\n");
        output.push_str("----------\n");

        for reg in registers {
            let name = reg.name;
            let content = &reg.content;
            let linewise = if reg.linewise { " (linewise)" } else { "" };

            // 截断过长的内容
            let max_len = 80;
            let display_content = if content.len() > max_len {
                format!("{}...", &content[..max_len])
            } else {
                content.clone()
            };

            // 将内容中的换行符替换为可见表示
            let display_content = display_content.replace('\n', "↵");
            let display_content = display_content.replace('\t', "→");

            output.push_str(&format!("\"{}   {}{}\n", name, display_content, linewise));
        }

        output
    }

    // ==================== 缓冲区管理 ====================

    /// 获取所有缓冲区的列表
    pub fn list_buffers(&self) -> Vec<(BufferId, String, bool)> {
        let mut result = Vec::new();
        
        for (id, buffer) in &self.buffers {
            let name = buffer.file_path()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("[缓冲区 {}]", id.as_usize()));
            
            let is_current = *id == self.current_buffer;
            result.push((*id, name, is_current));
        }
        
        // 按缓冲区ID排序
        result.sort_by_key(|(id, _, _)| id.as_usize());
        result
    }

    /// 格式化缓冲区列表为字符串（用于 :ls 命令）
    pub fn format_buffer_list(&self) -> String {
        let buffers = self.list_buffers();
        if buffers.is_empty() {
            return "没有缓冲区".to_string();
        }

        let mut output = String::from("缓冲区列表:\n");
        output.push_str(&"-".repeat(40));
        output.push('\n');

        for (id, name, is_current) in buffers {
            let current_mark = if is_current { "%" } else { " " };
            let buffer = self.buffers.get(&id).unwrap();
            
            // 状态标记
            let modified_mark = if buffer.is_modified() { "+" } else { " " };
            
            output.push_str(&format!(
                "{}{}  {}  {}\n",
                current_mark,
                modified_mark,
                id.as_usize(),
                name
            ));
        }

        output
    }

    /// 切换到指定缓冲区
    pub fn switch_buffer(&mut self, buffer_id: BufferId) -> Result<(), String> {
        if !self.buffers.contains_key(&buffer_id) {
            return Err(format!("缓冲区 {} 不存在", buffer_id.as_usize()));
        }

        // 保存当前缓冲区的光标位置
        self.buffer_cursors.insert(self.current_buffer, self.cursor.clone());

        // 切换缓冲区
        self.current_buffer = buffer_id;
        
        // 恢复该缓冲区的光标位置
        self.cursor = self.buffer_cursors.get(&buffer_id).cloned().unwrap_or_else(Cursor::at_origin);
        
        Ok(())
    }

    /// 切换到下一个缓冲区
    pub fn next_buffer(&mut self) -> Result<(), String> {
        let mut buffer_ids: Vec<BufferId> = self.buffers.keys().cloned().collect();
        if buffer_ids.len() <= 1 {
            return Err("没有其他缓冲区".to_string());
        }

        // 按ID排序
        buffer_ids.sort_by_key(|id| id.as_usize());

        // 找到当前缓冲区的索引
        let current_idx = buffer_ids.iter().position(|&id| id == self.current_buffer).unwrap_or(0);
        
        // 计算下一个索引（循环）
        let next_idx = (current_idx + 1) % buffer_ids.len();
        let next_buffer_id = buffer_ids[next_idx];

        self.switch_buffer(next_buffer_id)
    }

    /// 创建新的空缓冲区
    pub fn create_new_buffer(&mut self) {
        // 保存当前缓冲区的光标位置
        self.buffer_cursors.insert(self.current_buffer, self.cursor.clone());

        // 创建新缓冲区
        let buffer_id = BufferId::new(self.next_buffer_id);
        self.next_buffer_id += 1;

        let buffer = Buffer::new(buffer_id);
        self.buffers.insert(buffer_id, buffer);
        self.buffer_cursors.insert(buffer_id, Cursor::at_origin());

        // 切换到新缓冲区
        self.current_buffer = buffer_id;
        self.cursor = Cursor::at_origin();
        self.mode = Mode::Normal;
    }

    /// 切换到上一个缓冲区
    pub fn prev_buffer(&mut self) -> Result<(), String> {
        let mut buffer_ids: Vec<BufferId> = self.buffers.keys().cloned().collect();
        if buffer_ids.len() <= 1 {
            return Err("没有其他缓冲区".to_string());
        }

        // 按ID排序
        buffer_ids.sort_by_key(|id| id.as_usize());

        // 找到当前缓冲区的索引
        let current_idx = buffer_ids.iter().position(|&id| id == self.current_buffer).unwrap_or(0);
        
        // 计算上一个索引（循环）
        let prev_idx = if current_idx == 0 {
            buffer_ids.len() - 1
        } else {
            current_idx - 1
        };
        let prev_buffer_id = buffer_ids[prev_idx];

        self.switch_buffer(prev_buffer_id)
    }

    /// 删除缓冲区
    pub fn delete_buffer(&mut self, buffer_id: BufferId) -> Result<(), String> {
        if !self.buffers.contains_key(&buffer_id) {
            return Err(format!("缓冲区 {} 不存在", buffer_id.as_usize()));
        }

        // 检查是否有未保存的修改
        if let Some(buffer) = self.buffers.get(&buffer_id) {
            if buffer.is_modified() {
                return Err(format!("缓冲区 {} 有未保存的修改，请使用 :bd! 强制删除", buffer_id.as_usize()));
            }
        }

        // 如果删除的是当前缓冲区，需要先切换到其他缓冲区
        if buffer_id == self.current_buffer {
            let other_buffer = self.buffers.keys()
                .find(|&&id| id != buffer_id)
                .cloned();
            
            if let Some(other_id) = other_buffer {
                self.switch_buffer(other_id)?;
            } else {
                // 没有其他缓冲区，创建一个新的空缓冲区
                let new_buffer = Buffer::new(BufferId::new(self.next_buffer_id));
                self.next_buffer_id += 1;
                let new_id = new_buffer.id();
                self.buffers.insert(new_id, new_buffer);
                self.buffer_cursors.insert(new_id, Cursor::at_origin());
                self.current_buffer = new_id;
                self.cursor = Cursor::at_origin();
            }
        }

        // 删除缓冲区及其光标记录
        self.buffers.remove(&buffer_id);
        self.buffer_cursors.remove(&buffer_id);

        Ok(())
    }

    /// 强制删除缓冲区（忽略未保存的修改）
    pub fn delete_buffer_force(&mut self, buffer_id: BufferId) -> Result<(), String> {
        if !self.buffers.contains_key(&buffer_id) {
            return Err(format!("缓冲区 {} 不存在", buffer_id.as_usize()));
        }

        // 如果删除的是当前缓冲区，需要先切换到其他缓冲区
        if buffer_id == self.current_buffer {
            let other_buffer = self.buffers.keys()
                .find(|&&id| id != buffer_id)
                .cloned();
            
            if let Some(other_id) = other_buffer {
                self.switch_buffer(other_id)?;
            } else {
                // 没有其他缓冲区，创建一个新的空缓冲区
                let new_buffer = Buffer::new(BufferId::new(self.next_buffer_id));
                self.next_buffer_id += 1;
                let new_id = new_buffer.id();
                self.buffers.insert(new_id, new_buffer);
                self.buffer_cursors.insert(new_id, Cursor::at_origin());
                self.current_buffer = new_id;
                self.cursor = Cursor::at_origin();
            }
        }

        // 删除缓冲区及其光标记录
        self.buffers.remove(&buffer_id);
        self.buffer_cursors.remove(&buffer_id);

        Ok(())
    }

    /// 获取当前缓冲区ID
    pub fn current_buffer_id(&self) -> BufferId {
        self.current_buffer
    }

    /// 获取缓冲区数量
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
