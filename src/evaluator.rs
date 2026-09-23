use crate::models::evaluator::EvaluateError;
use crate::models::lexer::NumberValue;
use crate::models::parser::{BinaryOperator, Expr, ExprKind, UnaryOperator};

#[derive(Debug, Default)]
pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(&self, expression: &Expr) -> Result<NumberValue, EvaluateError> {
        match &expression.kind {
            ExprKind::Literal(expr) => Ok(expr.value),

            ExprKind::Unary(expr) => {
                let value = self.evaluate(&expr.operand)?;
                match expr.op {
                    UnaryOperator::Neg => Ok(-value),
                }
            }

            ExprKind::Binary(expr) => {
                let left_value = self.evaluate(&expr.left)?;
                let right_value = self.evaluate(&expr.right)?;

                match expr.op {
                    BinaryOperator::Add => Ok(left_value + right_value),
                    BinaryOperator::Sub => Ok(left_value - right_value),
                    BinaryOperator::Mul => Ok(left_value * right_value),
                    BinaryOperator::Div => {
                        if right_value.as_f64() == 0.0 {
                            return Err(EvaluateError {
                                row: expression.row,
                                col: expression.col,
                                message: String::from("Division by zero"),
                            });
                        }
                        Ok(left_value / right_value)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn eval(source: &str) -> Result<NumberValue, EvaluateError> {
        let tokens = Lexer::new().tokenize(source).unwrap();
        let expr = Parser::new().parse(tokens).unwrap();
        Evaluator::new().evaluate(&expr)
    }

    #[test]
    fn adds_two_ints() {
        assert_eq!(eval("1 + 2").unwrap(), NumberValue::Int(3));
    }

    #[test]
    fn respects_operator_precedence() {
        assert_eq!(eval("1 + 2 * 3").unwrap(), NumberValue::Int(7));
    }

    #[test]
    fn parentheses_override_precedence() {
        assert_eq!(eval("(1 + 2) * 3").unwrap(), NumberValue::Int(9));
    }

    #[test]
    fn unary_minus_negates() {
        assert_eq!(eval("-5 + 3").unwrap(), NumberValue::Int(-2));
    }

    #[test]
    fn int_and_int_stays_int() {
        assert_eq!(eval("7 - 2").unwrap(), NumberValue::Int(5));
    }

    #[test]
    fn mixing_float_widens_to_float() {
        assert_eq!(eval("1 + 2.5").unwrap(), NumberValue::Float(3.5));
    }

    #[test]
    fn division_of_two_ints_is_still_a_float() {
        assert_eq!(eval("7 / 2").unwrap(), NumberValue::Float(3.5));
    }

    #[test]
    fn division_by_int_zero_is_an_error() {
        assert!(eval("1 / 0").is_err());
    }

    #[test]
    fn division_by_float_zero_is_an_error() {
        assert!(eval("1 / 0.0").is_err());
    }
}
