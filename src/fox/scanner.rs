use super::{CodeLocation, ErrorKind, FoxError, Object, Source, Token, TokenType};

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

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, FoxError> {
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

    fn scan_next(&mut self) -> Result<ScanData, FoxError> {
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
                    self.advance_to_eol();
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
            '\"' => self.scan_string()?,
            ch if ch.is_ascii_digit() => self.scan_number()?,
            ch if ch.is_ascii_alphabetic() => self.scan_identifier()?,
            _ => {
                return Err(self.error(ErrorKind::UnexpectedCharacter));
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
        if Some(value) == self.peek() {
            self.current += 1;
            return true;
        };
        false
    }

    fn advance_to_eol(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            _ = self.advance();
        }
    }

    fn scan_string(&mut self) -> Result<ScanData, FoxError> {
        loop {
            let Some(ch) = self.advance() else {
                return Err(self.error(ErrorKind::UnterminatedString));
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

    fn scan_number(&mut self) -> Result<ScanData, FoxError> {
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
            .map_err(|_| self.error(ErrorKind::UnexpectedCharacter))?;
        let data = self.scan_data_by_type_literal(TokenType::Number, Object::Double(double));
        Ok(data)
    }

    fn scan_identifier(&mut self) -> Result<ScanData, FoxError> {
        while is_alphanumeric(self.peek()) {
            _ = self.advance();
        }
        let value = self.substring(self.start, self.current);
        use TokenType::*;
        let t_type = match value.as_str() {
            "and" => And,
            "class" => Class,
            "else" => Else,
            "false" => False,
            "for" => For,
            "fun" => Fun,
            "if" => If,
            "nil" => Nil,
            "or" => Or,
            "print" => Print,
            "return" => Return,
            "super" => Super,
            "this" => This,
            "true" => True,
            "var" => Var,
            "while" => While,
            _ => Identifier,
        };
        let data = self.scan_data_by_type(t_type);
        Ok(data)
    }

    fn scan_data_by_type(&self, token_type: TokenType) -> ScanData {
        self.scan_data_by_type_literal(token_type, Object::Empty)
    }

    fn scan_data_by_type_literal(&self, token_type: TokenType, literal: Object) -> ScanData {
        ScanData::Token(self.token_with_literal(token_type, literal))
    }

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

    fn error(&self, error_kind: ErrorKind) -> FoxError {
        FoxError::code(error_kind, self.code_location())
    }

    fn code_location(&self) -> CodeLocation {
        CodeLocation::new(self.line, self.current)
    }

    fn substring(&self, start: usize, end: usize) -> String {
        self.source[start..end].iter().collect::<String>()
    }
}

fn is_digit(value: Option<char>) -> bool {
    is_matches_criteria(value, |ch| ch.is_ascii_digit())
}

fn is_alphanumeric(value: Option<char>) -> bool {
    Some('_') == value || is_matches_criteria(value, |ch| ch.is_ascii_alphanumeric())
}

fn is_matches_criteria<F>(value: Option<char>, criteria: F) -> bool
where
    F: Fn(char) -> bool,
{
    value.map(criteria).unwrap_or(false)
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
        let result = scanner.scan_tokens().unwrap();
        use TokenType::*;
        let expected = [LeftParenthesis, RightParenthesis, Eof];
        assert!(is_token_types_matches(&result, &expected));
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
    fn test_not_terminated_string_parse() {
        let input = "\"ABCDEF".chars().collect::<Vec<_>>();
        let mut scanner = Scanner::with_source(&input);
        let result = scanner.scan_tokens();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err.kind(), ErrorKind::UnterminatedString));
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

    #[test]
    fn test_token_parse() {
        let input = "(){},.+-;*!!===<<=>>=/".chars().collect::<Vec<_>>();
        let mut scanner = Scanner::with_source(&input);
        let result = scanner.scan_tokens().unwrap();
        use TokenType::*;
        let expected = [
            LeftParenthesis,
            RightParenthesis,
            LeftBrace,
            RightBrace,
            Comma,
            Dot,
            Plus,
            Minus,
            Semicolon,
            Star,
            Bang,
            BangEqual,
            EqualEqual,
            Less,
            LessEqual,
            Greater,
            GreaterEqual,
            Slash,
            Eof,
        ];
        assert!(is_token_types_matches(&result, &expected));
    }

    #[test]
    fn test_identifier_parse() {
        let input =
            "and class else false for fun if nil or print return super this true var while aa_aa bbb"
                .chars()
                .collect::<Vec<_>>();
        let mut scanner = Scanner::with_source(&input);
        let result = scanner.scan_tokens().unwrap();
        use TokenType::*;
        let expected = [
            And, Class, Else, False, For, Fun, If, Nil, Or, Print, Return, Super, This, True, Var,
            While, Identifier, Identifier, Eof,
        ];
        assert!(is_token_types_matches(&result, &expected));
    }

    /// utils
    fn is_token_types_matches(tokens: &[Token], t_types: &[TokenType]) -> bool {
        tokens
            .iter()
            .zip(t_types.iter())
            .all(|(token, tt)| variant_eq(token.token_type, *tt))
    }

    fn variant_eq<T>(first: T, second: T) -> bool {
        std::mem::discriminant(&first) == std::mem::discriminant(&second)
    }
}
