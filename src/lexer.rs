use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Number,
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
    Int(u32),
    Float(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub row: Option<usize>,
    pub col: Option<usize>,
    pub value: Option<NumberValue>,
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
            } else if let Some(token_type) = operator_token_type(ch) {
                tokens.push(Token {
                    token_type,
                    row: Some(1),
                    col: Some(self.position),
                    value: None,
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
            value: None,
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
            if !(next.is_whitespace() || operator_token_type(next).is_some()) {
                return Err(self.error_at(index));
            }
        }

        let lexeme: String = self.chars[start..index].iter().collect();
        self.position = index;

        let value = if lexeme.contains('.') {
            NumberValue::Float(
                lexeme
                    .parse()
                    .expect("lexeme was validated as digits + '.'"),
            )
        } else {
            NumberValue::Int(lexeme.parse().expect("lexeme was validated as digits"))
        };

        Ok(Token {
            token_type: TokenType::Number,
            row: Some(1),
            col: Some(start),
            value: Some(value),
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
        assert_eq!(token_types("1 + 2"), vec![Number, Plus, Number, Eof]);
    }

    #[test]
    fn tokenizes_all_operators() {
        use TokenType::*;
        assert_eq!(
            token_types("1 + 2 - 3 * 4 / (5)"),
            vec![
                Number, Plus, Number, Minus, Number, Star, Number, Slash, LParen, Number, RParen,
                Eof
            ]
        );
    }

    #[test]
    fn parses_int_and_float_values() {
        let tokens = Lexer::new().tokenize("6 3.14").unwrap();
        assert_eq!(tokens[0].value, Some(NumberValue::Int(6)));
        assert_eq!(tokens[1].value, Some(NumberValue::Float(3.14)));
    }

    #[test]
    fn single_digit_source_produces_number_then_eof() {
        use TokenType::*;
        assert_eq!(token_types("6"), vec![Number, Eof]);
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
