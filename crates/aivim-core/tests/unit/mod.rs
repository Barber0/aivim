//! 单元测试模块
//!
//! 测试文件与源文件的对应关系：
//! - motion_test.rs -> src/motion.rs
//! - buffer_test.rs -> src/buffer.rs
//! - cursor_test.rs -> src/cursor.rs
//! - register_test.rs -> src/register.rs
//! - search_test.rs -> src/search.rs
//! - text_object_test.rs -> src/text_object.rs
//! - replace_test.rs -> src/replace.rs
//! - buffer_snapshot_test.rs -> src/buffer_snapshot.rs
//! - registers_command_test.rs -> src/editor.rs (registers command)
//! - clipboard_test.rs -> src/register.rs (clipboard integration)
//! - edit_test.rs -> src/edit.rs
//! - editor_delete_test.rs -> src/editor.rs (delete operations)
//! - buffer_manager_test.rs -> src/editor.rs (buffer management)
//! - line_number_test.rs -> src/editor.rs (line number options)
//! - chinese_test.rs -> src/buffer.rs (chinese text support)
//! - display_width_test.rs -> src/buffer.rs (display width calculation)

pub mod motion_test;
pub mod buffer_test;
pub mod cursor_test;
pub mod register_test;
pub mod search_test;
pub mod text_object_test;
pub mod replace_test;
pub mod buffer_snapshot_test;
pub mod registers_command_test;
pub mod clipboard_test;
pub mod edit_test;
pub mod editor_delete_test;
pub mod buffer_manager_test;
pub mod line_number_test;
pub mod chinese_test;
pub mod display_width_test;
