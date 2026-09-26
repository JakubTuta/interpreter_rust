use crate::models::ast::{
    BinaryOp, Expression, ExpressionKind, ExpressionStatement, Literal, Statement, StatementKind,
    UnaryOp,
};
use crate::models::lexer::{Token, TokenType};
use crate::models::parser::{BinaryOperator, ParseError, UnaryOperator};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.validate_tokens()?;

        let mut statements = Vec::new();
        for statement in self.split_statements() {
            let parsed_statement = self.parse_statement(&statement)?;
            statements.push(parsed_statement);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self, statement: &Vec<Token>) -> Result<Statement, ParseError> {
        self.position = 0;
        let parsed_expression = self.parse_expression(statement, 0)?;

        Ok(Statement {
            row: statement[0].row,
            col: statement[0].col,
            kind: StatementKind::ExpressionStatement(ExpressionStatement {
                expression: parsed_expression,
            }),
        })
    }

    fn validate_tokens(&self) -> Result<(), ParseError> {
        if self.tokens.is_empty() {
            return Err(ParseError {
                row: None,
                col: None,
                message: String::from("Token list is empty"),
            });
        }

        let last_token = self.tokens.last().unwrap();
        if last_token.token_type != TokenType::Eof {
            return Err(ParseError {
                row: last_token.row,
                col: last_token.col,
                message: String::from("Last token has to be EOF"),
            });
        }

        Ok(())
    }

    fn parse_expression(
        &mut self,
        statement: &Vec<Token>,
        min_precedence: i8,
    ) -> Result<Expression, ParseError> {
        let mut left_expr = self.parse_prefix(statement)?;

        loop {
            let token = statement[self.position];
            let current_precedence = Self::precedence(token.token_type);

            if current_precedence < min_precedence {
                break;
            }

            let operator = Self::binary_operator(token)?;
            self.position += 1;

            let right_expr = self.parse_expression(statement, current_precedence + 1)?;
            left_expr = Expression {
                row: token.row,
                col: token.col,
                kind: ExpressionKind::Binary(BinaryOp {
                    left: Box::new(left_expr),
                    op: operator,
                    right: Box::new(right_expr),
                }),
            };
        }

        Ok(left_expr)
    }

    fn parse_prefix(&mut self, statement: &Vec<Token>) -> Result<Expression, ParseError> {
        let token = statement[self.position];

        match token.token_type {
            TokenType::Number(val) => {
                self.position += 1;
                Ok(Expression {
                    row: token.row,
                    col: token.col,
                    kind: ExpressionKind::Literal(Literal { value: val }),
                })
            }
            TokenType::Minus => {
                self.position += 1;
                let operand = self.parse_expression(statement, 3)?;
                Ok(Expression {
                    row: token.row,
                    col: token.col,
                    kind: ExpressionKind::Unary(UnaryOp {
                        op: UnaryOperator::Neg,
                        operand: Box::new(operand),
                    }),
                })
            }
            TokenType::LParen => {
                self.position += 1;
                let expression = self.parse_expression(statement, 0)?;

                let current_token = statement[self.position];
                if current_token.token_type != TokenType::RParen {
                    return Err(ParseError {
                        row: current_token.row,
                        col: current_token.col,
                        message: format!("Expected ')', found '{current_token}'"),
                    });
                }

                self.position += 1;
                Ok(expression)
            }
            _ => Err(ParseError {
                row: token.row,
                col: token.col,
                message: format!("Unexpected token '{token}', expected an expression"),
            }),
        }
    }

    fn precedence(token_type: TokenType) -> i8 {
        match token_type {
            TokenType::Plus | TokenType::Minus => 1,
            TokenType::Star | TokenType::Slash => 2,
            _ => -1,
        }
    }

    fn binary_operator(token: Token) -> Result<BinaryOperator, ParseError> {
        match token.token_type {
            TokenType::Plus => Ok(BinaryOperator::Add),
            TokenType::Minus => Ok(BinaryOperator::Sub),
            TokenType::Star => Ok(BinaryOperator::Mul),
            TokenType::Slash => Ok(BinaryOperator::Div),
            _ => Err(ParseError {
                row: token.row,
                col: token.col,
                message: format!("Unexpected token '{token}', expected an operator"),
            }),
        }
    }

    fn split_statements(&self) -> Vec<Vec<Token>> {
        let mut statements = Vec::new();
        let mut current_vec = Vec::new();

        for token in &self.tokens {
            current_vec.push(*token);

            if token.token_type == TokenType::Semicolon {
                statements.push(current_vec);
                current_vec = Vec::new();
            }
        }

        return statements;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::models::lexer::NumberValue;

    fn parse(source: &str) -> Result<Expression, ParseError> {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let statements = Parser::new(tokens).parse()?;
        let first = statements.into_iter().next().expect("expected at least one statement");
        let StatementKind::ExpressionStatement(stmt) = first.kind;
        Ok(stmt.expression)
    }

    #[test]
    fn parses_simple_addition() {
        let expr = parse("1 + 2").unwrap();
        match expr {
            Expression {
                kind: ExpressionKind::Binary(b),
                ..
            } => assert_eq!(b.op, BinaryOperator::Add),
            other => panic!("expected a binary expression, got {other:?}"),
        }
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        let expr = parse("1 + 2 * 3").unwrap();
        match expr {
            Expression {
                kind:
                    ExpressionKind::Binary(BinaryOp {
                        op: BinaryOperator::Add,
                        right,
                        ..
                    }),
                ..
            } => match *right {
                Expression {
                    kind:
                        ExpressionKind::Binary(BinaryOp {
                            op: BinaryOperator::Mul,
                            ..
                        }),
                    ..
                } => {}
                other => panic!("expected the right side to be a multiplication, got {other:?}"),
            },
            other => panic!("expected a top-level addition, got {other:?}"),
        }
    }

    #[test]
    fn parentheses_override_precedence() {
        let expr = parse("(1 + 2) * 3").unwrap();
        match expr {
            Expression {
                kind: ExpressionKind::Binary(b),
                ..
            } => assert_eq!(b.op, BinaryOperator::Mul),
            other => panic!("expected a binary expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_unary_minus() {
        let expr = parse("-5").unwrap();
        match expr {
            Expression {
                kind: ExpressionKind::Unary(u),
                ..
            } => assert_eq!(u.op, UnaryOperator::Neg),
            other => panic!("expected a unary expression, got {other:?}"),
        }
    }

    #[test]
    fn errors_on_empty_token_list() {
        let err = Parser::new(vec![]).parse().unwrap_err();
        assert_eq!(err.message, "Token list is empty");
    }

    #[test]
    fn errors_on_missing_eof() {
        let tokens = vec![Token {
            token_type: TokenType::Number(NumberValue::Int(1)),
            row: Some(1),
            col: Some(0),
        }];
        let err = Parser::new(tokens).parse().unwrap_err();
        assert_eq!(err.message, "Last token has to be EOF");
    }

    #[test]
    fn errors_on_unclosed_parenthesis() {
        assert!(parse("(1 + 2").is_err());
    }

    #[test]
    fn errors_on_trailing_tokens() {
        assert!(parse("1 2").is_err());
    }
}
