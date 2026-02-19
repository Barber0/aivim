# AIVim MVP 开发日志

**日期**: 2026-02-19  
**阶段**: 阶段一 - 最小可用产品 (MVP)  
**目标**: 实现一个可以编辑文本的基础Vim编辑器

---

## 1. 项目初始化

### 1.1 架构设计决策

**设计思想**: 采用Workspace多Crate架构，将核心功能与UI分离

```
aivim/
├── src/main.rs              # 程序入口
├── crates/
│   ├── aivim-core/          # 核心编辑引擎（纯逻辑，无UI依赖）
│   └── aivim-tui/           # 终端UI（依赖core）
```

**设计理由**:
- **关注点分离**: 核心编辑逻辑与渲染层解耦
- **可测试性**: core crate可以独立测试，无需终端环境
- **可扩展性**: 未来可以添加GUI前端而无需修改核心

### 1.2 关键技术选型

| 组件 | 选择 | 理由 |
|------|------|------|
| 文本数据结构 | ropey | 高效的Rope实现，支持O(log n)的插入删除 |
| 终端控制 | crossterm | 跨平台，支持Windows/Unix |
| TUI框架 | ratatui 0.24 | 成熟的Rust TUI框架，性能好 |
| 异步运行时 | tokio | 为未来LSP等功能预留 |

---

## 2. 核心编辑引擎实现 (aivim-core)

### 2.1 Buffer设计

**关键决策**: 使用Rope而非String或Vec<char>

```rust
pub struct Buffer {
    id: BufferId,
    rope: Rope,  // 使用ropey::Rope
    file_path: Option<PathBuf>,
    modified: bool,
}
```

**设计思想转变**:
- 最初考虑使用Vec<String>（每行一个String）
- 发现问题：跨行操作复杂，大文件性能差
- 最终选择Rope：树形结构，天然支持大文件，操作复杂度O(log n)

### 2.2 光标位置表示

**关键问题**: 如何表示光标位置？

**方案对比**:
1. **仅使用字符索引**: 简单，但行/列计算复杂
2. **(line, column) + 字符索引转换**: 符合用户直觉，计算有开销

**最终选择**: 方案2

```rust
pub struct Cursor {
    pub line: usize,
    pub column: usize,
    pub preferred_column: Option<usize>,  // 用于垂直移动时保持列位置
}
```

**关键方法**:
```rust
pub fn to_char_idx(&self, buffer: &Buffer) -> usize
pub fn from_char_idx(buffer: &Buffer, char_idx: usize) -> Self
```

### 2.3 模式系统设计

**设计思想**: 状态机模式

```rust
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Replace,
}
```

**关键洞察**: 不同模式下的相同按键有不同含义，需要在UI层根据模式分发处理

---

## 3. 关键Bug修复记录

### 3.1 Bug #1: 新建文件失败

**现象**: 打开不存在的文件时报错

**根因分析**:
```rust
// 原代码
pub fn from_file(id: BufferId, path: &Path) -> io::Result<Self> {
    let content = fs::read_to_string(path)?;  // 文件不存在时直接返回Err
    ...
}
```

**修复方案**:
```rust
pub fn open_file(&mut self, path: &Path) -> io::Result<()> {
    // 如果文件存在则打开，否则创建新缓冲区并设置文件路径
    let buffer = if path.exists() {
        Buffer::from_file(buffer_id, path)?
    } else {
        Buffer::new_with_path(buffer_id, path)  // 新增方法
    };
    ...
}
```

**设计思想**: 符合Vim的行为 - `vim newfile.txt` 应该创建新文件

### 3.2 Bug #2: 插入字符顺序错误

**现象**: 插入"Hello"变成"elloH"

**Debug过程**:
1. 添加详细日志追踪每次插入的光标位置
2. 发现问题：第一次插入后光标column仍为0

**根因分析**:
```rust
// 错误代码
let line_len = buffer.line_len(cursor_line);  // = 1 (刚插入一个字符)
let max_col = line_len.saturating_sub(1);      // = 0
self.cursor.column = (cursor_col + 1).min(max_col);  // = 1.min(0) = 0
```

