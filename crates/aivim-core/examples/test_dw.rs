// 测试 dw 删除
use std::path::Path;
use aivim_core::Editor;
use aivim_core::motion::Motion;

fn main() {
    println!("Testing dw deletion...\n");

    let test_path = Path::new("/tmp/test_dw.txt");
    std::fs::write(test_path, "hello world\n").unwrap();

    let mut editor = Editor::new();
    editor.open_file(test_path).unwrap();

    println!("Initial: {:?}", editor.current_buffer().to_string());
    println!("Cursor: line={}, col={}", editor.cursor().line, editor.cursor().column);

    // 移动到 "world" 开头
    editor.execute_motion(Motion::WordForward);
    println!("\nAfter w: line={}, col={}", editor.cursor().line, editor.cursor().column);

    // 执行 dw
    let result = editor.delete_to_motion(Motion::WordForward);
    println!("After dw: {:?}", editor.current_buffer().to_string());
    println!("Deleted: {:?}", result);
    println!("Cursor: line={}, col={}", editor.cursor().line, editor.cursor().column);

    // 期望: "hello " (world 被删除)
    println!("\nExpected: 'hello \\' (world deleted)");

    // 测试撤销
    println!("\n=== Test undo ===");
    editor.undo();
    println!("After u: {:?}", editor.current_buffer().to_string());

    // 清理
    std::fs::remove_file(test_path).unwrap();

    println!("\n=== Test completed ===");
}
