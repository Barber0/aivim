/// 寄存器系统 - 实现Vim的寄存器功能
///
/// 寄存器类型:
/// - `"` (unnamed): 默认寄存器，删除/复制操作自动使用
/// - `0-9`: 数字寄存器，存储删除历史
/// - `a-z`: 命名寄存器，用户指定
/// - `+` / `*`: 系统剪贴板
/// - `_`: 黑洞寄存器，内容被丢弃
/// - `/`: 搜索寄存器

#[derive(Debug, Clone)]
pub struct Register {
    pub name: char,
    pub content: String,
    pub linewise: bool,  // true表示整行操作，false表示字符操作
}

impl Register {
    pub fn new(name: char, content: impl Into<String>, linewise: bool) -> Self {
        Self {
            name,
            content: content.into(),
            linewise,
        }
    }

    pub fn empty(name: char) -> Self {
        Self {
            name,
            content: String::new(),
            linewise: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn lines(&self) -> Vec<&str> {
        self.content.lines().collect()
    }
}

#[derive(Debug, Clone)]
pub struct RegisterManager {
    /// 无名寄存器 "
    unnamed: Register,
    /// 小删除寄存器 - (删除少于一行)
    small_delete: Register,
    /// 数字寄存器 0-9 (0是最新，9是最旧)
    numbered: Vec<Register>,
    /// 命名寄存器 a-z
    named: std::collections::HashMap<char, Register>,
    /// 只读寄存器
    readonly: std::collections::HashMap<char, Register>,
    /// 搜索寄存器 /
    search: Register,
}

impl RegisterManager {
    pub fn new() -> Self {
        let mut readonly = std::collections::HashMap::new();
        // % 寄存器: 当前文件名
        readonly.insert('%', Register::empty('%'));
        // # 寄存器: 备用文件名
        readonly.insert('#', Register::empty('#'));
        // : 寄存器: 上次执行的命令
        readonly.insert(':', Register::empty(':'));
        // . 寄存器: 上次插入的文本
        readonly.insert('.', Register::empty('.'));

        Self {
            unnamed: Register::empty('"'),
            small_delete: Register::empty('-'),
            numbered: (0..=9).map(|i| Register::empty(std::char::from_digit(i, 10).unwrap())).collect(),
            named: std::collections::HashMap::new(),
            readonly,
            search: Register::empty('/'),
        }
    }

    /// 获取寄存器内容
    pub fn get(&self, name: char) -> Option<&Register> {
        match name {
            '"' => Some(&self.unnamed),
            '-' => Some(&self.small_delete),
            '0'..='9' => self.numbered.get(name.to_digit(10).unwrap() as usize),
            'a'..='z' | 'A'..='Z' => self.named.get(&name.to_ascii_lowercase()),
            '%' | '#' | ':' | '.' => self.readonly.get(&name),
            '/' => Some(&self.search),
            _ => None,
        }
    }

    /// 设置寄存器内容
    pub fn set(&mut self, name: char, content: impl Into<String>, linewise: bool) {
        let content = content.into();
        match name {
            '"' => self.unnamed = Register::new('"', content, linewise),
            '-' => self.small_delete = Register::new('-', content, linewise),
            '0'..='9' => {
                let idx = name.to_digit(10).unwrap() as usize;
                if idx < self.numbered.len() {
                    self.numbered[idx] = Register::new(name, content, linewise);
                }
            }
            'a'..='z' => {
                self.named.insert(name, Register::new(name, content, linewise));
            }
            'A'..='Z' => {
                // 大写表示追加
                let lower = name.to_ascii_lowercase();
                if let Some(existing) = self.named.get_mut(&lower) {
                    existing.content.push_str(&content);
                    existing.linewise = linewise;
                } else {
                    self.named.insert(lower, Register::new(lower, content, linewise));
                }
            }
            '/' => self.search = Register::new('/', content, linewise),
            _ => {}
        }
    }

    /// 设置无名寄存器，同时更新数字寄存器
    pub fn set_unnamed(&mut self, content: impl Into<String>, linewise: bool) {
        let content = content.into();

        // 将0-8的内容移到1-9
        for i in (1..=8).rev() {
            self.numbered[i + 1] = self.numbered[i].clone();
        }

        // 原来的无名寄存器内容移到0
        self.numbered[1] = self.unnamed.clone();

        // 设置新的无名寄存器
        self.unnamed = Register::new('"', content, linewise);

        // 新内容也放入0号寄存器
        self.numbered[0] = self.unnamed.clone();
    }

    /// 设置小删除寄存器
    pub fn set_small_delete(&mut self, content: impl Into<String>) {
        self.small_delete = Register::new('-', content, false);
    }

    /// 设置搜索寄存器
    pub fn set_search(&mut self, pattern: impl Into<String>) {
        self.search = Register::new('/', pattern, false);
    }

    /// 获取搜索寄存器内容
    pub fn get_search(&self) -> &str {
        &self.search.content
    }

    /// 更新只读寄存器
    pub fn set_readonly(&mut self, name: char, content: impl Into<String>) {
        if self.readonly.contains_key(&name) {
            self.readonly.insert(name, Register::new(name, content, false));
        }
    }

    /// 获取系统剪贴板内容（模拟）
    pub fn get_clipboard(&self) -> Option<String> {
        // TODO: 实际实现需要使用外部crate如arboard
        None
    }

    /// 设置系统剪贴板内容（模拟）
    pub fn set_clipboard(&mut self, _content: &str) {
        // TODO: 实际实现需要使用外部crate如arboard
    }
}

impl Default for RegisterManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unnamed_register() {
        let mut manager = RegisterManager::new();
        manager.set_unnamed("hello", false);

        assert_eq!(manager.get('"').unwrap().content, "hello");
        assert_eq!(manager.get('0').unwrap().content, "hello");
    }

    #[test]
    fn test_numbered_registers() {
        let mut manager = RegisterManager::new();

        manager.set_unnamed("first", false);
        manager.set_unnamed("second", false);
        manager.set_unnamed("third", false);

        // 0号应该是最新的
        assert_eq!(manager.get('0').unwrap().content, "third");
        // 1号应该是上一个
        assert_eq!(manager.get('1').unwrap().content, "second");
        // 2号应该是第一个
        assert_eq!(manager.get('2').unwrap().content, "first");
    }

    #[test]
    fn test_named_registers() {
        let mut manager = RegisterManager::new();

        manager.set('a', "content a", false);
        assert_eq!(manager.get('a').unwrap().content, "content a");

        // 大写表示追加
        manager.set('A', " appended", false);
        assert_eq!(manager.get('a').unwrap().content, "content a appended");
    }

    #[test]
    fn test_search_register() {
        let mut manager = RegisterManager::new();

        manager.set_search("pattern");
        assert_eq!(manager.get_search(), "pattern");
        assert_eq!(manager.get('/').unwrap().content, "pattern");
    }
}
