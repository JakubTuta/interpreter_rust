use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluateError {
    pub row: Option<usize>,
    pub col: Option<usize>,
    pub message: String,
}

impl fmt::Display for EvaluateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.row, self.col) {
            (Some(row), Some(col)) => {
                write!(f, "{} (line {}, column {})", self.message, row, col)
            }
            _ => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for EvaluateError {}
