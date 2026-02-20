use aivim_core::{motion::Motion, Editor, Mode};
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::event::{Event, EventHandler};
use crate::ui::{self, calculate_scroll_offset};

/// 操作符等待状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatorState {
    None,
    Delete,      // d - 等待动作
    Yank,        // y - 等待动作
    Change,      // c - 等待动作 (计划中)
    G,           // g - 等待第二个g (gg)
    TextObject { operator: TextObjectOperator, around: bool }, // a/i - 等待文本对象
    Register(char), // " - 等待寄存器名，char为操作符(d/y/p等)
}

/// 文本对象操作符类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextObjectOperator {
    Delete,  // d
    Yank,    // y
    Change,  // c
}

pub struct App {
    editor: Editor,
    event_handler: EventHandler,
    scroll_offset: usize,
    should_quit: bool,
    operator_state: OperatorState,
}

impl App {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
            event_handler: EventHandler::new(Duration::from_millis(50)),
            scroll_offset: 0,
            should_quit: false,
            operator_state: OperatorState::None,
        }
    }

    pub fn with_file(path: PathBuf) -> io::Result<Self> {
        Ok(Self {
            editor: Editor::with_file(&path)?,
            event_handler: EventHandler::new(Duration::from_millis(50)),
            scroll_offset: 0,
            should_quit: false,
            operator_state: OperatorState::None,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let result = self.run_loop(&mut terminal);
        
        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        
        result
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        loop {
            terminal.draw(|f| ui::draw(f, &self.editor, self.scroll_offset, self.operator_state))?;

            if self.should_quit {
                break;
            }

            match self.event_handler.next()? {
                Event::Tick => {}
                Event::Key(key) => self.handle_key_event(key),
                Event::Resize(_, height) => {
                    self.update_scroll_offset(height as usize);
                }
            }
        }
        
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.editor.mode() {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Command => self.handle_command_mode(key),
            Mode::SearchForward | Mode::SearchBackward => self.handle_search_mode(key),
            _ => {}
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        // 检查是否有操作符等待状态
        match self.operator_state {
            OperatorState::Delete => {
                // 检查是否是文本对象操作（daw, diw）
                match key.code {
                    KeyCode::Char('a') => {
                        self.operator_state = OperatorState::TextObject { 
                            operator: TextObjectOperator::Delete, 
                            around: true 
                        };
                        return;
                    }
                    KeyCode::Char('i') => {
                        self.operator_state = OperatorState::TextObject { 
                            operator: TextObjectOperator::Delete, 
                            around: false 
                        };
                        return;
                    }
                    _ => {
                        self.handle_operator_motion(key, OperatorState::Delete);
                        return;
                    }
                }
            }
            OperatorState::Yank => {
                // 检查是否是文本对象操作（yaw, yiw）
                match key.code {
                    KeyCode::Char('a') => {
                        self.operator_state = OperatorState::TextObject { 
                            operator: TextObjectOperator::Yank, 
                            around: true 
                        };
                        return;
                    }
                    KeyCode::Char('i') => {
                        self.operator_state = OperatorState::TextObject { 
                            operator: TextObjectOperator::Yank, 
                            around: false 
                        };
                        return;
                    }
                    _ => {
                        self.handle_operator_motion(key, OperatorState::Yank);
                        return;
                    }
                }
            }
            OperatorState::TextObject { operator, around } => {
                // 处理文本对象（w, W, s, S, p, P）
                self.handle_text_object(key, operator, around);
                return;
            }
            OperatorState::Register(op) => {
                // 处理寄存器选择
                self.handle_register(key, op);
                return;
            }
            OperatorState::Change => {
                // c - 修改操作符（计划中）
                self.operator_state = OperatorState::None;
            }
            OperatorState::G => {
                // g - 等待第二个 g (gg)
                match key.code {
                    KeyCode::Char('g') => {
                        // gg - 跳到文件开头
                        self.editor.execute_motion(Motion::DocumentStart);
                        // 更新滚动偏移量，确保光标在可视区域内
                        self.update_scroll_offset(terminal_height());
                    }
                    _ => {}
                }
                self.operator_state = OperatorState::None;
                return;
            }
            OperatorState::None => {}
        }

        match key.code {
            KeyCode::Char('i') => {
                self.editor.set_mode(Mode::Insert);
            }
            KeyCode::Char('a') => {
                self.editor.enter_append_mode();
            }
            KeyCode::Char('o') => {
                self.editor.execute_motion(Motion::LineEnd);
                self.editor.insert_newline();
                self.editor.set_mode(Mode::Insert);
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.editor.execute_motion(Motion::Left);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.editor.execute_motion(Motion::Down);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.editor.execute_motion(Motion::Up);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.editor.execute_motion(Motion::Right);
            }
            KeyCode::Char('0') => {
                self.editor.execute_motion(Motion::LineStart);
            }
            KeyCode::Char('$') => {
                self.editor.execute_motion(Motion::LineEnd);
            }
            KeyCode::Char('^') => {
                self.editor.execute_motion(Motion::FirstNonBlank);
            }
            KeyCode::Char('g') => {
                // 进入 g 等待状态，等待第二个 g (gg)
                self.operator_state = OperatorState::G;
            }
            KeyCode::Char('G') => {
                self.editor.execute_motion(Motion::DocumentEnd);
                // 更新滚动偏移量，确保光标在可视区域内
                self.update_scroll_offset(terminal_height());
            }
            KeyCode::Char('w') => {
                self.editor.execute_motion(Motion::WordForward);
            }
            KeyCode::Char('b') => {
                self.editor.execute_motion(Motion::WordBackward);
            }
            KeyCode::Char('e') => {
                self.editor.execute_motion(Motion::WordEnd);
            }
            KeyCode::Char('x') => {
                self.editor.delete_char_to_register(None);
            }
            KeyCode::Char('"') => {
                // " - 进入寄存器选择状态
                self.operator_state = OperatorState::Register('\0'); // \0 表示等待操作符
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.execute_motion(Motion::PageDown);
            }
            KeyCode::Char('d') => {
                // d - 进入删除操作符等待状态
                self.operator_state = OperatorState::Delete;
            }
            KeyCode::Char('y') => {
                // y - 进入复制操作符等待状态
                self.operator_state = OperatorState::Yank;
            }
            KeyCode::Char('p') => {
                // p - 在光标后粘贴
                self.editor.paste(None, false);
            }
            KeyCode::Char('P') => {
                // P - 在光标前粘贴
                self.editor.paste(None, true);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.execute_motion(Motion::PageUp);
            }
            KeyCode::Char('u') => {
                self.editor.undo();
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.redo();
            }
            KeyCode::Char(':') => {
                self.editor.set_mode(Mode::Command);
                self.editor.command_line_mut().clear();
            }
            KeyCode::Char('/') => {
                self.editor.set_mode(Mode::SearchForward);
                self.editor.command_line_mut().clear();
            }
            KeyCode::Char('?') => {
                self.editor.set_mode(Mode::SearchBackward);
                self.editor.command_line_mut().clear();
            }
            KeyCode::Char('n') => {
                self.editor.search_next();
            }
            KeyCode::Char('N') => {
                self.editor.search_prev();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
            }
            _ => {}
        }
        
        self.update_scroll_offset(terminal_height());
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.editor.set_mode(Mode::Normal);
            }
            KeyCode::Char(c) => {
                self.editor.insert_char(c);
            }
            KeyCode::Enter => {
                self.editor.insert_newline();
            }
            KeyCode::Backspace => {
                self.editor.backspace();
            }
            KeyCode::Tab => {
                self.editor.insert_char(' ');
                self.editor.insert_char(' ');
                self.editor.insert_char(' ');
                self.editor.insert_char(' ');
            }
            _ => {}
        }
        
        self.update_scroll_offset(terminal_height());
    }

    fn handle_command_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.editor.set_mode(Mode::Normal);
                self.editor.command_line_mut().clear();
                self.editor.clear_message();
            }
            KeyCode::Enter => {
                let command = self.editor.command_line().to_string();
                self.editor.command_line_mut().clear();
                self.editor.set_mode(Mode::Normal);
                
                if let Err(e) = self.editor.execute_command(&command) {
                    self.editor.set_message(e);
                } else {
                    match command.as_str() {
                        "q" | "quit" | "qa" => {
                            self.should_quit = true;
                        }
                        "q!" => {
                            self.should_quit = true;
                        }
                        "wq" => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char(c) => {
                self.editor.command_line_mut().push(c);
            }
            KeyCode::Backspace => {
                self.editor.command_line_mut().pop();
            }
            _ => {}
        }
    }

    fn handle_search_mode(&mut self, key: KeyEvent) {
        use aivim_core::SearchDirection;

        match key.code {
            KeyCode::Esc => {
                self.editor.set_mode(Mode::Normal);
                self.editor.command_line_mut().clear();
            }
            KeyCode::Enter => {
                let pattern = self.editor.command_line().to_string();
                self.editor.command_line_mut().clear();
                self.editor.set_mode(Mode::Normal);

                let direction = match self.editor.mode() {
                    Mode::SearchForward => SearchDirection::Forward,
                    Mode::SearchBackward => SearchDirection::Backward,
                    _ => SearchDirection::Forward,
                };

                if !pattern.is_empty() {
                    self.editor.start_search(direction, &pattern);
                }
            }
            KeyCode::Char(c) => {
                self.editor.command_line_mut().push(c);
            }
            KeyCode::Backspace => {
                self.editor.command_line_mut().pop();
            }
            _ => {}
        }
    }

    /// 处理操作符 + 动作的组合
    fn handle_operator_motion(&mut self, key: KeyEvent, operator: OperatorState) {
        // 重置操作符状态
        self.operator_state = OperatorState::None;

        match key.code {
            KeyCode::Char('d') => {
                // dd - 删除当前行
                if operator == OperatorState::Delete {
                    self.editor.delete_line(None);
                }
            }
            KeyCode::Char('y') => {
                // yy - 复制当前行
                if operator == OperatorState::Yank {
                    self.editor.yank_line(None);
                }
            }
            KeyCode::Char('w') => {
                // dw/yw - 删除/复制到下一个单词
                match operator {
                    OperatorState::Delete => {
                        self.editor.delete_to_motion(Motion::WordForward);
                    }
                    OperatorState::Yank => {
                        self.editor.yank_to_motion(Motion::WordForward);
                    }
                    _ => {}
                }
            }
            KeyCode::Char('b') => {
                // db/yb - 删除/复制到上一个单词
                match operator {
                    OperatorState::Delete => {
                        self.editor.delete_to_motion(Motion::WordBackward);
                    }
                    OperatorState::Yank => {
                        self.editor.yank_to_motion(Motion::WordBackward);
                    }
                    _ => {}
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                // dl - 删除/复制到右边一个字符
                match operator {
                    OperatorState::Delete => {
                        self.editor.delete_to_motion(Motion::Right);
                    }
                    OperatorState::Yank => {
                        self.editor.yank_to_motion(Motion::Right);
                    }
                    _ => {}
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                // dh - 删除/复制到左边一个字符
                match operator {
                    OperatorState::Delete => {
                        self.editor.delete_to_motion(Motion::Left);
                    }
                    OperatorState::Yank => {
                        self.editor.yank_to_motion(Motion::Left);
                    }
                    _ => {}
                }
            }
            KeyCode::Char('$') => {
                // d$/y$ - 删除/复制到行尾
                match operator {
                    OperatorState::Delete => {
                        self.editor.delete_to_motion(Motion::LineEnd);
                    }
                    OperatorState::Yank => {
                        self.editor.yank_to_motion(Motion::LineEnd);
                    }
                    _ => {}
                }
            }
            KeyCode::Char('0') => {
                // d0/y0 - 删除/复制到行首
                match operator {
                    OperatorState::Delete => {
                        self.editor.delete_to_motion(Motion::LineStart);
                    }
                    OperatorState::Yank => {
                        self.editor.yank_to_motion(Motion::LineStart);
                    }
                    _ => {}
                }
            }
            KeyCode::Esc => {
                // 取消操作符
            }
            _ => {}
        }
    }

    /// 处理文本对象操作（daw, diw, yaw, yiw 等）
    fn handle_text_object(&mut self, key: KeyEvent, operator: TextObjectOperator, around: bool) {
        use aivim_core::text_object::TextObject;

        // 重置操作符状态
        self.operator_state = OperatorState::None;

        let text_object = match key.code {
            KeyCode::Char('w') => {
                if around {
                    TextObject::AroundWord
                } else {
                    TextObject::InnerWord
                }
            }
            KeyCode::Char('W') => {
                if around {
                    TextObject::AroundWord
                } else {
                    TextObject::InnerWord
                }
            }
            _ => {
                // 不支持的文本对象，取消操作
                return;
            }
        };

        match operator {
            TextObjectOperator::Delete => {
                self.editor.delete_text_object(text_object);
            }
            TextObjectOperator::Yank => {
                self.editor.yank_text_object(text_object);
            }
            TextObjectOperator::Change => {
                // TODO: 实现 change 操作
            }
        }
    }

    /// 处理寄存器选择（"ayy, "ap 等）
    fn handle_register(&mut self, key: KeyEvent, op: char) {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {
                if op == '\0' {
                    // 等待操作符（d/y/p等）
                    // 这里简化处理，直接等待下一个键作为操作符
                    // 实际上应该存储寄存器名，然后等待操作符
                    // 为了简化，我们假设用户输入的是操作符
                    match c {
                        'd' => self.operator_state = OperatorState::Delete,
                        'y' => self.operator_state = OperatorState::Yank,
                        'p' => {
                            // 从寄存器粘贴（简化实现，使用默认寄存器）
                            self.editor.paste(None, false);
                            self.operator_state = OperatorState::None;
                        }
                        _ => self.operator_state = OperatorState::None,
                    }
                }
            }
            KeyCode::Esc => {
                self.operator_state = OperatorState::None;
            }
            _ => {}
        }
    }

    fn update_scroll_offset(&mut self, viewport_height: usize) {
        let cursor_line = self.editor.cursor().line;
        self.scroll_offset = calculate_scroll_offset(
            cursor_line,
            viewport_height.saturating_sub(2),
            self.scroll_offset,
        );
    }
}

fn terminal_height() -> usize {
    crossterm::terminal::size()
        .map(|(_, h)| h as usize)
        .unwrap_or(24)
}
