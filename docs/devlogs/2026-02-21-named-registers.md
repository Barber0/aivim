# 命名寄存器功能实现日志

**日期**: 2026-02-21  
**作者**: AI Assistant  
**相关提交**: d280692

---

## 背景

在之前的实现中，AIVim 只支持无名寄存器和数字寄存器。用户反馈需要完整的命名寄存器支持（如 `"ayy`, `"ap` 等），以便同时保存多段文本并在需要时选择性使用。

## 目标

实现完整的 Vim 风格命名寄存器支持：
- `"ayy` - 复制当前行到寄存器 a
- `"ap` - 从寄存器 a 粘贴
- `"add` - 删除当前行到寄存器 a
- 支持所有操作符和动作的组合

---

## 实现过程

### 1. 状态机设计

命名寄存器需要两步序列：
1. `"` + `a` - 选择寄存器 a
2. `y` + `y` - 执行操作

因此需要设计一个能够存储中间状态的状态机：

```rust
pub enum OperatorState {
    // ... 其他状态
    RegisterPending(Option<char>), // None=等待寄存器名, Some('a')=已选a,等待操作符
}
```

### 2. 修改 OperatorState

为所有操作符状态添加寄存器参数：

```rust
pub enum OperatorState {
    Delete { register: Option<char> },
    Yank { register: Option<char> },
    // ...
}
```

这样可以在操作执行时知道要使用哪个寄存器。

### 3. 实现 handle_register_selection

这是核心逻辑，处理两步序列：

```rust
fn handle_register_selection(&mut self, key: KeyEvent, pending_reg: Option<char>) {
    match pending_reg {
        None => {
            // 第一步：等待寄存器名
            match key.code {
                KeyCode::Char(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {
                    self.operator_state = OperatorState::RegisterPending(Some(c));
                }
                // ...
            }
        }
        Some(reg) => {
            // 第二步：已选择寄存器，等待操作符
            match key.code {
                KeyCode::Char('d') => {
                    self.operator_state = OperatorState::Delete { register: Some(reg) };
                }
                KeyCode::Char('y') => {
                    self.operator_state = OperatorState::Yank { register: Some(reg) };
                }
                KeyCode::Char('p') => {
                    self.editor.paste(Some(reg), false);
                    self.operator_state = OperatorState::None;
                }
                // ...
            }
        }
    }
}
```

### 4. 添加带寄存器参数的方法

在 Editor 中添加新方法：

```rust
pub fn yank_to_motion_with_register(&mut self, motion: Motion, register: Option<char>)
pub fn delete_to_motion_with_register(&mut self, motion: Motion, register: Option<char>)
```

这些方法在指定了寄存器时，会同时存入命名寄存器和无名寄存器。

### 5. 更新操作符处理器

修改 `handle_operator_motion` 和 `handle_text_object` 来提取和使用寄存器参数：

```rust
let (is_delete, register) = match operator {
    OperatorState::Delete { register } => (true, register),
    OperatorState::Yank { register } => (false, register),
    _ => return,
};

// 执行操作时传入 register
if is_delete {
    self.editor.delete_to_motion_with_register(motion, register);
} else {
    self.editor.yank_to_motion_with_register(motion, register);
}
```

---

## 技术难点

### 难点 1: 状态机的复杂性

**问题**: 需要处理多种状态组合（操作符 + 寄存器 + 文本对象）

**解决**: 使用结构化的状态定义，将寄存器作为参数传递给所有操作符状态：

```rust
TextObject { 
    operator: TextObjectOperator, 
    around: bool, 
    register: Option<char>  // 支持 "adaw
}
```

### 难点 2: 向后兼容

**问题**: 原有代码使用 `OperatorState::Delete` 等无参数版本

**解决**: 将所有状态改为带参数的形式，默认值使用 `None`：

```rust
// 原有代码
self.operator_state = OperatorState::Delete;

// 新代码
self.operator_state = OperatorState::Delete { register: None };
```

### 难点 3: 粘贴操作的特殊性

**问题**: `p` 和 `P` 是立即执行的，不需要等待后续按键

**解决**: 在 `handle_register_selection` 的第二分支中直接处理：

```rust
KeyCode::Char('p') => {
    self.editor.paste(Some(reg), false);  // 立即执行
    self.operator_state = OperatorState::None;
}
```

---

## 测试结果

```
=== Testing named registers ===

1. Testing yank to register a...
   Register a: Some("first line\n")

2. After yy with register b on second line
   Register b: Some("second line\n")

3. Testing paste from register a...
   After "ap:
   Content: first line\nsecond line\nfirst line\nthird line\n

4. Testing delete to register c...
   Register c: Some("first line\n")

5. Testing uppercase register append...
   Register a after 'A': Some("first line\n - appended")

=== Test completed ===
```

所有测试通过！✅

---

## 支持的命令

| 命令 | 功能 |
|------|------|
| `"ayy` | 复制当前行到寄存器 a |
| `"byw` | 复制当前单词到寄存器 b |
| `"add` | 删除当前行到寄存器 a |
| `"bdw` | 删除当前单词到寄存器 b |
| `"ap` | 从寄存器 a 粘贴（光标后） |
| `"aP` | 从寄存器 a 粘贴（光标前） |
| `"daw` | 删除单词+空格到寄存器 a |
| `"yiwx` | 复制单词内部到寄存器 x |

---

## 代码统计

- 修改文件数: 2
- 新增代码行数: ~150
- 删除代码行数: ~105
- 主要修改: `app.rs`, `editor.rs`

---

## 后续优化

1. **可视化反馈**: 在状态栏显示当前选择的寄存器名
2. **寄存器列表**: 添加 `:registers` 命令查看所有寄存器内容
3. **表达式寄存器**: 实现 `"=` 计算表达式功能
4. **系统剪贴板**: 支持 `"*` 和 `"+` 与系统剪贴板交互

---

## 总结

本次实现完整支持了 Vim 风格的命名寄存器功能，用户可以：
- 使用 `"a`-`"z` 保存多段文本
- 使用 `"ap` 选择性粘贴
- 使用大写 `"A` 追加内容

实现采用了清晰的两步状态机设计，保持了代码的可维护性和向后兼容性。

**状态**: ✅ 已完成  
**测试**: ✅ 全部通过  
**文档**: ✅ 已更新教程
