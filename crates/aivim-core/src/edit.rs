use crate::buffer::Buffer;
use crate::cursor::Cursor;

pub enum Edit {
    InsertChar(char),
    InsertNewline,
    DeleteChar,
    DeleteLine,
    Backspace,
    DeleteWord,
    ChangeLine,
    YankLine,
    Paste,
}

impl Edit {
    pub fn execute(&self, cursor: &mut Cursor, buffer: &mut Buffer) -> Option<EditResult> {
        match self {
            Edit::InsertChar(ch) => {
                let char_idx = cursor.to_char_idx(buffer);
                buffer.insert_char(char_idx, *ch);
                cursor.move_right(buffer, 1);
                None
            }
            Edit::InsertNewline => {
                let char_idx = cursor.to_char_idx(buffer);
                buffer.insert(char_idx, "\n");
                cursor.line += 1;
                cursor.column = 0;
                None
            }
            Edit::DeleteChar => {
                let char_idx = cursor.to_char_idx(buffer);
                let deleted = buffer.remove_char(char_idx);
                deleted.map(|ch| EditResult::DeletedChar(ch))
            }
            Edit::DeleteLine => {
                let line_idx = cursor.line;
                if let Some(line) = buffer.line(line_idx) {
                    let line_text = line.to_string();
                    let line_start = buffer.line_to_char(line_idx);
                    let line_len = line.len_chars();
                    
                    buffer.remove(line_start, line_len);
                    
                    if buffer.len_lines() == 0 {
                        buffer.insert(0, "");
                    }
                    
                    cursor.ensure_valid(buffer);
                    
                    Some(EditResult::DeletedLine(line_text))
                } else {
                    None
                }
            }
            Edit::Backspace => {
                if cursor.column > 0 {
                    let char_idx = cursor.to_char_idx(buffer);
                    let deleted = buffer.remove_char(char_idx - 1);
                    cursor.move_left(buffer, 1);
                    deleted.map(|ch| EditResult::DeletedChar(ch))
                } else if cursor.line > 0 {
                    let current_line = cursor.line;
                    let prev_line_len = buffer.line_len(current_line - 1);
                    
                    let current_line_start = buffer.line_to_char(current_line);
                    buffer.remove(current_line_start - 1, 1);
                    
                    cursor.line -= 1;
                    cursor.column = prev_line_len;
                    
                    None
                } else {
                    None
                }
            }
            Edit::DeleteWord => {
                let start_idx = cursor.to_char_idx(buffer);
                let text = buffer.rope().to_string();
                
                if start_idx >= text.len() {
                    return None;
                }
                
                let remaining = &text[start_idx..];
                let mut chars = remaining.chars().peekable();
                
                // 第一阶段：删除单词字符
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                // 第二阶段：删除跟随的非单词字符（但不包括换行符）
                while let Some(&ch) = chars.peek() {
                    // 遇到换行符立即停止，不跨行删除
                    if ch == '\n' {
                        break;
                    }
                    if !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                // 第三阶段：删除跟随的空白字符（但不包括换行符）
                while let Some(&ch) = chars.peek() {
                    // 遇到换行符立即停止，不跨行删除
                    if ch == '\n' {
                        break;
                    }
                    if ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                let consumed = remaining.len() - chars.collect::<String>().len();
                if consumed > 0 {
                    buffer.remove(start_idx, consumed);
                    Some(EditResult::DeletedText(remaining[..consumed].to_string()))
                } else {
                    None
                }
            }
            Edit::ChangeLine => {
                let line_idx = cursor.line;
                if let Some(line) = buffer.line(line_idx) {
                    let line_text = line.to_string();
                    let line_start = buffer.line_to_char(line_idx);
                    let line_len = line.len_chars();
                    
                    buffer.remove(line_start, line_len);
                    cursor.column = 0;
                    
                    Some(EditResult::DeletedLine(line_text))
                } else {
                    None
                }
            }
            Edit::YankLine => {
                let line_idx = cursor.line;
                if let Some(line) = buffer.line(line_idx) {
                    let line_text = line.to_string();
                    Some(EditResult::YankedText(line_text))
                } else {
                    None
                }
            }
            Edit::Paste => {
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EditResult {
    DeletedChar(char),
    DeletedText(String),
    DeletedLine(String),
    YankedText(String),
}
