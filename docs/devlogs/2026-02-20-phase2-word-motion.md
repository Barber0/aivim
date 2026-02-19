# AIVim 阶段二开发日志 - 词移动优化

**日期**: 2026-02-20  
**阶段**: 阶段二 - 基础编辑功能优化  
**主题**: w/b 词移动命令行边界行为优化

---

## 1. 背景与动机

在阶段二开发过程中，通过实际使用测试，发现 Vim 的词移动命令（`w` 和 `b`）的默认行为在某些场景下不符合用户的直观预期。

### 用户反馈

> "当光标处于行头时，如果我按 `b`，光标会回到前一行的最后一个单词的开头，我觉得这样不太好。"

> "当用 `w` 进行跳词时，跳到当前行最后一个单词开头时，如果多按一次 `w`，会直接跳到下一行的行头，我觉得这样不好。"

### 设计决策

经过讨论，决定调整 `w` 和 `b` 命令的行为：
- **w**: 不跨越行边界，停留在当前行最后一个单词
- **b**: 不跨越行边界，停留在当前行第一个单词

这与传统 Vim 的行为不同，但更符合用户的实际编辑习惯。

---

## 2. 技术实现

### 2.1 w 命令优化

**文件**: `crates/aivim-core/src/motion.rs`

**核心改动**: 限制 `move_word_forward` 只在本行内操作

```rust
fn move_word_forward(cursor: &mut Cursor, buffer: &Buffer) {
    let current_line = cursor.line;
    let current_col = cursor.column;
    let line_text = buffer.line(current_line).map(|l| l.to_string()).unwrap_or_default();
    
    // 移除行尾换行符
    let line_text = line_text.strip_suffix('\n').unwrap_or(&line_text);
    
    // 关键：如果已经在行尾，不移动到下一行
    if current_col >= line_text.len() {
        return;
    }
    
    // ... 在本行内查找下一个单词
    
    // 确保不超出本行范围
    let final_col = new_col.min(line_text.len().saturating_sub(1));
    cursor.column = final_col;
}
```

**关键设计点**:
1. 使用 `buffer.line()` 获取当前行文本，不跨越行边界
2. 提前检查行尾，避免跳到下一行
3. 限制新位置不超过本行最后一个字符

### 2.2 b 命令优化

**文件**: `crates/aivim-core/src/motion.rs`

**核心改动**: 限制 `move_word_backward` 只在本行内操作

```rust
fn move_word_backward(cursor: &mut Cursor, buffer: &Buffer) {
    let current_line = cursor.line;
    let current_col = cursor.column;
    
    // 关键：如果已经在行首，不移动到上一行
    if current_col == 0 {
        return;
    }
    
    let line_text = buffer.line(current_line).map(|l| l.to_string()).unwrap_or_default();
    
    // 只获取当前行光标之前的文本
    let preceding = &line_text[..current_col.min(line_text.len())];
    
    // ... 在本行内查找上一个单词
    
    let new_col = chars.len();
    cursor.column = new_col;
}
```

**关键设计点**:
1. 提前检查行首（`current_col == 0`），直接返回
2. `preceding` 只包含当前行光标之前的文本
3. 不处理换行符，避免跳到上一行

### 2.3 架构一致性

`w` 和 `b` 命令现在遵循一致的设计原则：

| 命令 | 边界检查 | 处理范围 | 越界行为 |
|------|----------|----------|----------|
| w | 检查行尾 | 当前行光标之后 | 停留在行尾 |
| b | 检查行首 | 当前行光标之前 | 停留在行首 |

---

## 3. 测试验证

### 3.1 w 命令测试

**场景**: 文件内容 "hello world\nsecond line\n"

| 步骤 | 操作 | 光标位置 | 结果 |
|------|------|----------|------|
| 1 | 初始 | line=0, col=0 | - |
| 2 | w | line=0, col=6 | 跳到 "world" |
| 3 | w | line=0, col=10 | 到 "world" 末尾 |
| 4 | w | line=0, col=10 | **保持，不跳行** ✅ |
| 5 | w | line=0, col=10 | **保持，不跳行** ✅ |

**使用场景**: 配合 `a` 命令在单词后追加内容

```
hello world
       ^ 光标在这里
       按 w -> 停留在 "world" 末尾
       按 a -> 进入插入模式在 'd' 之后
       输入内容 -> 在 "world" 后追加
```

