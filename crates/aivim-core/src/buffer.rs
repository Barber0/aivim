use ropey::Rope;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Buffer {
    id: BufferId,
    rope: Rope,
    file_path: Option<PathBuf>,
    modified: bool,
    read_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(usize);

impl BufferId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl Buffer {
    pub fn new(id: BufferId) -> Self {
        Self {
            id,
            rope: Rope::new(),
            file_path: None,
            modified: false,
            read_only: false,
        }
    }

    pub fn from_file(id: BufferId, path: &Path) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let rope = Rope::from_str(&content);

        Ok(Self {
            id,
            rope,
            file_path: Some(path.to_path_buf()),
            modified: false,
            read_only: false,
        })
    }

    pub fn new_with_path(id: BufferId, path: &Path) -> Self {
        Self {
            id,
            rope: Rope::new(),
            file_path: Some(path.to_path_buf()),
            modified: false,
            read_only: false,
        }
    }

    pub fn id(&self) -> BufferId {
        self.id
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    pub fn line(&self, line_idx: usize) -> Option<ropey::RopeSlice> {
        if line_idx < self.rope.len_lines() {
            Some(self.rope.line(line_idx))
        } else {
            None
        }
    }

    pub fn line_len(&self, line_idx: usize) -> usize {
        self.line(line_idx).map(|l| l.len_chars()).unwrap_or(0)
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) {
        if self.read_only {
            return;
        }
        self.rope.insert(char_idx, text);
        self.modified = true;
    }

    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        if self.read_only {
            return;
        }
        self.rope.insert_char(char_idx, ch);
        self.modified = true;
    }

    pub fn remove(&mut self, char_idx: usize, len: usize) {
        if self.read_only {
            return;
        }
        let end_idx = (char_idx + len).min(self.rope.len_chars());
        self.rope.remove(char_idx..end_idx);
        self.modified = true;
    }

    pub fn remove_char(&mut self, char_idx: usize) -> Option<char> {
        if self.read_only || char_idx >= self.rope.len_chars() {
            return None;
        }
        let ch = self.rope.char(char_idx);
        self.rope.remove(char_idx..char_idx + 1);
        self.modified = true;
        Some(ch)
    }

    pub fn char(&self, char_idx: usize) -> char {
        self.rope.char(char_idx)
    }

    pub fn slice(&self, range: std::ops::Range<usize>) -> ropey::RopeSlice {
        self.rope.slice(range)
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }

    pub fn save(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.file_path {
            let mut file = fs::File::create(path)?;
            for chunk in self.rope.chunks() {
                file.write_all(chunk.as_bytes())?;
            }
            // 确保文件以换行符结尾（Unix文本文件惯例）
            if self.rope.len_chars() > 0 {
                let last_char = self.rope.char(self.rope.len_chars() - 1);
                if last_char != '\n' {
                    file.write_all(b"\n")?;
                }
            }
            self.modified = false;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No file path set",
            ))
        }
    }

    pub fn save_as(&mut self, path: &Path) -> io::Result<()> {
        self.file_path = Some(path.to_path_buf());
        self.save()
    }

    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = Buffer::new(BufferId::new(0));
        assert!(buffer.is_empty());
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_buffer_insert() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "Hello, World!");
        assert_eq!(buffer.to_string(), "Hello, World!");
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_buffer_remove() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "Hello, World!");
        buffer.remove(7, 5);
        assert_eq!(buffer.to_string(), "Hello, !");
    }

    #[test]
    fn test_buffer_line_operations() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.len_lines(), 3);
        
        let line0 = buffer.line(0).unwrap();
        assert_eq!(line0.to_string(), "Line 1\n");
        
        let line1 = buffer.line(1).unwrap();
        assert_eq!(line1.to_string(), "Line 2\n");
        
        let line2 = buffer.line(2).unwrap();
        assert_eq!(line2.to_string(), "Line 3");
    }
}
