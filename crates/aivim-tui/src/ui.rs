use aivim_core::Editor;
use crate::app::OperatorState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, editor: &Editor, scroll_offset: usize, operator_state: OperatorState) {
    let size = frame.size();
    
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
    use aivim_core::Mode;

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
    
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(1), Constraint::Length(15)])
        .split(area);

    let mode_widget = Paragraph::new(Line::from(mode_span));
    frame.render_widget(mode_widget, status_chunks[0]);

    let file_widget = Paragraph::new(file_info)
        .alignment(Alignment::Left);
    frame.render_widget(file_widget, status_chunks[1]);

    let pos_widget = Paragraph::new(position)
        .alignment(Alignment::Right);
    frame.render_widget(pos_widget, status_chunks[2]);
}

fn draw_command_line(frame: &mut Frame, editor: &Editor, area: Rect) {
    use aivim_core::Mode;

    let content = match editor.mode() {
        Mode::Command => format!(":{}", editor.command_line()),
        Mode::SearchForward => format!("/{}", editor.command_line()),
        Mode::SearchBackward => format!("?{}", editor.command_line()),
        _ => {
            if let Some(msg) = editor.message() {
                msg.to_string()
            } else {
                String::new()
            }
        }
    };

    let style = match editor.mode() {
        Mode::Command | Mode::SearchForward | Mode::SearchBackward => Style::default(),
        _ => Style::default().fg(Color::Yellow),
    };

    let widget = Paragraph::new(Line::from(Span::styled(content, style)));
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
