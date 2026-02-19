#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Replace,
    SearchForward,
    SearchBackward,
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::Command => "COMMAND",
            Mode::Replace => "REPLACE",
            Mode::SearchForward => "SEARCH",
            Mode::SearchBackward => "SEARCH",
        }
    }

    pub fn is_insert(&self) -> bool {
        matches!(self, Mode::Insert | Mode::Replace)
    }

    pub fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual)
    }

    pub fn is_command(&self) -> bool {
        matches!(self, Mode::Command)
    }

    pub fn is_search(&self) -> bool {
        matches!(self, Mode::SearchForward | Mode::SearchBackward)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}
