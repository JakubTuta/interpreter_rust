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

#[derive(Debug, Default)]
pub struct Lexer {
    chars: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tokenize(&mut self, source: &str) -> Result<Vec<Token>, LexError> {
        self.chars = source.chars().collect();
        self.position = 0;

        let mut tokens = Vec::new();

        while let Some(&ch) = self.chars.get(self.position) {
            if ch.is_ascii_digit() {
                tokens.push(self.consume_number()?);
                continue;
            } else if ch.is_whitespace() {
                // no token produced
            } else if let Some(token_type) = Self::operator_token_type(ch) {
                tokens.push(Token {
                    token_type,
                    row: Some(1),
                    col: Some(self.position),
                });
            } else {
                return Err(LexError {
                    row: 1,
                    col: self.position,
                    character: ch,
                });
            }

            self.position += 1;
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            row: None,
            col: None,
        });

        Ok(tokens)
    }

    fn consume_number(&mut self) -> Result<Token, LexError> {
        let start = self.position;
        let mut index = start;

        index += self.count_digits_from(index);

        if self.chars.get(index) == Some(&'.') {
            let dot_index = index;
            index += 1;

            let fraction_len = self.count_digits_from(index);
            if fraction_len == 0 {
                return Err(self.error_at(dot_index));
            }
            index += fraction_len;
        }

        if let Some(&next) = self.chars.get(index) {
            if !(next.is_whitespace() || Self::operator_token_type(next).is_some()) {
                return Err(self.error_at(index));
            }
        }

        let lexeme: String = self.chars[start..index].iter().collect();
        self.position = index;

        let value = if lexeme.contains('.') {
            NumberValue::Float(lexeme.parse().map_err(|_| self.error_at(start))?)
        } else {
            NumberValue::Int(lexeme.parse().map_err(|_| self.error_at(start))?)
        };

        Ok(Token {
            token_type: TokenType::Number(value),
            row: Some(1),
            col: Some(start),
        })
    }

    fn count_digits_from(&self, index: usize) -> usize {
        self.chars[index..]
            .iter()
            .take_while(|c| c.is_ascii_digit())
            .count()
    }

    fn error_at(&self, index: usize) -> LexError {
        LexError {
            row: 1,
            col: index,
            character: self.chars[index],
        }
    }

    fn operator_token_type(ch: char) -> Option<TokenType> {
        match ch {
            '+' => Some(TokenType::Plus),
            '-' => Some(TokenType::Minus),
            '*' => Some(TokenType::Star),
            '/' => Some(TokenType::Slash),
            '(' => Some(TokenType::LParen),
            ')' => Some(TokenType::RParen),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_types(source: &str) -> Vec<TokenType> {
        Lexer::new()
            .tokenize(source)
            .unwrap()
            .iter()
            .map(|t| t.token_type)
            .collect()
    }

    #[test]
    fn tokenizes_addition() {
        use TokenType::*;
        assert_eq!(
            token_types("1 + 2"),
            vec![
                Number(NumberValue::Int(1)),
                Plus,
                Number(NumberValue::Int(2)),
                Eof
            ]
        );
    }

    #[test]
    fn tokenizes_all_operators() {
        use TokenType::*;
        assert_eq!(
            token_types("1 + 2 - 3 * 4 / (5)"),
            vec![
                Number(NumberValue::Int(1)),
                Plus,
                Number(NumberValue::Int(2)),
                Minus,
                Number(NumberValue::Int(3)),
                Star,
                Number(NumberValue::Int(4)),
                Slash,
                LParen,
                Number(NumberValue::Int(5)),
                RParen,
                Eof
            ]
        );
    }

    #[test]
    fn parses_int_and_float_values() {
        let tokens = Lexer::new().tokenize("6 3.14").unwrap();
        assert_eq!(tokens[0].token_type, TokenType::Number(NumberValue::Int(6)));
        assert_eq!(
            tokens[1].token_type,
            TokenType::Number(NumberValue::Float(3.14))
        );
    }

    #[test]
    fn single_digit_source_produces_number_then_eof() {
        use TokenType::*;
        assert_eq!(token_types("6"), vec![Number(NumberValue::Int(6)), Eof]);
    }

    #[test]
    fn unexpected_character_is_an_error_not_silently_skipped() {
        let err = Lexer::new().tokenize("asd").unwrap_err();
        assert_eq!(err.col, 0);
        assert_eq!(err.character, 'a');
    }

    #[test]
    fn number_with_trailing_garbage_is_an_error_not_a_panic() {
        let err = Lexer::new().tokenize("123asd").unwrap_err();
        assert_eq!(err.col, 3);
        assert_eq!(err.character, 'a');
    }

    #[test]
    fn error_location_is_correct_even_after_a_valid_decimal_part() {
        let err = Lexer::new().tokenize("123.45asd").unwrap_err();
        assert_eq!(err.col, 6);
        assert_eq!(err.character, 'a');
    }

    #[test]
    fn trailing_dot_with_no_fraction_is_an_error() {
        let err = Lexer::new().tokenize("1.").unwrap_err();
        assert_eq!(err.col, 1);
        assert_eq!(err.character, '.');
    }
}
