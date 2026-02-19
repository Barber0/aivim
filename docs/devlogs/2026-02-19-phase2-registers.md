# AIVim 阶段二开发日志 - 寄存器系统与Bug修复

**日期**: 2026-02-19  
**阶段**: 阶段二 - 基础编辑功能  
**主题**: 寄存器系统实现与关键Bug修复

---

## 1. 功能实现

### 1.1 寄存器系统 (Register System)

**设计目标**: 实现Vim兼容的寄存器系统，支持复制粘贴操作

**实现文件**: `crates/aivim-core/src/register.rs`

#### 支持的寄存器类型

| 寄存器 | 名称 | 用途 |
|--------|------|------|
| `"` | 无名寄存器 | 默认寄存器，删除/复制自动使用 |
| `0-9` | 数字寄存器 | 存储删除历史（0最新，9最旧） |
| `a-z` | 命名寄存器 | 用户指定存储 |
| `A-Z` | 命名寄存器（追加） | 追加到对应小写寄存器 |
| `-` | 小删除寄存器 | 存储小于一行的删除 |
| `/` | 搜索寄存器 | 存储搜索模式 |
| `%`, `#`, `:`, `.` | 只读寄存器 | 当前文件、备用文件、上次命令、上次插入 |

#### 核心数据结构

```rust
pub struct Register {
    pub name: char,
    pub content: String,
    pub linewise: bool,  // true=整行操作, false=字符操作
}

pub struct RegisterManager {
    unnamed: Register,
    small_delete: Register,
    numbered: Vec<Register>,  // 0-9
    named: HashMap<char, Register>,  // a-z
    readonly: HashMap<char, Register>,
    search: Register,
}
```

#### 关键方法

```rust
// 设置无名寄存器（自动更新数字寄存器）
pub fn set_unnamed(&mut self, content: impl Into<String>, linewise: bool)

// 设置命名寄存器（大写表示追加）
pub fn set(&mut self, name: char, content: impl Into<String>, linewise: bool)

// 获取寄存器内容
pub fn get(&self, name: char) -> Option<&Register>
```

#### Editor集成

```rust
// 复制当前行 (yy)
pub fn yank_line(&mut self, register: Option<char>)

// 粘贴 (p/P)
pub fn paste(&mut self, register: Option<char>, before_cursor: bool)

// 删除行 (dd)
pub fn delete_line(&mut self, register: Option<char>) -> Option<String>

// 删除字符到寄存器 (x)
pub fn delete_char_to_register(&mut self, register: Option<char>) -> Option<char>
```

---

## 2. 关键Bug修复

### 2.1 Bug: `a` (append) 命令在行尾不工作

**现象**: 当光标在行尾时，按 `a` 进入插入模式后，输入的字符插入到了倒数第二个字符之前，而不是最后一个字符之后。

**根因分析**:

Vim的光标模型：
- **Normal模式**: 光标在**字符上**（位置范围: 0 到 line_len-1）
- **Insert模式**: 光标在**字符之间**（位置范围: 0 到 line_len）

当使用 `a` 命令时：
- 如果光标在位置0（'H'上），应该在 'H' **之后**插入
- 如果光标在位置4（'o'上，最后一个字符），应该在 'o' **之后**插入

但我们的 `insert_char` 是在当前光标位置插入，所以当光标在行尾时，插入位置是 `line_start + line_len - 1`，也就是在最后一个字符**之前**。

**修复方案**:

1. **添加标志位**: 在 Editor 中添加 `insert_after_cursor: bool` 标志
2. **修改 `enter_append_mode`**: 检测光标是否在行尾，如果是则设置标志
3. **修改 `insert_char`**: 检查标志，如果设置则调用 `insert_char_after_cursor`
4. **实现 `insert_char_after_cursor`**: 在光标位置**之后**插入字符

