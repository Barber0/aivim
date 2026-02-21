/// 光标移动（Motion）模块
///
/// 实现 Vim 风格的光标移动命令，如 w, b, e, $, 0 等

use crate::buffer::Buffer;
use crate::cursor::Cursor;

/// 光标移动命令
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    /// 左移一个字符 (h)
    Left,
    /// 右移一个字符 (l)
    Right,
    /// 上移一行 (k)
    Up,
    /// 下移一行 (j)
    Down,
    /// 移到行首 (0)
    LineStart,
    /// 移到行尾 ($)
    LineEnd,
    /// 移到第一个非空字符 (^)
    FirstNonBlank,
    /// 向前移动一个单词 (w)
    WordForward,
    /// 向后移动一个单词 (b)
    WordBackward,
    /// 向前移动到单词结尾 (e)
    WordEnd,
    /// 向前移动一个 WORD（大写，以空白分隔）(W)
    WORDForward,
    /// 向后移动一个 WORD（大写）(B)
    WORDBackward,
    /// 向前移动到 WORD 结尾 (E)
    WORDEnd,
    /// 移到文件开头 (gg)
    DocumentStart,
    /// 移到文件结尾 (G)
    DocumentEnd,
    /// 向下移动半页 (Ctrl+D)
    PageDown,
    /// 向上移动半页 (Ctrl+U)
    PageUp,
}

impl Motion {
    /// 执行光标移动
    pub fn execute(&self, cursor: &mut Cursor, buffer: &Buffer) {
        match self {
            Motion::Left => move_left(cursor, buffer),
            Motion::Right => move_right(cursor, buffer),
            Motion::Up => move_up(cursor, buffer),
            Motion::Down => move_down(cursor, buffer),
            Motion::LineStart => move_line_start(cursor),
            Motion::LineEnd => move_line_end(cursor, buffer),
            Motion::FirstNonBlank => move_first_non_blank(cursor, buffer),
            Motion::WordForward => move_word_forward(cursor, buffer),
            Motion::WordBackward => move_word_backward(cursor, buffer),
            Motion::WordEnd => move_word_end(cursor, buffer),
            Motion::WORDForward => move_word_forward(cursor, buffer), // 简化实现
            Motion::WORDBackward => move_word_backward(cursor, buffer), // 简化实现
            Motion::WORDEnd => move_word_end(cursor, buffer), // 简化实现
            Motion::DocumentStart => move_document_start(cursor),
            Motion::DocumentEnd => move_document_end(cursor, buffer),
            Motion::PageDown => page_down(cursor, buffer),
            Motion::PageUp => page_up(cursor),
        }
    }
}

fn move_left(cursor: &mut Cursor, _buffer: &Buffer) {
    if cursor.column > 0 {
        cursor.column -= 1;
        cursor.update_preferred_column();
    }
}

fn move_right(cursor: &mut Cursor, buffer: &Buffer) {
    let line_text = buffer.line(cursor.line).map(|l| l.to_string()).unwrap_or_default();
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    if cursor.column < line_text.len().saturating_sub(1) {
        cursor.column += 1;
        cursor.update_preferred_column();
    }
}

fn move_up(cursor: &mut Cursor, buffer: &Buffer) {
    if cursor.line > 0 {
        cursor.line -= 1;
        // 确保列位置不超过新行的长度
        let line_text = buffer.line(cursor.line).map(|l| l.to_string()).unwrap_or_default();
        let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
        cursor.column = cursor.column.min(line_text.len().saturating_sub(1));
    }
}

fn move_down(cursor: &mut Cursor, buffer: &Buffer) {
    if cursor.line + 1 < buffer.len_lines() {
        cursor.line += 1;
        // 确保列位置不超过新行的长度
        let line_text = buffer.line(cursor.line).map(|l| l.to_string()).unwrap_or_default();
        let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
        cursor.column = cursor.column.min(line_text.len().saturating_sub(1));
    }
}

fn move_line_start(cursor: &mut Cursor) {
    cursor.column = 0;
    cursor.update_preferred_column();
}

fn move_line_end(cursor: &mut Cursor, buffer: &Buffer) {
    let line_text = buffer.line(cursor.line).map(|l| l.to_string()).unwrap_or_default();
    // 移除行尾换行符
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    // 行尾是 line_text.len()，但光标应该停在最后一个字符
    cursor.column = line_text.len().saturating_sub(1);
    cursor.update_preferred_column();
}

fn move_first_non_blank(cursor: &mut Cursor, buffer: &Buffer) {
    let line_text = buffer.line(cursor.line).map(|l| l.to_string()).unwrap_or_default();
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    
    // 找到第一个非空白字符
    let first_non_blank = line_text
        .chars()
        .position(|ch| !ch.is_whitespace())
        .unwrap_or(0);
    
    cursor.column = first_non_blank;
    cursor.update_preferred_column();
}

fn move_word_forward(cursor: &mut Cursor, buffer: &Buffer) {
    move_word_forward_internal(cursor, buffer, false);
}

