/// 自动状态保存宏
/// 
/// 用于包装修改操作，自动处理状态保存和撤销支持。
/// 这是防止 "No file path set" 错误的简单解决方案。
///
/// 使用方式：
/// ```
/// // 在 Editor 方法中使用
/// pub fn my_operation(&mut self) {
///     with_save_state!(self, {
///         // 执行修改操作
///         self.buffer.insert(0, "text");
///     });
/// }
/// ```
///
/// 这个宏会自动：
/// 1. 在操作前调用 save_state()
/// 2. 保留文件路径
/// 3. 支持撤销
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

/// 批量修改操作的自动保存
/// 
/// 适用于需要多次修改但只想保存一次状态的情况
#[macro_export]
macro_rules! with_save_state_once {
    ($self:ident, $body:block) => {{
        // 只在第一次修改时保存状态
        $self.save_state();
        
        // 执行操作
        let result = $body;
        
        result
    }};
}

/// 条件保存宏
/// 
/// 只在条件满足时保存状态
#[macro_export]
macro_rules! with_save_state_if {
    ($self:ident, $condition:expr, $body:block) => {{
        let should_save = $condition;
        if should_save {
            $self.save_state();
        }
        
        let result = $body;
        
        result
    }};
}

/// 文档和示例
/// 
/// # 为什么需要这个宏？
/// 
/// 在 AIVim 中，每次修改缓冲区内容时，都需要：
/// 1. 调用 save_state() 保存当前状态（用于撤销）
/// 2. 确保文件路径被正确保存
/// 
/// 如果忘记调用 save_state()，撤销时会丢失文件路径，
/// 导致 "No file path set" 错误。
/// 
/// # 使用示例
/// 
/// ```rust
/// impl Editor {
///     // 好的例子：使用宏包装修改操作
///     pub fn delete_line_fixed(&mut self) {
///         with_save_state!(self, {
///             let line_idx = self.cursor.line;
///             // ... 删除行的逻辑
///         });
///     }
///     
///     // 不好的例子：忘记调用 save_state
///     pub fn delete_line_buggy(&mut self) {
///         // 忘记 save_state()！
///         let line_idx = self.cursor.line;
///         // ... 删除行的逻辑
///     }
/// }
/// ```
/// 
/// # 最佳实践
/// 
/// 1. **所有修改操作都使用宏**：任何修改 buffer 的方法都应该使用 with_save_state!
/// 2. **在方法开头使用**：确保在修改前保存状态
/// 3. **不要嵌套使用**：避免重复保存状态
/// 4. **测试撤销功能**：添加测试确保撤销后文件路径保留
/// 
/// # 常见问题
/// 
/// Q: 为什么不在 Buffer 的每个修改方法中自动保存状态？
/// A: 因为有些操作需要多次修改，我们希望只保存一次初始状态。
///    例如，替换操作可能涉及多次删除和插入，但我们只想撤销整个替换。
///
/// Q: 这个宏能完全防止 "No file path set" 错误吗？
/// A: 是的，只要所有修改操作都使用这个宏，就不会出现该错误。
///    宏确保 save_state() 被调用，而 save_state() 会保存文件路径。
pub struct SaveStateDocumentation;