```rust
pub struct Editor {
    // ... 其他字段
    insert_after_cursor: bool,  // 新增
}

pub fn enter_append_mode(&mut self) {
    let buffer = self.current_buffer();
    let line_len = buffer.line_len(self.cursor.line);
    let max_col = line_len.saturating_sub(1);
    let at_end = self.cursor.column >= max_col;

    if at_end {
        // 在行尾：设置标志，让 insert_char 在光标后插入
        self.insert_after_cursor = true;
        self.set_mode(Mode::Insert);
    } else {
        // 不在行尾：向右移动一位
        drop(buffer);
        self.execute_motion(Motion::Right);
        self.set_mode(Mode::Insert);
    }
}

pub fn insert_char(&mut self, ch: char) {
    if self.mode.is_insert() {
        // 检查是否需要在光标后插入
        if self.insert_after_cursor {
            self.insert_char_after_cursor(ch);
            self.insert_after_cursor = false;
            return;
        }
        // ... 原有逻辑
    }
}

fn insert_char_after_cursor(&mut self, ch: char) {
    let cursor_line = self.cursor.line;
    let cursor_col = self.cursor.column;

    // 计算插入位置：在光标位置之后
    let char_idx = {
        let buffer = self.current_buffer();
        let line_start = buffer.line_to_char(cursor_line);
        line_start + cursor_col + 1  // +1 表示在光标后
    };

    let buffer = self.current_buffer_mut();
    let line_len = buffer.line_len(cursor_line);
    let insert_idx = char_idx.min(line_len);  // 确保不越界

    buffer.insert_char(insert_idx, ch);
    // ... 更新光标位置
}
```

**测试结果**:

| 场景 | 输入 | 预期 | 结果 |
|------|------|------|------|
| 行首 | `a!` | "H!ello" | ✅ |
| 行尾 | `a!` | "Hello!" | ✅ |
| 中间 | `a!` | "Hel!lo" | ✅ |

---

## 3. 技术难点与解决方案

### 3.1 Rust借用检查器挑战

**问题**: 在 `paste` 方法中，需要同时访问 `register_manager` 和 `current_buffer_mut`

```rust
// 错误代码
if let Some(register) = self.register_manager.get(reg) {
    // 不可变借用 register_manager
    let buffer = self.current_buffer_mut();  // 可变借用 self - 冲突！
    buffer.insert(char_idx, &register.content);  // 使用 register - 冲突！
}
```

**解决方案**: 提前克隆需要的数据

```rust
// 正确代码
let (content, linewise) = {
    if let Some(register) = self.register_manager.get(reg) {
        (register.content.clone(), register.linewise)  // 提前克隆
    } else {
        return;
    }
};
// 现在可以安全地可变借用 self
let buffer = self.current_buffer_mut();
buffer.insert(char_idx, &content);
```

**设计模式**: "先收集信息，再执行操作"

### 3.2 光标位置管理

**复杂性**: Normal模式和Insert模式的光标语义不同

| 模式 | 光标位置 | 插入行为 |
|------|----------|----------|
| Normal | 在字符上 | - |
| Insert | 在字符之间 | 在当前位置插入 |
| Append | 在字符之间 | 在光标后插入 |

**解决方案**: 
- 使用标志位区分 `i` 和 `a` 的插入行为
- 在模式切换时正确处理光标位置

---

## 4. 代码结构改进

### 4.1 新增文件

```
crates/aivim-core/src/
├── register.rs          # 新增：寄存器系统
├── lib.rs               # 修改：导出 register 模块
├── editor.rs            # 修改：集成寄存器操作
```

### 4.2 新增方法

**Editor**:
- `register_manager()` / `register_manager_mut()` - 访问寄存器管理器
- `yank_line()` - 复制行
- `yank()` - 复制区域
- `paste()` - 粘贴
- `delete_line()` - 删除行到寄存器
- `delete_char_to_register()` - 删除字符到寄存器
- `enter_append_mode()` - 处理 `a` 命令
- `insert_char_after_cursor()` - 在光标后插入

### 4.3 UI层绑定

```rust
// crates/aivim-tui/src/app.rs
KeyCode::Char('a') => {
    self.editor.enter_append_mode();  // 使用新方法
}
KeyCode::Char('y') => {
    self.editor.yank_line(None);  // yy
}
KeyCode::Char('d') => {
    self.editor.delete_line(None);  // dd
}
KeyCode::Char('p') => {
    self.editor.paste(None, false);  // p
}
KeyCode::Char('P') => {
    self.editor.paste(None, true);  // P
}
KeyCode::Char('x') => {
    self.editor.delete_char_to_register(None);  // x
}
```

