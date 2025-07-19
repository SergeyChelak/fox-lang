use super::{CodeLocation, Error, ErrorKind, Object, Source, Token, TokenType};

enum Scan {
    Token(Token),
    NewLine,
    Comment,
}

struct Context {
    start: usize,
    current: usize,
}

pub struct Scanner<'l> {
    start: usize,
    current: usize,
    line: usize,
    source: &'l Source,
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
            let scan = self.scan()?;
            match scan {
                Scan::Token(token) => {
                    is_eof = token.is_eof();
                    tokens.push(token);
                }
                Scan::NewLine => {
                    self.line += 1;
                }
                _ => {
                    // no op
                }
            };
        }
        Ok(tokens)
    }

    fn current_code_location(&self) -> CodeLocation {
        CodeLocation::new(self.line, self.current)
    }

    fn scan(&mut self) -> Result<Scan, Error> {
        let Some(byte) = self.advance() else {
            return Ok(Scan::Token(self.eof_token()));
        };

        let next = self.source.get(self.current).cloned();
        use TokenType::*;
        let token = match byte {
            b'(' => self.create_token(LeftParenthesis),
            b')' => self.create_token(RightParenthesis),
            b'{' => self.create_token(LeftBrace),
            b'}' => self.create_token(RightBrace),
            b',' => self.create_token(Comma),
            b'.' => self.create_token(Dot),
            b'-' => self.create_token(Minus),
            b'+' => self.create_token(Plus),
            b';' => self.create_token(Semicolon),
            b'*' => self.create_token(Star),
            b'!' if next == Some(b'=') => {
                self.current += 1;
                self.create_token(BangEqual)
            }
            b'!' => self.create_token(Bang),
            b'=' if next == Some(b'=') => {
                self.current += 1;
                self.create_token(EqualEqual)
            }
            b'=' => self.create_token(Equal),
            b'<' if next == Some(b'=') => {
                self.current += 1;
                self.create_token(LessEqual)
            }
            b'<' => self.create_token(Less),
            b'>' if next == Some(b'=') => {
                self.current += 1;
                self.create_token(GreaterEqual)
            }
            b'>' => self.create_token(Greater),
            _ => {
                return Err(Error::new(
                    ErrorKind::UnexpectedCharacter,
                    Some(self.current_code_location()),
                ));
            }
        };
        Ok(Scan::Token(token))
    }

    fn advance(&mut self) -> Option<u8> {
        let value = self.source.get(self.current).cloned();
        self.current += 1;
        value
    }

    // fn matches(&mut self, value: u8) -> bool {
    //     let Some(byte) = self.source.get(self.current) else {
    //         return false;
    //     };
    //     if *byte != value {
    //         return false;
    //     }
    //     self.current += 1;
    //     true
    // }

    fn create_token(&mut self, token_type: TokenType) -> Token {
        self.create_token_with_literal(token_type, Object::Empty)
    }

    fn create_token_with_literal(&self, token_type: TokenType, literal: Object) -> Token {
        let text = self.source[self.start..self.current]
            .iter()
            .map(|b| *b as char)
            .collect::<String>();
        Token {
            token_type,
            lexeme: text,
            literal,
            code_location: self.current_code_location(),
        }
    }

    fn eof_token(&self) -> Token {
        Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: Object::Empty,
            code_location: self.current_code_location(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        //
    }
}