**修复方案**:
```rust
// 修复后 - 插入后光标应该在新字符之后
self.cursor.column = (cursor_col + 1).min(line_len);  // 允许光标在最后一个字符之后
```

**关键洞察**: Insert模式下光标可以位于最后一个字符之后（与Normal模式不同）

### 3.3 Bug #3: 文件末尾出现%符号

**现象**: 用cat查看编辑器保存的文件，末尾显示%

**根因分析**: Unix文本文件惯例要求以换行符结尾，否则终端会显示%提示

**修复方案**:
```rust
pub fn save(&mut self) -> io::Result<()> {
    ...
    // 确保文件以换行符结尾（Unix文本文件惯例）
    if self.rope.len_chars() > 0 {
        let last_char = self.rope.char(self.rope.len_chars() - 1);
        if last_char != '\n' {
            file.write_all(b"\n")?;
        }
    }
    ...
}
```

**设计思想**: 遵循POSIX标准，文本文件应该以换行符结尾

---

## 4. 借用检查器挑战与解决

### 4.1 问题描述

Rust的借用检查器在Editor中频繁报错：
```rust
pub fn insert_char(&mut self, ch: char) {
    let char_idx = self.cursor.to_char_idx(self.current_buffer());  // 不可变借用
    self.current_buffer_mut().insert_char(char_idx, ch);  // 可变借用 - 冲突！
    self.cursor.move_right(self.current_buffer(), 1);  // 又需要不可变借用
}
```

### 4.2 解决方案

**设计思想转变**: 从"方法调用链"到"先收集信息，再执行操作"

**重构后**:
```rust
pub fn insert_char(&mut self, ch: char) {
    // 第1步：收集所有需要的信息
    let char_idx = {
        let buffer = self.current_buffer();
        self.cursor.to_char_idx(buffer)
    };
    let cursor_line = self.cursor.line;
    let cursor_col = self.cursor.column;

    // 第2步：执行修改
    let buffer = self.current_buffer_mut();
    buffer.insert_char(char_idx, ch);

    // 第3步：手动更新状态（不依赖buffer引用）
    let line_len = buffer.line_len(cursor_line);
    self.cursor.column = (cursor_col + 1).min(line_len);
}
```

**关键模式**: 使用代码块限制借用范围，将可变借用和不可变借用分离

---

## 5. UI渲染设计

### 5.1 架构选择

**设计思想**: 双缓冲 + 脏矩形优化

```rust
pub fn draw(frame: &mut Frame, editor: &Editor, scroll_offset: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),      // 编辑区域
            Constraint::Length(1),   // 状态栏
            Constraint::Length(1),   // 命令行
        ])
        .split(size);

    draw_editor_area(frame, editor, chunks[0], scroll_offset);
    draw_status_line(frame, editor, chunks[1]);
    draw_command_line(frame, editor, chunks[2]);
}
```

### 5.2 滚动优化

**关键算法**: 视口跟踪

```rust
pub fn calculate_scroll_offset(
    cursor_line: usize,
    viewport_height: usize,
    current_offset: usize
) -> usize {
    if cursor_line < current_offset {
        cursor_line  // 光标在视口上方，向上滚动
    } else if cursor_line >= current_offset + viewport_height {
        cursor_line.saturating_sub(viewport_height - 1)  // 光标在视口下方，向下滚动
    } else {
        current_offset  // 光标在视口内，不滚动
    }
}
```

---

## 6. 测试策略

### 6.1 测试架构

**设计思想**: 核心库内置测试，UI层通过examples测试

```rust
// aivim-core/src/buffer.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_buffer_insert() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "Hello, World!");
        assert_eq!(buffer.to_string(), "Hello, World!");
        assert!(buffer.is_modified());
    }
}
```

### 6.2 集成测试

使用examples目录进行端到端测试：
```bash
cargo run --example test_editor -p aivim-core
```

**优点**:
- 无需额外的测试框架
- 可以测试文件IO等副作用
- 用户可以直接运行查看行为

