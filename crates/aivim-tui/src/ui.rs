use aivim_core::Editor;
use crate::app::OperatorState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, editor: &Editor, scroll_offset: usize, operator_state: OperatorState, show_registers: bool) {
    let size = frame.size();
    
    if show_registers {
        // 显示寄存器面板时，使用弹出窗口布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(size);

        draw_editor_area(frame, editor, chunks[0], scroll_offset);
        draw_status_line(frame, editor, chunks[1], operator_state);
        draw_command_line(frame, editor, chunks[2]);
        
        // 在编辑器区域上方绘制寄存器面板
        draw_registers_panel(frame, editor, chunks[0]);
    } else {
        // 正常布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(size);

        draw_editor_area(frame, editor, chunks[0], scroll_offset);
        draw_status_line(frame, editor, chunks[1], operator_state);
        draw_command_line(frame, editor, chunks[2]);
    }
}

fn draw_editor_area(
    frame: &mut Frame,
    editor: &Editor,
    area: Rect,
    scroll_offset: usize,
) {
    let visible_height = area.height as usize;
    let visible_lines = editor.visible_lines(visible_height, scroll_offset);
    let cursor = editor.cursor();

    let mut text_lines: Vec<Line> = visible_lines
        .into_iter()
        .map(|(line_idx, content)| {
            let is_current_line = line_idx == cursor.line;
            let style = if is_current_line {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            Line::from(Span::styled(content, style))
        })
        .collect();

    if text_lines.is_empty() {
        text_lines.push(Line::from(""));
    }

    let editor_widget = Paragraph::new(Text::from(text_lines))
        .block(Block::default())
        .wrap(Wrap { trim: false });

    frame.render_widget(editor_widget, area);

    let cursor_x = area.x + cursor.column as u16;
    let cursor_y = area.y + (cursor.line - scroll_offset) as u16;
    
    if cursor_y < area.y + area.height && cursor_y >= area.y {
        frame.set_cursor(cursor_x, cursor_y);
    }
}

fn draw_status_line(frame: &mut Frame, editor: &Editor, area: Rect, operator_state: OperatorState) {

    let buffer = editor.current_buffer();
    let mode = editor.mode();

    let file_name = buffer
        .file_path()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("[No Name]");

    let modified_indicator = if buffer.is_modified() { " [+]" } else { "" };

    let cursor = editor.cursor();
    let position = format!("{}:{} ", cursor.line + 1, cursor.column + 1);

    // 获取当前选择的寄存器信息
    let register_info = get_register_info(operator_state);

    // 如果有操作符等待状态，显示在模式后面
    let mode_name = if operator_state != OperatorState::None {
        format!("{}-OPERATOR", mode.name())
    } else {
        mode.name().to_string()
    };

    let mode_style = Style::default()
        .fg(Color::Black)
        .bg(mode_color(mode))
        .add_modifier(Modifier::BOLD);

    let mode_span = Span::styled(format!(" {} ", mode_name), mode_style);

    let file_info = format!("{}{}", file_name, modified_indicator);
    
    // 如果有寄存器信息，调整布局
    let status_chunks = if register_info.is_empty() {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10), Constraint::Min(1), Constraint::Length(15)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10), Constraint::Length(5), Constraint::Min(1), Constraint::Length(15)])
            .split(area)
    };

    let mode_widget = Paragraph::new(Line::from(mode_span));
    frame.render_widget(mode_widget, status_chunks[0]);

    // 如果有寄存器信息，显示在第二列
    if !register_info.is_empty() {
        let reg_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let reg_span = Span::styled(register_info, reg_style);
        let reg_widget = Paragraph::new(Line::from(reg_span));
        frame.render_widget(reg_widget, status_chunks[1]);

        let file_widget = Paragraph::new(file_info)
            .alignment(Alignment::Left);
        frame.render_widget(file_widget, status_chunks[2]);

        let pos_widget = Paragraph::new(position)
            .alignment(Alignment::Right);
        frame.render_widget(pos_widget, status_chunks[3]);
    } else {
        let file_widget = Paragraph::new(file_info)
            .alignment(Alignment::Left);
        frame.render_widget(file_widget, status_chunks[1]);

        let pos_widget = Paragraph::new(position)
            .alignment(Alignment::Right);
        frame.render_widget(pos_widget, status_chunks[2]);
    }
}

