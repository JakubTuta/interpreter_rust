use std::fmt;

use crate::models::lexer::NumberValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub value: NumberValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOp {
    pub left: Box<Expr>,
    pub op: BinaryOperator,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOp {
    pub op: UnaryOperator,
    pub operand: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    Binary(BinaryOp),
    Unary(UnaryOp),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub row: Option<usize>,
    pub col: Option<usize>,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub row: Option<usize>,
    pub col: Option<usize>,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.row, self.col) {
            (Some(row), Some(col)) => {
                write!(f, "{} (line {}, column {})", self.message, row, col)
            }
            _ => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}
