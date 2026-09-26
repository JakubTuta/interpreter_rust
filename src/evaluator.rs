use crate::models::ast::{
    Expression, ExpressionKind, ExpressionStatement, Statement, StatementKind,
};
use crate::models::evaluator::EvaluateError;
use crate::models::lexer::NumberValue;
use crate::models::parser::{BinaryOperator, UnaryOperator};

#[derive(Debug)]
pub struct Evaluator {
    statements: Vec<Statement>,
}

impl Evaluator {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub fn evaluate(&self) -> Result<Vec<NumberValue>, EvaluateError> {
        let mut evaluated = Vec::new();

        for statement in &self.statements {
            evaluated.push(self.evaluate_statement(&statement)?);
        }

        Ok(evaluated)
    }

    pub fn evaluate_statement(&self, statement: &Statement) -> Result<NumberValue, EvaluateError> {
        match &statement.kind {
            StatementKind::ExpressionStatement(ExpressionStatement { expression: expr }) => {
                Ok(self.evaluate_expression(&expr)?)
            }
        }
    }

    pub fn evaluate_expression(
        &self,
        expression: &Expression,
    ) -> Result<NumberValue, EvaluateError> {
        match &expression.kind {
            ExpressionKind::Literal(expr) => Ok(expr.value),

            ExpressionKind::Unary(expr) => {
                let value = self.evaluate_expression(&expr.operand)?;
                match expr.op {
                    UnaryOperator::Neg => Ok(-value),
                }
            }

            ExpressionKind::Binary(expr) => {
                let left_value = self.evaluate_expression(&expr.left)?;
                let right_value = self.evaluate_expression(&expr.right)?;

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

    fn eval(source: &str) -> Result<Vec<NumberValue>, EvaluateError> {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let expr = Parser::new(tokens).parse().unwrap();
        Evaluator::new(expr).evaluate()
    }

    #[test]
    fn adds_two_ints() {
        let mut results = Vec::new();
        results.push(NumberValue::Int(3));
        assert_eq!(eval("1 + 2").unwrap(), results);
    }

    #[test]
    fn respects_operator_precedence() {
        let mut results = Vec::new();
        results.push(NumberValue::Int(7));
        assert_eq!(eval("1 + 2 * 3").unwrap(), results);
    }

    #[test]
    fn parentheses_override_precedence() {
        let mut results = Vec::new();
        results.push(NumberValue::Int(9));
        assert_eq!(eval("(1 + 2) * 3").unwrap(), results);
    }

    #[test]
    fn unary_minus_negates() {
        let mut results = Vec::new();
        results.push(NumberValue::Int(-2));
        assert_eq!(eval("-5 + 3").unwrap(), results);
    }

    #[test]
    fn int_and_int_stays_int() {
        let mut results = Vec::new();
        results.push(NumberValue::Int(5));
        assert_eq!(eval("7 - 2").unwrap(), results);
    }

    #[test]
    fn mixing_float_widens_to_float() {
        let mut results = Vec::new();
        results.push(NumberValue::Float(3.5));
        assert_eq!(eval("1 + 2.5").unwrap(), results);
    }

    #[test]
    fn division_of_two_ints_is_still_a_float() {
        let mut results = Vec::new();
        results.push(NumberValue::Float(3.5));
        assert_eq!(eval("7 / 2").unwrap(), results);
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
