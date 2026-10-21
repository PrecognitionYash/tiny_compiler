use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileError {
    pub phase: u8,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl CompileError {
    pub fn new(phase: u8, line: usize, column: usize, message: impl Into<String>) -> Self {
        Self {
            phase,
            line,
            column,
            message: message.into(),
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(0, 0, 0, message)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "phase {}: {}", self.phase, self.message)
        } else if self.column == 0 {
            write!(
                f,
                "phase {} at line {}: {}",
                self.phase, self.line, self.message
            )
        } else {
            write!(
                f,
                "phase {} at {}:{}: {}",
                self.phase, self.line, self.column, self.message
            )
        }
    }
}

impl std::error::Error for CompileError {}

pub type CompileResult<T> = Result<T, CompileError>;

pub fn phase_name(phase: u8) -> &'static str {
    match phase {
        0 => "Preprocessor",
        1 => "Lexical analysis",
        2 => "Syntax analysis",
        3 => "Semantic analysis",
        4 => "Intermediate representation",
        5 => "Optimization and basic blocks",
        6 => "Target code generation",
        _ => "Unknown phase",
    }
}
