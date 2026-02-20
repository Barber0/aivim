pub mod buffer;
pub mod buffer_snapshot;
pub mod cursor;
pub mod editor;
pub mod macros;
pub mod mode;
pub mod motion;
pub mod edit;
pub mod register;
pub mod search;
pub mod replace;
pub mod text_object;

pub use buffer::Buffer;
pub use buffer_snapshot::{BufferSnapshot, SnapshotManager};
pub use cursor::Cursor;
pub use editor::Editor;
pub use mode::Mode;
pub use register::{Register, RegisterManager};
pub use search::{SearchState, SearchDirection};
pub use replace::{ReplaceResult, replace_in_buffer, parse_substitute_command};
pub use text_object::{TextObject, parse_text_object};

// 重新导出宏
pub use macros::SaveStateDocumentation;
