/// 替换功能模块
///
/// 实现Vim风格的替换命令：
/// - :s/old/new - 替换当前行第一个匹配
/// - :s/old/new/g - 替换当前行所有匹配
/// - :%s/old/new/g - 替换整个文件所有匹配

use crate::buffer::Buffer;

#[derive(Debug, Clone)]
pub struct ReplaceResult {
    /// 替换次数
    pub count: usize,
    /// 替换后的文本
    pub new_text: String,
}

/// 执行替换操作
///
/// # 参数
/// - `buffer`: 缓冲区
/// - `pattern`: 要替换的模式
/// - `replacement`: 替换内容
/// - `global`: 是否替换所有匹配（g标志）
/// - `line_range`: 行范围（None表示当前行，Some((start, end))表示范围）
pub fn replace_in_buffer(
    buffer: &mut Buffer,
    pattern: &str,
    replacement: &str,
    global: bool,
    line_range: Option<(usize, usize)>,
) -> ReplaceResult {
    if pattern.is_empty() {
        return ReplaceResult {
            count: 0,
            new_text: buffer.to_string(),
        };
    }

    let text = buffer.to_string();
    let lines: Vec<&str> = text.lines().collect();
    
    // 确定替换范围
    let (start_line, end_line) = match line_range {
        Some((start, end)) => (start, end),
        None => (0, lines.len()), // None 表示整个文件
    };

    let mut new_lines = Vec::new();
    let mut total_replacements = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let new_line = if line_idx >= start_line && line_idx < end_line {
            if global {
                // 替换所有匹配
                let count = line.matches(pattern).count();
                total_replacements += count;
                line.replace(pattern, replacement)
            } else {
                // 只替换第一个匹配
                if let Some(pos) = line.find(pattern) {
                    total_replacements += 1;
                    let mut result = String::new();
                    result.push_str(&line[..pos]);
                    result.push_str(replacement);
                    result.push_str(&line[pos + pattern.len()..]);
                    result
                } else {
                    line.to_string()
                }
            }
        } else {
            line.to_string()
        };
        new_lines.push(new_line);
    }

    let new_text = new_lines.join("\n") + "\n";

    // 更新缓冲区
    let buffer_id = buffer.id();
    *buffer = crate::buffer::Buffer::new(buffer_id);
    buffer.insert(0, &new_text);

    ReplaceResult {
        count: total_replacements,
        new_text,
    }
}

/// 解析替换命令
///
/// 支持的格式：
/// - :s/old/new
/// - :s/old/new/g
/// - :%s/old/new/g
pub fn parse_substitute_command(command: &str) -> Option<(String, String, bool, bool)> {
    // 移除开头的 ':'（如果存在）和 's/'
    let content = command.strip_prefix(':').unwrap_or(command);
    
    // 检查是否是替换命令
    let full_file = content.starts_with("%s/");
    let content = if full_file {
        content.strip_prefix("%s/")?
    } else {
        content.strip_prefix("s/")?
    };

    let global = content.ends_with("/g");
    let content = if global {
        &content[..content.len() - 2]
    } else {
        content
    };

    // 找到分隔符位置
    let parts: Vec<&str> = content.splitn(2, '/').collect();
    if parts.len() != 2 {
        return None;
    }

    let pattern = parts[0].to_string();
    let replacement = parts[1].to_string();

    Some((pattern, replacement, global, full_file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffer, BufferId};

    #[test]
    fn test_replace_first() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "hello world hello\n");

        let result = replace_in_buffer(&mut buffer, "hello", "hi", false, None);

        assert_eq!(result.count, 1);
        assert!(result.new_text.contains("hi world hello"));
    }

    #[test]
    fn test_replace_global() {
        let mut buffer = Buffer::new(BufferId::new(0));
        buffer.insert(0, "hello world hello\n");

        let result = replace_in_buffer(&mut buffer, "hello", "hi", true, None);

        assert_eq!(result.count, 2);
        assert!(result.new_text.contains("hi world hi"));
    }

    #[test]
    fn test_parse_substitute() {
        let result = parse_substitute_command(":s/old/new");
        assert_eq!(result, Some(("old".to_string(), "new".to_string(), false, false)));

        let result = parse_substitute_command(":s/old/new/g");
        assert_eq!(result, Some(("old".to_string(), "new".to_string(), true, false)));

        let result = parse_substitute_command(":%s/old/new/g");
        assert_eq!(result, Some(("old".to_string(), "new".to_string(), true, true)));
    }
}
