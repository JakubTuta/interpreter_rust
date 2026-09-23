use crate::models::lexer::{Token, TokenType};
use crate::models::parser::{
    BinaryOp, BinaryOperator, Expr, ExprKind, Literal, ParseError, UnaryOp, UnaryOperator,
};

#[derive(Debug, Default)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Expr, ParseError> {
        self.tokens = tokens;
        self.position = 0;

        self.validate_tokens()?;

        let expression = self.parse_expression(0)?;
        let current_token = self.tokens[self.position];

        if current_token.token_type != TokenType::Eof {
            return Err(ParseError {
                row: current_token.row,
                col: current_token.col,
                message: format!("Unexpected token '{current_token}', expected end of input"),
            });
        }

        Ok(expression)
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

    fn parse_expression(&mut self, min_precedence: i8) -> Result<Expr, ParseError> {
        let mut left_expr = self.parse_prefix()?;

        loop {
            let token = self.tokens[self.position];
            let current_precedence = Self::precedence(token.token_type);

            if current_precedence < min_precedence {
                break;
            }

            let operator = Self::binary_operator(token)?;
            self.position += 1;

            let right_expr = self.parse_expression(current_precedence + 1)?;
            left_expr = Expr {
                row: token.row,
                col: token.col,
                kind: ExprKind::Binary(BinaryOp {
                    left: Box::new(left_expr),
                    op: operator,
                    right: Box::new(right_expr),
                }),
            };
        }

        Ok(left_expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let token = self.tokens[self.position];

        match token.token_type {
            TokenType::Number(val) => {
                self.position += 1;
                Ok(Expr {
                    row: token.row,
                    col: token.col,
                    kind: ExprKind::Literal(Literal { value: val }),
                })
            }
            TokenType::Minus => {
                self.position += 1;
                let operand = self.parse_expression(3)?;
                Ok(Expr {
                    row: token.row,
                    col: token.col,
                    kind: ExprKind::Unary(UnaryOp {
                        op: UnaryOperator::Neg,
                        operand: Box::new(operand),
                    }),
                })
            }
            TokenType::LParen => {
                self.position += 1;
                let expression = self.parse_expression(0)?;

                let current_token = self.tokens[self.position];
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::models::lexer::NumberValue;

    fn parse(source: &str) -> Result<Expr, ParseError> {
        let tokens = Lexer::new().tokenize(source).unwrap();
        Parser::new().parse(tokens)
    }

    #[test]
    fn parses_simple_addition() {
        let expr = parse("1 + 2").unwrap();
        match expr {
            Expr {
                kind: ExprKind::Binary(b),
                ..
            } => assert_eq!(b.op, BinaryOperator::Add),
            other => panic!("expected a binary expression, got {other:?}"),
        }
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        let expr = parse("1 + 2 * 3").unwrap();
        match expr {
            Expr {
                kind:
                    ExprKind::Binary(BinaryOp {
                        op: BinaryOperator::Add,
                        right,
                        ..
                    }),
                ..
            } => match *right {
                Expr {
                    kind:
                        ExprKind::Binary(BinaryOp {
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
            Expr {
                kind: ExprKind::Binary(b),
                ..
            } => assert_eq!(b.op, BinaryOperator::Mul),
            other => panic!("expected a binary expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_unary_minus() {
        let expr = parse("-5").unwrap();
        match expr {
            Expr {
                kind: ExprKind::Unary(u),
                ..
            } => assert_eq!(u.op, UnaryOperator::Neg),
            other => panic!("expected a unary expression, got {other:?}"),
        }
    }

    #[test]
    fn errors_on_empty_token_list() {
        let err = Parser::new().parse(vec![]).unwrap_err();
        assert_eq!(err.message, "Token list is empty");
    }

    #[test]
    fn errors_on_missing_eof() {
        let tokens = vec![Token {
            token_type: TokenType::Number(NumberValue::Int(1)),
            row: Some(1),
            col: Some(0),
        }];
        let err = Parser::new().parse(tokens).unwrap_err();
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