/// 根据操作符状态获取寄存器信息显示
fn get_register_info(operator_state: OperatorState) -> String {
    use crate::app::OperatorState;
    
    match operator_state {
        // 正在等待寄存器名（刚按下 "）
        OperatorState::RegisterPending(None) => "\"?".to_string(),
        // 已选择寄存器，等待操作符
        OperatorState::RegisterPending(Some(reg)) => format!("\"{}", reg),
        // 删除操作符已指定寄存器
        OperatorState::Delete { register: Some(reg) } => format!("\"{}", reg),
        // 复制操作符已指定寄存器
        OperatorState::Yank { register: Some(reg) } => format!("\"{}", reg),
        // 修改操作符已指定寄存器
        OperatorState::Change { register: Some(reg) } => format!("\"{}", reg),
        // 文本对象操作符已指定寄存器
        OperatorState::TextObject { register: Some(reg), .. } => format!("\"{}", reg),
        // 其他情况不显示寄存器信息
        _ => String::new(),
    }
}

fn draw_command_line(frame: &mut Frame, editor: &Editor, area: Rect) {
    use aivim_core::Mode;

    let (text, style) = match editor.mode() {
        Mode::Command => (format!(":{}", editor.command_line()), Style::default()),
        Mode::SearchForward => (format!("/{}", editor.command_line()), Style::default()),
        Mode::SearchBackward => (format!("?{}", editor.command_line()), Style::default()),
        _ => {
            if let Some(msg) = editor.message() {
                (msg.to_string(), Style::default().fg(Color::Yellow))
            } else {
                (String::new(), Style::default().fg(Color::Yellow))
            }
        }
    };

    // 支持多行消息：将文本按行分割，每行创建一个 Line
    let lines: Vec<Line> = text
        .lines()
        .map(|line| Line::from(Span::styled(line.to_string(), style)))
        .collect();

    let widget = Paragraph::new(lines);
    frame.render_widget(widget, area);
}

fn mode_color(mode: aivim_core::Mode) -> Color {
    match mode {
        aivim_core::Mode::Normal => Color::Blue,
        aivim_core::Mode::Insert => Color::Green,
        aivim_core::Mode::Visual => Color::Yellow,
        aivim_core::Mode::Command => Color::Magenta,
        aivim_core::Mode::Replace => Color::Red,
        aivim_core::Mode::SearchForward | aivim_core::Mode::SearchBackward => Color::Cyan,
    }
}

pub fn calculate_scroll_offset(cursor_line: usize, viewport_height: usize, current_offset: usize) -> usize {
    if cursor_line < current_offset {
        cursor_line
    } else if cursor_line >= current_offset + viewport_height {
        cursor_line.saturating_sub(viewport_height - 1)
    } else {
        current_offset
    }
}

/// 绘制寄存器内容面板
fn draw_registers_panel(frame: &mut Frame, editor: &Editor, editor_area: Rect) {
    // 计算面板大小（占据编辑器区域的 80%）
    let panel_width = (editor_area.width as f32 * 0.8) as u16;
    let panel_height = (editor_area.height as f32 * 0.8) as u16;
    
    let panel_x = editor_area.x + (editor_area.width - panel_width) / 2;
    let panel_y = editor_area.y + (editor_area.height - panel_height) / 2;
    
    let panel_area = Rect::new(panel_x, panel_y, panel_width, panel_height);
    
    // 先清除背景
    frame.render_widget(Clear, panel_area);
    
    // 获取寄存器内容
    let registers_text = editor.format_registers();
    
    // 将文本分割成行
    let lines: Vec<Line> = registers_text
        .lines()
        .map(|line| {
            // 高亮寄存器名（如 "a）
            if line.starts_with('"') && line.len() > 2 {
                let reg_name = &line[..2]; // "a
                let rest = &line[2..];
                Line::from(vec![
                    Span::styled(reg_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(rest, Style::default()),
                ])
            } else if line.starts_with("Registers:") || line.starts_with("---") {
                // 标题和分隔线使用不同颜色
                Line::from(Span::styled(line, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            } else {
                Line::from(Span::styled(line, Style::default()))
            }
        })
        .collect();
    
    // 创建带边框的面板
    let panel = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .title(" Registers (press q or Esc to close) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
        )
        .wrap(Wrap { trim: false });
    
    frame.render_widget(panel, panel_area);
}
