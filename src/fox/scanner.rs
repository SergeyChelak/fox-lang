use super::{CodeLocation, Error, ErrorKind, Object, Source, Token, TokenType};

pub struct Scanner<'l> {
    start: usize,
    current: usize,
    line: usize,
    source: &'l Source,
}

enum ScanData {
    Skip,
    Token(Token),
}

impl<'l> Scanner<'l> {
    pub fn with_source(source: &'l Source) -> Self {
        Self {
            start: 0,
            current: 0,
            line: 1,
            source,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::<Token>::new();
        let mut is_eof = false;
        while !is_eof {
            self.start = self.current;
            let data = self.scan_next()?;
            match data {
                ScanData::Token(token) => {
                    is_eof = token.is_eof();
                    tokens.push(token)
                }
                _ => { // no op
                }
            }
        }
        Ok(tokens)
    }

    fn scan_next(&mut self) -> Result<ScanData, Error> {
        let Some(ch) = self.advance() else {
            return Ok(self.scan_data_by_type(Eof));
        };

        use TokenType::*;
        let data = match ch {
            '(' => self.scan_data_by_type(LeftParenthesis),
            ')' => self.scan_data_by_type(RightParenthesis),
            '{' => self.scan_data_by_type(LeftBrace),
            '}' => self.scan_data_by_type(RightBrace),
            ',' => self.scan_data_by_type(Comma),
            '.' => self.scan_data_by_type(Dot),
            '-' => self.scan_data_by_type(Minus),
            '+' => self.scan_data_by_type(Plus),
            ';' => self.scan_data_by_type(Semicolon),
            '*' => self.scan_data_by_type(Star),
            '!' => {
                let t_type = if self.matches('=') { BangEqual } else { Bang };
                self.scan_data_by_type(t_type)
            }
            '=' => {
                let t_type = if self.matches('=') { EqualEqual } else { Equal };
                self.scan_data_by_type(t_type)
            }
            '<' => {
                let t_type = if self.matches('=') { LessEqual } else { Less };
                self.scan_data_by_type(t_type)
            }
            '>' => {
                let t_type = if self.matches('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.scan_data_by_type(t_type)
            }
            '/' => {
                if self.matches('/') {
                    self.skip_to_eol();
                    ScanData::Skip
                } else {
                    self.scan_data_by_type(Slash)
                }
            }
            ' ' | '\r' | '\t' => ScanData::Skip,
            '\n' => {
                self.line += 1;
                ScanData::Skip
            }
            '\"' => self.advance_string()?,
            '0'..='9' => self.advance_number()?,
            _ => {
                return Err(Error::new(
                    ErrorKind::UnexpectedCharacter,
                    Some(self.code_location()),
                ));
            }
        };
        Ok(data)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.current).cloned()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.current + 1).cloned()
    }

    fn advance(&mut self) -> Option<char> {
        let value = self.peek();
        self.current += 1;
        value
    }

    fn matches(&mut self, value: char) -> bool {
        let Some(ch) = self.peek() else {
            return false;
        };

        if ch != value {
            return false;
        }
        self.current += 1;
        true
    }

    fn skip_to_eol(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            _ = self.advance();
        }
    }

    fn advance_string(&mut self) -> Result<ScanData, Error> {
        loop {
            let Some(ch) = self.advance() else {
                return Err(Error::new(
                    ErrorKind::UnterminatedString,
                    Some(self.code_location()),
                ));
            };
            if ch == '\n' {
                self.line += 1;
            }

            if ch == '\"' {
                let value = self.substring(self.start + 1, self.current - 1);
                let data = self.scan_data_by_type_literal(TokenType::String, Object::String(value));
                break Ok(data);
            }
        }
    }

    fn advance_number(&mut self) -> Result<ScanData, Error> {
        let is_digit =
            |value: Option<char>| -> bool { value.map(|x| x.is_ascii_digit()).unwrap_or(false) };

        while is_digit(self.peek()) {
            _ = self.advance();
        }

        if self.peek() == Some('.') && is_digit(self.peek_next()) {
            _ = self.advance();
        }

        while is_digit(self.peek()) {
            _ = self.advance();
        }

        let value = self.substring(self.start, self.current);
        let double = value
            .parse::<f32>()
            .map_err(|_| Error::new(ErrorKind::UnexpectedCharacter, Some(self.code_location())))?;
        let data = self.scan_data_by_type_literal(TokenType::Number, Object::Double(double));
        Ok(data)
    }

    // -----
    fn scan_data_by_type(&self, token_type: TokenType) -> ScanData {
        self.scan_data_by_type_literal(token_type, Object::Empty)
    }

    fn scan_data_by_type_literal(&self, token_type: TokenType, literal: Object) -> ScanData {
        ScanData::Token(self.token_with_literal(token_type, literal))
    }

    // fn token(&self, token_type: TokenType) -> Token {
    //     self.token_with_literal(token_type, Object::Empty)
    // }

    fn token_with_literal(&self, token_type: TokenType, literal: Object) -> Token {
        let lexeme = if self.start < self.current {
            "".to_string()
        } else {
            self.substring(self.start, self.current)
        };
        let code_location = self.code_location();
        Token {
            token_type,
            lexeme,
            literal,
            code_location,
        }
    }

    fn code_location(&self) -> CodeLocation {
        CodeLocation::new(self.line, self.current)
    }

    fn substring(&self, start: usize, end: usize) -> String {
        self.source[start..end].iter().collect::<String>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_comment_parse() {
        let input = r"
            (
            // comment
            ) // other comment
        "
        .chars()
        .collect::<Vec<_>>();
        let mut scanner = Scanner::with_source(&input);
        let result = scanner.scan_tokens();
        if let Err(err) = result {
            panic!("Parse error: {:?}", err);
        }
        let tokens = result.unwrap();
        use TokenType::*;
        assert!(variant_eq(tokens[0].token_type, LeftParenthesis));
        assert!(variant_eq(tokens[1].token_type, RightParenthesis));
        assert!(variant_eq(tokens[2].token_type, Eof));
    }

    #[test]
    fn test_string_parse() {
        let input = "\"ABCDEF\"".chars().collect::<Vec<_>>();
        let mut scanner = Scanner::with_source(&input);
        let result = scanner.scan_tokens();
        if let Err(err) = result {
            panic!("Parse error: {:?}", err);
        }
        let token = &result.unwrap()[0];
        let Object::String(value) = &token.literal else {
            panic!("Invalid literal type");
        };
        assert_eq!(*value, "ABCDEF".to_string());
    }

    #[test]
    fn test_int_parse() {
        let input = "123".chars().collect::<Vec<_>>();
        let mut scanner = Scanner::with_source(&input);
        let result = scanner.scan_tokens();
        if let Err(err) = result {
            panic!("Parse error: {:?}", err);
        }
        let token = &result.unwrap()[0];
        let Object::Double(value) = &token.literal else {
            panic!("Invalid literal type");
        };
        assert_eq!(*value, 123.0);
    }

    #[test]
    fn test_double_parse() {
        let input = "123.456".chars().collect::<Vec<_>>();
        let mut scanner = Scanner::with_source(&input);
        let result = scanner.scan_tokens();
        if let Err(err) = result {
            panic!("Parse error: {:?}", err);
        }
        let token = &result.unwrap()[0];
        let Object::Double(value) = &token.literal else {
            panic!("Invalid literal type");
        };
        assert_eq!(*value, 123.456);
    }

    fn variant_eq<T>(first: T, second: T) -> bool {
        std::mem::discriminant(&first) == std::mem::discriminant(&second)
    }
}
