use crate::fox::{FoxError, Object, TokenType, expression::Expression};

use super::{ErrorKind, Token};

pub struct Parser<'l> {
    tokens: &'l [Token],
    current: usize,
}

impl<'l> Parser<'l> {
    pub fn new(tokens: &'l [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expression, FoxError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expression, FoxError> {
        self.equality()
    }

    fn parse_binary<T>(
        &mut self,
        advance_expr: T,
        token_types: &[TokenType],
    ) -> Result<Expression, FoxError>
    where
        T: Fn(&mut Self) -> Result<Expression, FoxError>,
    {
        let mut expr = advance_expr(self)?;

        while self.matches(token_types) {
            let operator = self.force_previous_token()?;
            let right = advance_expr(self)?;
            expr = Expression::binary(Box::new(expr), operator, Box::new(right))
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression, FoxError> {
        use TokenType::*;
        self.parse_binary(Self::comparison, &[Bang, Equal])
    }

    fn comparison(&mut self) -> Result<Expression, FoxError> {
        use TokenType::*;
        self.parse_binary(Self::term, &[Greater, GreaterEqual, Less, LessEqual])
    }

    fn term(&mut self) -> Result<Expression, FoxError> {
        use TokenType::*;
        self.parse_binary(Self::factor, &[Minus, Plus])
    }

    fn factor(&mut self) -> Result<Expression, FoxError> {
        use TokenType::*;
        self.parse_binary(Self::unary, &[Slash, Star])
    }

    fn unary(&mut self) -> Result<Expression, FoxError> {
        use TokenType::*;
        if self.matches(&[Bang, Minus]) {
            let operator = self.force_previous_token()?;
            let right = self.unary()?;
            return Ok(Expression::unary(Box::new(right), operator));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, FoxError> {
        use TokenType::*;
        if self.matches(&[False]) {
            return Ok(Expression::literal(Object::Bool(false)));
        }
        if self.matches(&[True]) {
            return Ok(Expression::literal(Object::Bool(true)));
        }
        if self.matches(&[Nil]) {
            return Ok(Expression::literal(Object::Nil));
        }

        if self.matches(&[Number, String]) {
            let prev = self.force_previous_token()?;
            return Ok(Expression::literal(prev.literal));
        }

        if self.matches(&[LeftParenthesis]) {
            let expr = self.expression()?;
            self.consume_token(
                TokenType::RightParenthesis,
                ErrorKind::RightParenthesisExpected,
            )?;
            return Ok(Expression::grouping(Box::new(expr)));
        }
        Err(self.error(ErrorKind::ExpressionExpected))
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        for t_type in types {
            if self.check_type(t_type) {
                _ = self.advance();
                return true;
            }
        }
        false
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.current).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let value = self.peek();
        if value.is_some() {
            self.current += 1;
        }
        value
    }

    fn previous_token(&self) -> Option<Token> {
        if self.current == 0 {
            return None;
        }
        self.tokens.get(self.current - 1).cloned()
    }

    fn force_previous_token(&self) -> Result<Token, FoxError> {
        let Some(token) = self.previous_token() else {
            // TODO: provide code info from current token
            return Err(FoxError::error(ErrorKind::ExpectedOperator));
        };
        Ok(token)
    }

    fn consume_token(
        &mut self,
        t_type: TokenType,
        error_kind: ErrorKind,
    ) -> Result<Token, FoxError> {
        let token = if self.check_type(&t_type) {
            self.advance()
        } else {
            None
        };
        let Some(token) = token else {
            let error = self.error(error_kind);
            return Err(error);
        };
        Ok(token)
    }

    fn synchronize(&mut self) -> Result<(), FoxError> {
        self.advance();

        while let Some(current) = self.peek() {
            use TokenType::*;
            if self
                .previous_token()
                .map(|token| token.token_type == Semicolon)
                .unwrap_or(false)
            {
                break;
            }

            if matches!(
                current.token_type,
                Class | Fun | Var | For | If | While | Print | Return
            ) {
                break;
            }

            self.advance();
        }
        Ok(())
    }

    fn error(&self, error_kind: ErrorKind) -> FoxError {
        if let Some(token) = self.peek() {
            FoxError::code(error_kind, token.code_location)
        } else {
            FoxError::error(error_kind)
        }
    }

    fn check_type(&self, tt: &TokenType) -> bool {
        let Some(value) = self.peek() else {
            return false;
        };
        value.token_type == *tt
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
