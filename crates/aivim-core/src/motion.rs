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
    let char_idx = cursor.to_char_idx(buffer);
    let text = buffer.rope().to_string();
    
    if char_idx >= text.len() {
        return;
    }
    
    let remaining = &text[char_idx..];
    let mut chars = remaining.chars().peekable();
    
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            chars.next();
        } else {
            break;
        }
    }
    
    while let Some(&ch) = chars.peek() {
        if !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    
    let consumed = remaining.len() - chars.collect::<String>().len();
    let new_idx = char_idx + consumed;
    *cursor = Cursor::from_char_idx(buffer, new_idx);
}

fn move_word_backward(cursor: &mut Cursor, buffer: &Buffer) {
    let char_idx = cursor.to_char_idx(buffer);
    
    if char_idx == 0 {
        return;
    }
    
    let text = buffer.rope().to_string();
    let preceding = &text[..char_idx];
    let mut chars: Vec<char> = preceding.chars().collect();
    
    while let Some(&ch) = chars.last() {
        if ch.is_whitespace() {
            chars.pop();
        } else {
            break;
        }
    }
    
    while let Some(&ch) = chars.last() {
        if !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace() {
            chars.pop();
        } else {
            break;
        }
    }
    
    while let Some(&ch) = chars.last() {
        if ch.is_alphanumeric() || ch == '_' {
            chars.pop();
        } else {
            break;
        }
    }
    
    let new_idx = chars.len();
    *cursor = Cursor::from_char_idx(buffer, new_idx);
}

fn move_word_end(cursor: &mut Cursor, buffer: &Buffer) {
    let char_idx = cursor.to_char_idx(buffer);
    let text = buffer.rope().to_string();
    
    if char_idx >= text.len() {
        return;
    }
    
    let remaining = &text[char_idx..];
    let mut chars = remaining.chars().peekable();
    
    if let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }
    
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            chars.next();
        } else {
            break;
        }
    }
    
    while let Some(&ch) = chars.peek() {
        if !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    
    let consumed = remaining.len() - chars.collect::<String>().len();
    let new_idx = char_idx + consumed.saturating_sub(1);
    *cursor = Cursor::from_char_idx(buffer, new_idx);
}
