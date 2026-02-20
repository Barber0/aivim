use crate::buffer::Buffer;
use crate::cursor::Cursor;

pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
    FirstNonBlank,
    WordForward,
    WordBackward,
    WordEnd,
    PageUp,
    PageDown,
    DocumentStart,
    DocumentEnd,
    GoToLine(usize),
}

impl Motion {
    pub fn execute(&self, cursor: &mut Cursor, buffer: &Buffer) {
        match self {
            Motion::Left => cursor.move_left(buffer, 1),
            Motion::Right => cursor.move_right(buffer, 1),
            Motion::Up => cursor.move_up(buffer, 1),
            Motion::Down => cursor.move_down(buffer, 1),
            Motion::LineStart => cursor.move_to_line_start(),
            Motion::LineEnd => cursor.move_to_line_end(buffer),
            Motion::FirstNonBlank => cursor.move_to_first_non_blank(buffer),
            Motion::WordForward => move_word_forward(cursor, buffer),
            Motion::WordBackward => move_word_backward(cursor, buffer),
            Motion::WordEnd => move_word_end(cursor, buffer),
            Motion::PageUp => cursor.move_up(buffer, 20),
            Motion::PageDown => cursor.move_down(buffer, 20),
            Motion::DocumentStart => cursor.move_to_top(buffer),
            Motion::DocumentEnd => cursor.move_to_bottom(buffer),
            Motion::GoToLine(line) => cursor.move_to_line(*line, buffer),
        }
    }
}

fn move_word_forward(cursor: &mut Cursor, buffer: &Buffer) {
    let current_line = cursor.line;
    let current_col = cursor.column;
    let line_text = buffer.line(current_line).map(|l| l.to_string()).unwrap_or_default();
    
    // 移除行尾换行符
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    
    if current_col >= line_text.len() {
        // 已经在行尾，不移动到下一行，保持在当前位置
        return;
    }
    
    let remaining = &line_text[current_col..];
    let mut chars = remaining.chars().peekable();
    
    // 跳过当前单词的剩余部分
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
    
    // 跳过空白字符（但不包括换行，因为我们只在本行内移动）
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() && ch != '\n' {
            chars.next();
        } else {
            break;
        }
    }
    
    let consumed = remaining.len() - chars.collect::<String>().len();
    let new_col = current_col + consumed;

    // 确保不超出本行范围
    // 允许跳到行尾（line_text.len()），这样 dw 可以删除最后一个单词
    let final_col = new_col.min(line_text.len());
    cursor.column = final_col;
    cursor.update_preferred_column();
}

fn move_word_backward(cursor: &mut Cursor, buffer: &Buffer) {
    let current_line = cursor.line;
    let current_col = cursor.column;
    
    // 如果已经在行首，不移动到上一行
    if current_col == 0 {
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