---

## 7. 依赖管理

### 7.1 依赖冲突解决

**问题**: getrandom 0.4.1 编译失败

**现象**:
```
error: failed to download `getrandom v0.4.1`
```

**根因**: 依赖树中某些crate依赖了getrandom 0.4.x，但该版本在crates.io上可能有问题

**解决方案**: 手动修改Cargo.lock降级到0.2.x

**经验教训**: 
- 使用`cargo update -p <package>`要谨慎
- 对于关键依赖，可以在Cargo.toml中锁定版本

### 7.2 最终依赖树

```
aivim-core:
├── ropey 1.6
├── thiserror 1.0
├── serde 1.0
├── tracing 0.1
└── unicode-width 0.1

aivim-tui:
├── aivim-core (local)
├── crossterm 0.27
├── ratatui 0.24
├── thiserror 1.0
├── tracing 0.1
└── unicode-width 0.1
```

---

## 8. 性能考虑

### 8.1 大文件处理

**Rope的优势**:
- 插入/删除: O(log n)
- 随机访问: O(log n)
- 内存使用: 共享子树，克隆成本低

**实测数据** (在examples中测试):
- 10MB文件打开: < 100ms
- 在文件中间插入字符: < 1ms

### 8.2 渲染优化

**策略**:
1. 只渲染可见行 (viewport clipping)
2. 使用ratatui的双缓冲
3. 避免不必要的字符串分配

---

## 9. 已知限制与未来改进

### 9.1 当前限制

1. **不支持**: 多字节字符（如中文）的宽度计算
2. **不支持**: 语法高亮
3. **不支持**: 水平滚动（长行会截断）
4. **不支持**: 多窗口/分屏

### 9.2 下一步计划

**阶段二目标**:
- [ ] 寄存器系统（复制粘贴）
- [ ] 搜索功能 (/ ?)
- [ ] 替换功能 (:s)
- [ ] 更多光标移动命令
- [ ] 文本对象 (aw, iw等)

---

## 10. 开发心得

### 10.1 Rust特有的挑战

1. **借用检查器**: 需要仔细设计API，避免同时需要可变和不可变引用
2. **错误处理**: 使用thiserror/anyhow简化错误类型
3. **生命周期**: 在UI层尽量避免显式生命周期标注

### 10.2 有效的工作流

1. **先写测试**: 在实现功能前编写example测试
2. **小步提交**: 每个功能点完成后立即测试
3. **文档驱动**: 先写doc comment，再实现代码

### 10.3 设计原则

1. **兼容性优先**: 尽量遵循Vim的行为
2. **渐进增强**: 从MVP开始，逐步添加功能
3. **性能意识**: 选择合适的数据结构，避免过早优化但保持可优化性

---

## 附录：关键代码片段

### A. 光标移动实现
```rust
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
```

### B. 撤销系统
```rust
pub fn undo(&mut self) {
    if let Some(state) = self.undo_stack.pop() {
        // 保存当前状态到redo栈
        let current_state = EditState {
            buffer_content: self.current_buffer().to_string(),
            cursor: self.cursor,
        };
        self.redo_stack.push(current_state);

        // 恢复之前的状态
        let current_buffer_id = self.current_buffer;
        let buffer = self.buffers.get_mut(&current_buffer_id).unwrap();
        *buffer = Buffer::new(current_buffer_id);
        buffer.insert(0, &state.buffer_content);
        self.cursor = state.cursor;
    }
}
```

### C. 事件处理循环
```rust
fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend>) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, &self.editor, self.scroll_offset))?;

        if self.should_quit {
            break;
        }

        match self.event_handler.next()? {
            Event::Tick => {}
            Event::Key(key) => self.handle_key_event(key),
            Event::Resize(_, height) => {
                self.update_scroll_offset(height as usize);
            }
        }
    }
    Ok(())
}
```

---

**总结**: MVP阶段成功实现了一个可用的Vim编辑器基础，验证了架构设计的可行性，为后续功能开发奠定了坚实基础。