---

## 5. 测试策略

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_unnamed_register() {
        let mut manager = RegisterManager::new();
        manager.set_unnamed("hello", false);
        assert_eq!(manager.get('"').unwrap().content, "hello");
        assert_eq!(manager.get('0').unwrap().content, "hello");
    }

    #[test]
    fn test_named_registers() {
        let mut manager = RegisterManager::new();
        manager.set('a', "content", false);
        manager.set('A', " appended", false);  // 追加
        assert_eq!(manager.get('a').unwrap().content, "content appended");
    }
}
```

### 5.2 集成测试

```rust
// 测试 a 命令的各种场景
fn test_append_command() {
    // 场景1: 行首
    editor.execute_motion(Motion::Right);  // 到 'e'
    editor.enter_append_mode();
    editor.insert_char('!');
    assert_eq!(content, "H!ello");

    // 场景2: 行尾
    // ... 移动到行尾
    editor.enter_append_mode();
    editor.insert_char('!');
    assert_eq!(content, "Hello!");
}
```

---

## 6. 已知问题与TODO

### 6.1 当前限制

1. **数字寄存器行为**: 与Vim略有不同，需要进一步调整
2. **系统剪贴板**: 尚未实现（需要引入 `arboard` crate）
3. **Visual模式**: 尚未实现，无法选择区域复制

### 6.2 下一步计划

- [ ] 实现搜索功能 (`/`, `?`, `n`, `N`)
- [ ] 实现基础替换 (`:s`)
- [ ] 实现更多光标移动命令
- [ ] 实现文本对象 (`aw`, `iw`)

---

## 7. 开发心得

### 7.1 关于光标模型

Vim的光标模型比想象中复杂：
- Normal模式：光标在字符上
- Insert模式：光标在字符之间
- 模式切换时需要考虑光标位置的语义变化

**教训**: 在设计编辑器时，需要明确定义每种模式下光标的语义。

### 7.2 关于Rust借用检查器

借用检查器强制我们写出更清晰的代码：
- 不能同时持有可变和不可变引用
- 这促使我们将代码拆分为"收集信息"和"执行操作"两个阶段
- 最终代码更清晰、更易于理解

### 7.3 关于测试

对于编辑器这种交互复杂的软件，测试非常重要：
- 单元测试验证核心逻辑
- 集成测试验证用户场景
- 边界情况（行首、行尾、空行）需要特别测试

---

## 8. 附录：关键代码片段

### 8.1 寄存器管理器

```rust
impl RegisterManager {
    pub fn set_unnamed(&mut self, content: impl Into<String>, linewise: bool) {
        let content = content.into();
        
        // 将0-8的内容移到1-9
        for i in (1..=8).rev() {
            self.numbered[i + 1] = self.numbered[i].clone();
        }
        
        // 原来的无名寄存器内容移到1号
        self.numbered[1] = self.unnamed.clone();
        
        // 设置新的无名寄存器
        self.unnamed = Register::new('"', content.clone(), linewise);
        
        // 新内容也放入0号寄存器
        self.numbered[0] = Register::new('0', content, linewise);
    }
}
```

### 8.2 粘贴操作

```rust
pub fn paste(&mut self, register: Option<char>, before_cursor: bool) {
    let reg = register.unwrap_or('"');
    
    // 提前克隆寄存器内容，避免借用冲突
    let (content, linewise) = {
        if let Some(register) = self.register_manager.get(reg) {
            if register.is_empty() { return; }
            (register.content.clone(), register.linewise)
        } else { return; }
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
        self.cursor.line = insert_line;
        self.cursor.column = 0;
    } else {
        // 字符粘贴
        // ...
    }
}
```

---

**总结**: 阶段二的基础功能（寄存器系统）已经实现完成，修复了关键的 `a` 命令Bug，为后续功能开发奠定了坚实基础。
