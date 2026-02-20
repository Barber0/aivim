use crate::buffer::Buffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
    pub preferred_column: Option<usize>,
}

impl Cursor {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            preferred_column: None,
        }
    }

    pub fn at_origin() -> Self {
        Self::new(0, 0)
    }

    pub fn to_char_idx(&self, buffer: &Buffer) -> usize {
        let line_start = buffer.line_to_char(self.line);
        line_start + self.column
    }

    pub fn from_char_idx(buffer: &Buffer, char_idx: usize) -> Self {
        let line = buffer.char_to_line(char_idx);
        let line_start = buffer.line_to_char(line);
        let column = char_idx - line_start;
        Self::new(line, column)
    }

    pub fn move_left(&mut self, buffer: &Buffer, count: usize) {
        let _current_line_len = self.get_line_len(buffer);
        
        if self.column >= count {
            self.column -= count;
        } else if self.line > 0 {
            self.line -= 1;
            let prev_line_len = self.get_line_len(buffer);
            self.column = prev_line_len.saturating_sub(1).min(self.preferred_column.unwrap_or(0));
        } else {
            self.column = 0;
        }
        self.update_preferred_column();
    }

    pub fn move_right(&mut self, buffer: &Buffer, count: usize) {
        let current_line_len = self.get_line_len(buffer);
        let max_col = current_line_len.saturating_sub(1);
        
        if self.column + count <= max_col {
            self.column += count;
        } else if self.line + 1 < buffer.len_lines() {
            self.line += 1;
            self.column = 0;
        } else {
            self.column = max_col;
        }
        self.update_preferred_column();
    }

    pub fn move_up(&mut self, buffer: &Buffer, count: usize) {
        if self.line >= count {
            self.line -= count;
        } else {
            self.line = 0;
        }
        self.adjust_column(buffer);
    }

    pub fn move_down(&mut self, buffer: &Buffer, count: usize) {
        let max_line = buffer.len_lines().saturating_sub(1);
        self.line = (self.line + count).min(max_line);
        self.adjust_column(buffer);
    }

    pub fn move_to_line_start(&mut self) {
        self.column = 0;
        self.update_preferred_column();
    }

    pub fn move_to_line_end(&mut self, buffer: &Buffer) {
        let line_len = self.get_line_len(buffer);
        self.column = line_len.saturating_sub(1);
        self.update_preferred_column();
    }

    pub fn move_to_first_non_blank(&mut self, buffer: &Buffer) {
        if let Some(line) = buffer.line(self.line) {
            let line_str = line.to_string();
            let first_non_blank = line_str
                .chars()
                .position(|c| !c.is_whitespace())
                .unwrap_or(0);
            self.column = first_non_blank;
            self.update_preferred_column();
        }
    }

    pub fn move_to_line(&mut self, line: usize, buffer: &Buffer) {
        let max_line = buffer.len_lines().saturating_sub(1);
        self.line = line.min(max_line);
        self.adjust_column(buffer);
    }

    pub fn move_to_top(&mut self, buffer: &Buffer) {
        self.line = 0;
        self.adjust_column(buffer);
    }

    pub fn move_to_bottom(&mut self, buffer: &Buffer) {
        self.line = buffer.len_lines().saturating_sub(1);
        self.adjust_column(buffer);
    }

    fn get_line_len(&self, buffer: &Buffer) -> usize {
        buffer.line(self.line).map(|l| l.len_chars()).unwrap_or(0)
    }

    fn adjust_column(&mut self, buffer: &Buffer) {
        let line_len = self.get_line_len(buffer);
        let max_col = line_len.saturating_sub(1);
        
        if let Some(preferred) = self.preferred_column {
            self.column = preferred.min(max_col);
        } else {
            self.column = self.column.min(max_col);
        }
    }

    pub fn update_preferred_column(&mut self) {
        self.preferred_column = Some(self.column);
    }

    pub fn ensure_valid(&mut self, buffer: &Buffer) {
        let max_line = buffer.len_lines().saturating_sub(1);
        self.line = self.line.min(max_line);
        
        let line_len = self.get_line_len(buffer);
        let max_col = line_len.saturating_sub(1);
        self.column = self.column.min(max_col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffer, BufferId};

    #[test]
    fn test_cursor_movement() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "Line 1\nLine 2\nLine 3");
        
        let mut cursor = Cursor::at_origin();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 0);
        
        cursor.move_down(&buffer, 1);
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.column, 0);
        
        cursor.move_right(&buffer, 3);
        assert_eq!(cursor.column, 3);
        
        cursor.move_up(&buffer, 1);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 3);
    }

    #[test]
    fn test_cursor_line_start_end() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "Hello, World!");
        
        let mut cursor = Cursor::new(0, 5);
        
        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.column, 12);
        
        cursor.move_to_line_start();
        assert_eq!(cursor.column, 0);
    }

    #[test]
    fn test_cursor_char_idx_conversion() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "Line 1\nLine 2\nLine 3");
        
        let cursor = Cursor::new(1, 2);
        let char_idx = cursor.to_char_idx(&buffer);
        assert_eq!(char_idx, 9);
        
        let cursor2 = Cursor::from_char_idx(&buffer, char_idx);
        assert_eq!(cursor2.line, 1);
        assert_eq!(cursor2.column, 2);
    }
}
