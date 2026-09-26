use crate::models::lexer::NumberValue;
use crate::models::parser::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub value: NumberValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOp {
    pub left: Box<Expression>,
    pub op: BinaryOperator,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOp {
    pub op: UnaryOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Literal(Literal),
    Binary(BinaryOp),
    Unary(UnaryOp),
}

// evaluates to value
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub row: Option<usize>,
    pub col: Option<usize>,
    pub kind: ExpressionKind,
}

#[derive(Debug)]
pub struct ExpressionStatement {
    pub expression: Expression,
}

#[derive(Debug)]
pub enum StatementKind {
    ExpressionStatement(ExpressionStatement),
}

#[derive(Debug)]
// performs an action
pub struct Statement {
    pub row: Option<usize>,
    pub col: Option<usize>,
    pub kind: StatementKind,
}
