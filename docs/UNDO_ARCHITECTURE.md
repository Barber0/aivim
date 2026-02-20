# AIVim 撤销架构设计

## 问题背景

在 AIVim 的开发过程中，多次出现 "No file path set" 错误，根本原因是：

1. **忘记调用 `save_state()`**：某些修改操作没有保存状态，导致撤销时恢复的状态不完整
2. **文件路径丢失**：`undo/redo` 使用 `Buffer::new()` 重建缓冲区，丢失了文件路径

## 解决方案

### 方案一：使用 `with_save_state!` 宏（推荐）

这是最简单、最可靠的解决方案。通过宏包装修改操作，确保自动保存状态。

#### 宏定义

```rust
#[macro_export]
macro_rules! with_save_state {
    ($self:ident, $body:block) => {{
        // 保存状态
        $self.save_state();
        
        // 执行操作
        let result = $body;
        
        result
    }};
}
```

#### 使用示例

```rust
impl Editor {
    // ✅ 好的例子：使用宏包装修改操作
    pub fn delete_line_fixed(&mut self) {
        with_save_state!(self, {
            let line_idx = self.cursor.line;
            let line_text = self.get_line_text(line_idx).unwrap_or_default();
            
            // 保存到寄存器
            self.register_manager.set_unnamed(&line_text, true);
            
            // 删除行
            let start = self.current_buffer().line_to_char(line_idx);
            let end = start + line_text.chars().count();
            self.current_buffer_mut().remove(start, end - start);
        });
    }
    
    // ❌ 不好的例子：忘记调用 save_state
    pub fn delete_line_buggy(&mut self) {
        // 忘记 save_state()！撤销时会丢失文件路径
        let line_idx = self.cursor.line;
        // ... 删除行的逻辑
    }
}
```

#### 最佳实践

1. **所有修改操作都使用宏**
   ```rust
   pub fn my_operation(&mut self) {
       with_save_state!(self, {
           // 修改逻辑
       });
   }
   ```

2. **在方法开头使用**
   - 确保在修改前保存状态
   - 避免嵌套使用（防止重复保存）

3. **批量修改只保存一次**
   ```rust
   pub fn complex_operation(&mut self) {
       with_save_state!(self, {
           // 多次修改，但只保存一次初始状态
           self.modify_1();
           self.modify_2();
           self.modify_3();
       });
   }
   ```

### 方案二：使用 `BufferSnapshot` 系统

对于更复杂的场景，可以使用 `BufferSnapshot` 结构体来管理状态。

#### 核心结构

```rust
#[derive(Clone)]
pub struct BufferSnapshot {
    pub content: String,
    pub cursor: Cursor,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
}

impl BufferSnapshot {
    pub fn from_buffer(buffer: &Buffer, cursor: &Cursor) -> Self {
        Self {
            content: buffer.to_string(),
            cursor: *cursor,
            file_path: buffer.file_path().map(|p| p.to_path_buf()),
            modified: buffer.is_modified(),
        }
    }
    
    pub fn apply_to_buffer(&self, buffer: &mut Buffer, cursor: &mut Cursor) {
        // 恢复内容
        *buffer = Buffer::new(buffer.id());
        buffer.insert(0, &self.content);
        
        // 恢复文件路径
        if let Some(ref path) = self.file_path {
            buffer.set_file_path(path.clone());
        }
        
        // 恢复光标
        *cursor = self.cursor;
    }
}
```

#### 使用场景

- 需要自定义撤销逻辑
- 需要保存额外状态
- 实现分支/时间线功能

## 当前修复状态

以下操作已修复，支持正确的撤销/重做：

| 操作 | 状态 | 修复方式 |
|------|------|----------|
| `dw`, `db`, `dl`, `dh` | ✅ | 添加 `save_state()` |
| `yw` | ✅ | 只读操作，无需保存 |
| `p`, `P` | ✅ | 使用现有 `save_state()` |
| `:s`, `:%s` | ✅ | 添加 `save_state()` |
| `dd`, `yy` | ✅ | 使用现有 `save_state()` |
| `x` | ✅ | 使用现有 `save_state()` |
| `i`, `a`, `o` | ✅ | 使用现有 `save_state()` |

## DIY 命令指南

如果你想添加新的自定义命令，请遵循以下模式：

### 1. 简单修改命令

```rust
pub fn my_simple_edit(&mut self) {
    with_save_state!(self, {
        // 你的修改逻辑
        let buffer = self.current_buffer_mut();
        buffer.insert(0, "text");
    });
}
```

### 2. 复杂修改命令

```rust
pub fn my_complex_edit(&mut self) -> Option<String> {
    with_save_state!(self, {
        // 前置检查
        if !self.can_edit() {
            return None;
        }
        
        // 执行修改
        let result = self.do_something();
        
        // 更新状态
        self.set_message("Done");
        
        result
    })
}
```

### 3. 命令模式下的修改

```rust
pub fn execute_command(&mut self, command: &str) -> Result<(), String> {
    match command {
        "mycommand" => {
            with_save_state!(self, {
                // 执行修改
                self.my_operation();
            });
        }
        _ => {}
    }
    Ok(())
}
```

## 测试检查清单

添加新命令后，请测试：

- [ ] 命令正常执行
- [ ] 按 `u` 可以撤销
- [ ] 撤销后 `:wq` 可以保存（不报错 "No file path set"）
- [ ] 按 `Ctrl+R` 可以重做
- [ ] 多次撤销/重做后文件路径仍然保留

## 架构改进建议

### 短期（已完成）

- ✅ 添加 `with_save_state!` 宏
- ✅ 修复所有已知操作
- ✅ 添加文档

### 中期（可选）

- [ ] 使用 `BufferSnapshot` 重构 `undo/redo`
- [ ] 添加撤销历史限制（防止内存无限增长）
- [ ] 支持分支撤销（undo tree）

### 长期（可选）

- [ ] 实现事务系统（Transaction）
- [ ] 支持宏录制和回放
- [ ] 持久化撤销历史

## 总结

**核心原则**：任何修改缓冲区的操作都必须保存状态。

**推荐做法**：使用 `with_save_state!` 宏包装修改操作。

**验证方法**：测试撤销后是否能正常保存文件。

---

*最后更新：2026-02-20*
