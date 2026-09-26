use crate::models::lexer::{LexError, NumberValue, Token, TokenType};

#[derive(Debug)]
pub struct Lexer {
    chars: Vec<char>,
    position: usize,
    row: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            position: 0,
            row: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
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
                    row: Some(self.row),
                    col: Some(self.col),
                });
            } else if ch == '\n' {
                self.row += 1;
                self.col = 1;
            } else if ch == ';' {
                tokens.push(Token {
                    token_type: TokenType::Semicolon,
                    row: Some(self.row),
                    col: Some(self.col),
                });
            } else {
                return Err(LexError {
                    row: self.row,
                    col: self.col,
                    character: ch,
                });
            }

            self.position += 1;
            self.col += 1;
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
            if !(next.is_whitespace() || Self::operator_token_type(next).is_some() || next == ';') {
                return Err(self.error_at(index));
            }
        }

        let lexeme: String = self.chars[start..index].iter().collect();
        self.position = index;
        self.col = index;

        let value = if lexeme.contains('.') {
            NumberValue::Float(lexeme.parse().map_err(|_| self.error_at(start))?)
        } else {
            NumberValue::Int(lexeme.parse().map_err(|_| self.error_at(start))?)
        };

        Ok(Token {
            token_type: TokenType::Number(value),
            row: Some(self.row),
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
        Lexer::new(source)
            .tokenize()
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
        let tokens = Lexer::new("6 3.14").tokenize().unwrap();
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
        let err = Lexer::new("asd").tokenize().unwrap_err();
        assert_eq!(err.col, 0);
        assert_eq!(err.character, 'a');
    }

    #[test]
    fn number_with_trailing_garbage_is_an_error_not_a_panic() {
        let err = Lexer::new("123asd").tokenize().unwrap_err();
        assert_eq!(err.col, 3);
        assert_eq!(err.character, 'a');
    }

    #[test]
    fn error_location_is_correct_even_after_a_valid_decimal_part() {
        let err = Lexer::new("123.45asd").tokenize().unwrap_err();
        assert_eq!(err.col, 6);
        assert_eq!(err.character, 'a');
    }

    #[test]
    fn trailing_dot_with_no_fraction_is_an_error() {
        let err = Lexer::new("1.").tokenize().unwrap_err();
        assert_eq!(err.col, 1);
        assert_eq!(err.character, '.');
    }
}