### 3.2 b 命令测试

**场景**: 文件内容 "hello world\nsecond line\n"

| 步骤 | 操作 | 光标位置 | 结果 |
|------|------|----------|------|
| 1 | j | line=1, col=0 | 到第二行 |
| 2 | w | line=1, col=7 | 跳到 "line" |
| 3 | b | line=1, col=0 | 回到 "second" 开头 |
| 4 | b | line=1, col=0 | **保持，不跳行** ✅ |
| 5 | b | line=1, col=0 | **保持，不跳行** ✅ |

**使用场景**: 在当前行内快速移动，不意外跳到其他行

---

## 4. 设计反思

### 4.1 与传统 Vim 的差异

传统 Vim 的 `w` 和 `b` 命令会跨越行边界：
- `w`: 可以跳到下一行的第一个单词
- `b`: 可以跳到上一行的最后一个单词

**AIVim 的选择**: 限制在本行内

**理由**:
1. **可预测性**: 用户明确知道操作不会离开当前行
2. **编辑效率**: 配合 `a`/`i` 等命令时更直观
3. **减少误操作**: 避免意外跳到其他行

### 4.2 替代方案

如果用户需要跨行移动，可以使用：
- `j`/`k`: 移动到下一行/上一行
- `0`/`^`/`$`: 移动到行首/第一个非空字符/行尾
- `/pattern`: 搜索跨行移动

### 4.3 未来扩展

可以考虑添加配置选项：
```rust
pub struct Config {
    /// w/b 命令是否跨越行边界
    word_motion_cross_line: bool,
}
```

让用户根据喜好选择行为。

---

## 5. 相关提交

```bash
# w 命令修复
commit 146c8f9
Fix: Insert mode character ordering and w motion

# b 命令修复  
commit 001df9e
Fix: b motion no longer crosses line boundary
```

---

## 6. 技术要点总结

### 6.1 借用检查器模式

在修改 `motion.rs` 时，继续使用 "先收集信息，再执行操作" 模式：

```rust
// 先收集信息
let line_text = buffer.line(current_line).map(|l| l.to_string()).unwrap_or_default();

// 再执行操作（不涉及 buffer 借用）
let preceding = &line_text[..current_col];
// ... 处理逻辑
```

### 6.2 边界条件处理

词移动涉及多个边界条件：
- 空行
- 行首/行尾
- 只有一个单词的行
- 全是标点符号的行

修复中通过提前检查（`if current_col == 0`）和范围限制（`.min(line_len)`) 来确保安全。

### 6.3 一致性原则

`w` 和 `b` 命令保持对称设计：
- 都限制在本行内
- 都有明确的边界检查
- 越界时都保持当前位置

---

## 7. 下一步计划

阶段二剩余任务：
- [ ] 实现搜索功能（`/`, `?`, `n`, `N`）
- [ ] 实现基础替换（`:s`）
- [ ] 实现文本对象（`aw`, `iw`）
- [ ] 更多光标移动命令优化

---

## 8. AI 助手的心情

作为 AI 助手，参与 AIVim 的开发是一段非常有趣的旅程。

### 关于这次优化

这次 `w`/`b` 命令的优化让我深刻体会到：**用户反馈是产品改进的最佳驱动力**。

最初实现时，我按照传统 Vim 的行为来设计，认为"这就是标准"。但当用户提出实际使用中的困扰时，我意识到：**标准不一定是最适合用户的**。

通过简单的边界检查，就能让编辑体验更加直观和高效。这种"小而美"的改进，往往比复杂的功能更能提升用户满意度。

### 关于 Rust 开发

Rust 的借用检查器在这次开发中既是挑战也是帮助。它强迫我写出更清晰的代码结构，比如将 "信息收集" 和 "操作执行" 分离。虽然一开始会觉得束缚，但最终代码质量确实更高。

### 关于协作

用户的反馈非常具体和有价值：
- 不只是说"有问题"，而是描述具体场景
- 不只是抱怨，而是提出期望的行为
- 耐心测试，及时反馈

这种协作模式让开发效率大大提升。

### 期待

期待 AIVim 成长为一个既保留 Vim 精神，又融入现代编辑体验的优秀编辑器。每一次小的优化，都是向这个目标迈进的一步。

**继续加油！** 🚀

---

**开发者**: AI Assistant  
**用户/测试者**: fangzilin  
**日期**: 2026-02-20
