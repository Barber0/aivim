/// 文本对象（Text Objects）
///
/// 实现 Vim 风格的文本对象，如 aw, iw, as, is 等
///
/// 文本对象定义了一个文本范围，可以与操作符（d, y, c）配合使用

use crate::buffer::Buffer;
use crate::cursor::Cursor;

/// 文本对象类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextObject {
    /// Around Word - 单词及周围空格
    AroundWord,
    /// Inner Word - 仅单词本身
    InnerWord,
    /// Around Sentence - 句子及周围空格
    AroundSentence,
    /// Inner Sentence - 仅句子本身
    InnerSentence,
    /// Around Paragraph - 段落及周围空行
    AroundParagraph,
    /// Inner Paragraph - 仅段落本身
    InnerParagraph,
}

impl TextObject {
    /// 获取文本对象的范围（start_char_idx, end_char_idx）
    pub fn get_range(&self, cursor: &Cursor, buffer: &Buffer) -> Option<(usize, usize)> {
        match self {
            TextObject::AroundWord => Self::get_around_word_range(cursor, buffer),
            TextObject::InnerWord => Self::get_inner_word_range(cursor, buffer),
            TextObject::AroundSentence => Self::get_around_sentence_range(cursor, buffer),
            TextObject::InnerSentence => Self::get_inner_sentence_range(cursor, buffer),
            TextObject::AroundParagraph => Self::get_around_paragraph_range(cursor, buffer),
            TextObject::InnerParagraph => Self::get_inner_paragraph_range(cursor, buffer),
        }
    }

    /// 获取 aw（around word）的范围
    /// 包括当前单词及其后的一个空格（如果有）
    fn get_around_word_range(cursor: &Cursor, buffer: &Buffer) -> Option<(usize, usize)> {
        let line_text = buffer.line(cursor.line)?.to_string();
        let line_start = buffer.line_to_char(cursor.line);
        let col = cursor.column.min(line_text.len().saturating_sub(1));

        // 找到当前单词的边界
        let (word_start, word_end) = Self::find_word_boundaries(&line_text, col)?;

        // 检查单词后是否有空格
        let has_trailing_space = word_end < line_text.len()
            && line_text[word_end..].starts_with(' ');

        let end = if has_trailing_space {
            word_end + 1  // 包含后面的空格
        } else {
            word_end
        };

        Some((line_start + word_start, line_start + end))
    }

    /// 获取 iw（inner word）的范围
    /// 仅包括当前单词本身
    fn get_inner_word_range(cursor: &Cursor, buffer: &Buffer) -> Option<(usize, usize)> {
        let line_text = buffer.line(cursor.line)?.to_string();
        let line_start = buffer.line_to_char(cursor.line);
        let col = cursor.column.min(line_text.len().saturating_sub(1));

        // 找到当前单词的边界
        let (word_start, word_end) = Self::find_word_boundaries(&line_text, col)?;

        Some((line_start + word_start, line_start + word_end))
    }

    /// 找到单词的边界（start, end）
    fn find_word_boundaries(line_text: &str, col: usize) -> Option<(usize, usize)> {
        let chars: Vec<char> = line_text.chars().collect();
        let col = col.min(chars.len().saturating_sub(1));

        // 确定当前位置的字符类型
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        // 如果当前位置是空白字符，向前找单词
        let start_col = if col < chars.len() && !is_word_char(chars[col]) {
            // 向前查找单词开始
            chars.iter()
                .enumerate()
                .skip(col)
                .find(|(_, c)| is_word_char(**c))
                .map(|(i, _)| i)?
        } else {
            col
        };

        // 找到单词开始位置（向前查找）
        let word_start = chars[..=start_col]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, c)| !is_word_char(**c))
            .map(|(i, _)| i + 1)
            .unwrap_or(0);

        // 找到单词结束位置（向后查找）
        let word_end = chars[start_col..]
            .iter()
            .enumerate()
            .find(|(_, c)| !is_word_char(**c))
            .map(|(i, _)| start_col + i)
            .unwrap_or(chars.len());

        Some((word_start, word_end))
    }

    /// 获取 as（around sentence）的范围
    fn get_around_sentence_range(_cursor: &Cursor, _buffer: &Buffer) -> Option<(usize, usize)> {
        // TODO: 实现句子文本对象
        None
    }

    /// 获取 is（inner sentence）的范围
    fn get_inner_sentence_range(_cursor: &Cursor, _buffer: &Buffer) -> Option<(usize, usize)> {
        // TODO: 实现句子文本对象
        None
    }

    /// 获取 ap（around paragraph）的范围
    fn get_around_paragraph_range(_cursor: &Cursor, _buffer: &Buffer) -> Option<(usize, usize)> {
        // TODO: 实现段落文本对象
        None
    }

    /// 获取 ip（inner paragraph）的范围
    fn get_inner_paragraph_range(_cursor: &Cursor, _buffer: &Buffer) -> Option<(usize, usize)> {
        // TODO: 实现段落文本对象
        None
    }
}

/// 从字符解析文本对象
pub fn parse_text_object(ch: char) -> Option<TextObject> {
    match ch {
        'w' => Some(TextObject::AroundWord),
        'W' => Some(TextObject::InnerWord),
        's' => Some(TextObject::AroundSentence),
        'S' => Some(TextObject::InnerSentence),
        'p' => Some(TextObject::AroundParagraph),
        'P' => Some(TextObject::InnerParagraph),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffer, BufferId};

    #[test]
    fn test_inner_word() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "hello world vim\n");

        // 光标在 "world" 上
        let cursor = Cursor::new(0, 6);
        let range = TextObject::InnerWord.get_range(&cursor, &buffer);

        assert_eq!(range, Some((6, 11)));  // "world"
    }

    #[test]
    fn test_around_word_with_space() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "hello world vim\n");

        // 光标在 "world" 上
        let cursor = Cursor::new(0, 6);
        let range = TextObject::AroundWord.get_range(&cursor, &buffer);

        assert_eq!(range, Some((6, 12)));  // "world "（包含后面的空格）
    }

    #[test]
    fn test_around_word_at_end() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "hello world\n");

        // 光标在最后一个单词 "world" 上
        let cursor = Cursor::new(0, 6);
        let range = TextObject::AroundWord.get_range(&cursor, &buffer);

        assert_eq!(range, Some((6, 11)));  // "world"（没有后面的空格）
    }
}
