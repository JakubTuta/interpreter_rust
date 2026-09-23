use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Number(NumberValue),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberValue {
    Int(i64),
    Float(f64),
}

impl NumberValue {
    pub fn as_f64(self) -> f64 {
        match self {
            NumberValue::Int(v) => v as f64,
            NumberValue::Float(v) => v,
        }
    }

    fn both_ints(self, other: Self) -> Option<(i64, i64)> {
        match (self, other) {
            (NumberValue::Int(a), NumberValue::Int(b)) => Some((a, b)),
            _ => None,
        }
    }
}

impl std::ops::Neg for NumberValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            NumberValue::Int(v) => NumberValue::Int(-v),
            NumberValue::Float(v) => NumberValue::Float(-v),
        }
    }
}

impl std::ops::Add for NumberValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self.both_ints(other) {
            Some((a, b)) => NumberValue::Int(a + b),
            None => NumberValue::Float(self.as_f64() + other.as_f64()),
        }
    }
}

impl std::ops::Sub for NumberValue {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match self.both_ints(other) {
            Some((a, b)) => NumberValue::Int(a - b),
            None => NumberValue::Float(self.as_f64() - other.as_f64()),
        }
    }
}

impl std::ops::Mul for NumberValue {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match self.both_ints(other) {
            Some((a, b)) => NumberValue::Int(a * b),
            None => NumberValue::Float(self.as_f64() * other.as_f64()),
        }
    }
}

impl std::ops::Div for NumberValue {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        NumberValue::Float(self.as_f64() / other.as_f64())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub row: Option<usize>,
    pub col: Option<usize>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.token_type {
            TokenType::Number(val) => match val {
                NumberValue::Int(n) => write!(f, "{n}"),
                NumberValue::Float(n) => write!(f, "{n}"),
            },
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Star => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::LParen => write!(f, "("),
            TokenType::RParen => write!(f, ")"),
            TokenType::Eof => write!(f, "<EOF>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub row: usize,
    pub col: usize,
    pub character: char,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Incorrect character at line {}, column {}: {}",
            self.row, self.col, self.character
        )
    }
}

impl std::error::Error for LexError {}