fn move_word_forward_internal(cursor: &mut Cursor, buffer: &Buffer, cross_line: bool) {
    let current_line = cursor.line;
    let current_col = cursor.column;
    let line_text = buffer.line(current_line).map(|l| l.to_string()).unwrap_or_default();

    // 移除行尾换行符
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);

    // 如果已经在行尾或超出，移动到下一行的第一个单词
    if current_col >= line_text.len() {
        if current_line + 1 < buffer.len_lines() {
            cursor.line = current_line + 1;
            cursor.column = 0;
            // 递归调用以处理新行（跳过前导空白，停在第一个单词开头）
            move_word_forward_internal(cursor, buffer, true);
        }
        return;
    }

    let remaining = &line_text[current_col..];
    let mut chars = remaining.chars().peekable();

    // 如果是跨行后的第一次调用，先跳过前导空白，然后停在第一个单词开头
    if cross_line {
        // 跳过空白字符
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
        // 停在当前位置（第一个单词的开头）
        if chars.peek().is_some() {
            let consumed = remaining.len() - chars.collect::<String>().len();
            cursor.column = current_col + consumed;
            cursor.update_preferred_column();
            return;
        }
    }

    // 检查当前位置是否是单词字符
    let at_word = chars.peek().map(|&ch| ch.is_alphanumeric() || ch == '_').unwrap_or(false);

    if at_word {
        // 当前位置是单词，跳过这个单词的剩余部分
        while let Some(&ch) = chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                chars.next();
            } else {
                break;
            }
        }

        // 跳过标点符号
        while let Some(&ch) = chars.peek() {
            if !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        // 跳过空白字符（包括行尾，允许跨行）
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
    } else {
        // 当前位置不是单词（是空白或标点），跳过空白，停在下一个单词开头
        // 跳过标点符号
        while let Some(&ch) = chars.peek() {
            if !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        // 跳过空白字符
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
        // 停在这里（下一个单词的开头）
    }

    let consumed = remaining.len() - chars.collect::<String>().len();
    let new_col = current_col + consumed;

    // 如果已经到达或超过行尾，尝试移动到下一行
    if new_col >= line_text.len() {
        if current_line + 1 < buffer.len_lines() {
            cursor.line = current_line + 1;
            cursor.column = 0;
            // 递归调用以跳过新行的前导空白
            move_word_forward_internal(cursor, buffer, true);
        } else {
            // 在最后一行，移动到行尾
            cursor.column = line_text.len().saturating_sub(1);
        }
    } else {
        cursor.column = new_col;
    }
    cursor.update_preferred_column();
}

fn move_word_backward(cursor: &mut Cursor, buffer: &Buffer) {
    let current_line = cursor.line;
    let current_col = cursor.column;
    
    // 如果已经在行首，尝试移动到上一行的最后一个单词
    if current_col == 0 {
        if current_line > 0 {
            cursor.line = current_line - 1;
            // 获取上一行的文本
            let prev_line_text = buffer.line(cursor.line).map(|l| l.to_string()).unwrap_or_default();
            let prev_line_text = prev_line_text.strip_suffix('\n').unwrap_or(&prev_line_text);
            // 移动到上一行的行尾，然后递归调用以找到最后一个单词
            cursor.column = prev_line_text.len();
            move_word_backward(cursor, buffer);
        }
        return;
    }
    
    let line_text = buffer.line(current_line).map(|l| l.to_string()).unwrap_or_default();
    
    // 移除行尾换行符
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    
    // 获取当前位置之前的文本
    let preceding = &line_text[..current_col.min(line_text.len())];
    let mut chars: Vec<char> = preceding.chars().collect();
    
    // 跳过空白字符
    while let Some(&ch) = chars.last() {
        if ch.is_whitespace() {
            chars.pop();
        } else {
            break;
        }
    }
    
    // 跳过标点符号
    while let Some(&ch) = chars.last() {
        if !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace() {
            chars.pop();
        } else {
            break;
        }
    }
    
    // 跳过单词
    while let Some(&ch) = chars.last() {
        if ch.is_alphanumeric() || ch == '_' {
            chars.pop();
        } else {
            break;
        }
    }
    
    let new_col = chars.len();
    cursor.column = new_col;
    cursor.update_preferred_column();
}

fn move_word_end(cursor: &mut Cursor, buffer: &Buffer) {
    let current_line = cursor.line;
    let current_col = cursor.column;
    let line_text = buffer.line(current_line).map(|l| l.to_string()).unwrap_or_default();
    
    // 移除行尾换行符
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    
    if current_col >= line_text.len() {
        return;
    }
    
    let remaining = &line_text[current_col..];
    let mut chars = remaining.chars().peekable();
    
    // 跳过空白字符
    if let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() && ch != '\n' {
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }
    
    // 跳过单词
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            chars.next();
        } else {
            break;
        }
    }
    
    // 跳过标点
    while let Some(&ch) = chars.peek() {
        if !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    
    let consumed = remaining.len() - chars.collect::<String>().len();
    let new_col = current_col + consumed.saturating_sub(1);
    
    // 确保不超出本行范围
    let final_col = new_col.min(line_text.len().saturating_sub(1));
    cursor.column = final_col;
    cursor.update_preferred_column();
}

fn move_document_start(cursor: &mut Cursor) {
    cursor.line = 0;
    cursor.column = 0;
    cursor.update_preferred_column();
}

fn move_document_end(cursor: &mut Cursor, buffer: &Buffer) {
    cursor.line = buffer.len_lines().saturating_sub(1);
    let line_text = buffer.line(cursor.line).map(|l| l.to_string()).unwrap_or_default();
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    cursor.column = line_text.len().saturating_sub(1);
    cursor.update_preferred_column();
}

fn page_down(cursor: &mut Cursor, buffer: &Buffer) {
    // 向下移动半页（假设半页为 10 行）
    let half_page = 10;
    cursor.line = (cursor.line + half_page).min(buffer.len_lines().saturating_sub(1));
    let line_text = buffer.line(cursor.line).map(|l| l.to_string()).unwrap_or_default();
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    cursor.column = cursor.column.min(line_text.len().saturating_sub(1));
}

fn page_up(cursor: &mut Cursor) {
    // 向上移动半页
    let half_page = 10;
    cursor.line = cursor.line.saturating_sub(half_page);
}
