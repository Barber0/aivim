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
    Delete { register: Option<char> },      // d - 等待动作，可指定寄存器
    Yank { register: Option<char> },        // y - 等待动作，可指定寄存器
    Change { register: Option<char> },      // c - 等待动作 (计划中)
    G,           // g - 等待第二个g (gg)
    TextObject { operator: TextObjectOperator, around: bool, register: Option<char> }, // a/i - 等待文本对象
    RegisterPending(Option<char>), // " - 等待寄存器名，Some(char)表示已选寄存器，等待操作符
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
    show_registers_panel: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
            event_handler: EventHandler::new(Duration::from_millis(50)),
            scroll_offset: 0,
            should_quit: false,
            operator_state: OperatorState::None,
            show_registers_panel: false,
        }
    }

    pub fn with_file(path: PathBuf) -> io::Result<Self> {
        Ok(Self {
            editor: Editor::with_file(&path)?,
            event_handler: EventHandler::new(Duration::from_millis(50)),
            scroll_offset: 0,
            should_quit: false,
            operator_state: OperatorState::None,
            show_registers_panel: false,
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
            terminal.draw(|f| ui::draw(f, &self.editor, self.scroll_offset, self.operator_state, self.show_registers_panel))?;

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
        // 如果寄存器面板正在显示，优先处理关闭操作
        if self.show_registers_panel {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.show_registers_panel = false;
                    return;
                }
                _ => {
                    // 其他按键也关闭面板，但继续处理按键
                    self.show_registers_panel = false;
                }
            }
        }

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
            OperatorState::Delete { register } => {
                // 检查是否是文本对象操作（daw, diw）
                match key.code {
                    KeyCode::Char('a') => {
                        self.operator_state = OperatorState::TextObject { 
                            operator: TextObjectOperator::Delete, 
                            around: true,
                            register,
                        };
                        return;
                    }
                    KeyCode::Char('i') => {
                        self.operator_state = OperatorState::TextObject { 
                            operator: TextObjectOperator::Delete, 
                            around: false,
                            register,
                        };
                        return;
                    }
                    _ => {
                        self.handle_operator_motion(key, OperatorState::Delete { register });
                        return;
                    }
                }
            }
            OperatorState::Yank { register } => {
                // 检查是否是文本对象操作（yaw, yiw）
                match key.code {
                    KeyCode::Char('a') => {
                        self.operator_state = OperatorState::TextObject { 
                            operator: TextObjectOperator::Yank, 
                            around: true,
                            register,
                        };
                        return;
                    }
                    KeyCode::Char('i') => {
                        self.operator_state = OperatorState::TextObject { 
                            operator: TextObjectOperator::Yank, 
                            around: false,
                            register,
                        };
                        return;
                    }
                    _ => {
                        self.handle_operator_motion(key, OperatorState::Yank { register });
                        return;
                    }
                }
            }
            OperatorState::TextObject { operator, around, register } => {
                // 处理文本对象（w, W, s, S, p, P）
                self.handle_text_object(key, operator, around, register);
                return;
            }
            OperatorState::RegisterPending(pending_reg) => {
                // 处理寄存器选择
                self.handle_register_selection(key, pending_reg);
                return;
            }
            OperatorState::Change { .. } => {
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
                // " - 进入寄存器选择状态（等待寄存器名）
                self.operator_state = OperatorState::RegisterPending(None);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.execute_motion(Motion::PageDown);
            }
            KeyCode::Char('d') => {
                // d - 进入删除操作符等待状态
                self.operator_state = OperatorState::Delete { register: None };
            }
            KeyCode::Char('y') => {
                // y - 进入复制操作符等待状态
                self.operator_state = OperatorState::Yank { register: None };
            }
            KeyCode::Char('p') => {
                // p - 在光标后粘贴（无名寄存器）
                self.editor.paste(None, false);
            }
            KeyCode::Char('P') => {
                // P - 在光标前粘贴（无名寄存器）
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
                
                // 检查是否是 registers 命令
                if command == "reg" || command == "registers" {
                    self.show_registers_panel = true;
                } else if let Err(e) = self.editor.execute_command(&command) {
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

    /// 处理操作符 + 动作的组合（支持命名寄存器）
    fn handle_operator_motion(&mut self, key: KeyEvent, operator: OperatorState) {
        // 重置操作符状态
        self.operator_state = OperatorState::None;

        // 提取操作符类型和寄存器
        let (is_delete, register) = match operator {
            OperatorState::Delete { register } => (true, register),
            OperatorState::Yank { register } => (false, register),
            _ => return,
        };

        match key.code {
            KeyCode::Char('d') => {
                // dd - 删除当前行
                if is_delete {
                    self.editor.delete_line(register);
                }
            }
            KeyCode::Char('y') => {
                // yy - 复制当前行
                if !is_delete {
                    self.editor.yank_line(register);
                }
            }
            KeyCode::Char('w') => {
                // dw/yw - 删除/复制到下一个单词
                if is_delete {
                    self.editor.delete_to_motion_with_register(Motion::WordForward, register);
                } else {
                    self.editor.yank_to_motion_with_register(Motion::WordForward, register);
                }
            }
            KeyCode::Char('b') => {
                // db/yb - 删除/复制到上一个单词
                if is_delete {
                    self.editor.delete_to_motion_with_register(Motion::WordBackward, register);
                } else {
                    self.editor.yank_to_motion_with_register(Motion::WordBackward, register);
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                // dl/yl - 删除/复制到右边一个字符
                if is_delete {
                    self.editor.delete_to_motion_with_register(Motion::Right, register);
                } else {
                    self.editor.yank_to_motion_with_register(Motion::Right, register);
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                // dh/yh - 删除/复制到左边一个字符
                if is_delete {
                    self.editor.delete_to_motion_with_register(Motion::Left, register);
                } else {
                    self.editor.yank_to_motion_with_register(Motion::Left, register);
                }
            }
            KeyCode::Char('$') => {
                // d$/y$ - 删除/复制到行尾
                if is_delete {
                    self.editor.delete_to_motion_with_register(Motion::LineEnd, register);
                } else {
                    self.editor.yank_to_motion_with_register(Motion::LineEnd, register);
                }
            }
            KeyCode::Char('0') => {
                // d0/y0 - 删除/复制到行首
                if is_delete {
                    self.editor.delete_to_motion_with_register(Motion::LineStart, register);
                } else {
                    self.editor.yank_to_motion_with_register(Motion::LineStart, register);
                }
            }
            KeyCode::Esc => {
                // 取消操作符
            }
            _ => {}
        }
    }

    /// 处理文本对象操作（daw, diw, yaw, yiw 等，支持命名寄存器）
    fn handle_text_object(&mut self, key: KeyEvent, operator: TextObjectOperator, around: bool, register: Option<char>) {
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
                if let Some(content) = self.editor.delete_text_object(text_object) {
                    if let Some(reg) = register {
                        self.editor.register_manager_mut().set(reg, content, false);
                    }
                }
            }
            TextObjectOperator::Yank => {
                if let Some(content) = self.editor.yank_text_object(text_object) {
                    if let Some(reg) = register {
                        self.editor.register_manager_mut().set(reg, content, false);
                    }
                }
            }
            TextObjectOperator::Change => {
                // TODO: 实现 change 操作
            }
        }
    }

    /// 处理寄存器选择（"ayy, "ap 等）
    /// 
    /// 两步序列：
    /// 1. " + a → 进入 RegisterPending(Some('a'))，等待操作符
    /// 2. y + y → 执行操作，使用寄存器 a
    fn handle_register_selection(&mut self, key: KeyEvent, pending_reg: Option<char>) {
        match pending_reg {
            None => {
                // 第一步：等待寄存器名
                match key.code {
                    KeyCode::Char(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {
                        // 已选择寄存器名，进入等待操作符状态
                        self.operator_state = OperatorState::RegisterPending(Some(c));
                    }
                    KeyCode::Esc => {
                        self.operator_state = OperatorState::None;
                    }
                    _ => {
                        // 无效输入，取消
                        self.operator_state = OperatorState::None;
                    }
                }
            }
            Some(reg) => {
                // 第二步：已选择寄存器，等待操作符（d/y/p）
                match key.code {
                    KeyCode::Char('d') => {
                        // "ad... 进入删除操作符状态，指定寄存器
                        self.operator_state = OperatorState::Delete { register: Some(reg) };
                    }
                    KeyCode::Char('y') => {
                        // "ay... 进入复制操作符状态，指定寄存器
                        self.operator_state = OperatorState::Yank { register: Some(reg) };
                    }
                    KeyCode::Char('p') => {
                        // "ap - 直接从指定寄存器粘贴
                        self.editor.paste(Some(reg), false);
                        self.operator_state = OperatorState::None;
                    }
                    KeyCode::Char('P') => {
                        // "aP - 直接从指定寄存器粘贴（前）
                        self.editor.paste(Some(reg), true);
                        self.operator_state = OperatorState::None;
                    }
                    KeyCode::Esc => {
                        self.operator_state = OperatorState::None;
                    }
                    _ => {
                        // 无效输入，取消
                        self.operator_state = OperatorState::None;
                    }
                }
            }
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
