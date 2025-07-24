use crate::fox::{
    FoxError, FoxResult, Object, TokenType,
    ast::{Expression, Statement},
};

use super::{ErrorKind, Token};

pub struct Parser<'l> {
    tokens: &'l [Token],
    current: usize,
}

impl<'l> Parser<'l> {
    pub fn new(tokens: &'l [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> FoxResult<Vec<Statement>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // match self.declaration() {
            //     Ok(statement) => statements.push(statement),
            //     Err(_) => self.synchronize(),
            // }
            let statement = self.declaration()?;
            statements.push(statement);
        }

        Ok(statements)
    }

    fn is_at_end(&self) -> bool {
        // 1. We're expecting EOF is always last token in array according to design
        // but because of robust reasons we also need to check if we still in token's range
        // 2. Usage of peek() is less effective because it clones token each loop iteration
        let Some(token) = self.tokens.get(self.current) else {
            return true;
        };
        token.is_eof()
    }

    fn declaration(&mut self) -> FoxResult<Statement> {
        if self.matches(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> FoxResult<Statement> {
        let name = self.consume_token(TokenType::Identifier, ErrorKind::ExpectedVariableName)?;

        let initializer = if self.matches(&[TokenType::Equal]) {
            self.expression()?
        } else {
            Expression::literal(Object::Nil)
        };

        self.consume_token(TokenType::Semicolon, ErrorKind::ExpectedSemicolon)?;

        Ok(Statement::var(name, Box::new(initializer)))
    }

    fn statement(&mut self) -> FoxResult<Statement> {
        if self.matches(&[TokenType::Print]) {
            return self.print_statement();
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> FoxResult<Statement> {
        let expr = self.expression()?;
        self.consume_token(TokenType::Semicolon, ErrorKind::ExpectedSemicolon)?;
        Ok(Statement::print(Box::new(expr)))
    }

    fn expression_statement(&mut self) -> FoxResult<Statement> {
        let expr = self.expression()?;
        self.consume_token(TokenType::Semicolon, ErrorKind::ExpectedSemicolon)?;
        Ok(Statement::expression(Box::new(expr)))
    }

    fn expression(&mut self) -> FoxResult<Expression> {
        self.equality()
    }

    fn parse_binary<T>(
        &mut self,
        advance_expr: T,
        token_types: &[TokenType],
    ) -> Result<Expression, FoxError>
    where
        T: Fn(&mut Self) -> FoxResult<Expression>,
    {
        let mut expr = advance_expr(self)?;

        while self.matches(token_types) {
            let operator = self.force_previous_token()?;
            let right = advance_expr(self)?;
            expr = Expression::binary(Box::new(expr), operator, Box::new(right))
        }

        Ok(expr)
    }

    fn equality(&mut self) -> FoxResult<Expression> {
        use TokenType::*;
        self.parse_binary(Self::comparison, &[BangEqual, EqualEqual])
    }

    fn comparison(&mut self) -> FoxResult<Expression> {
        use TokenType::*;
        self.parse_binary(Self::term, &[Greater, GreaterEqual, Less, LessEqual])
    }

    fn term(&mut self) -> FoxResult<Expression> {
        use TokenType::*;
        self.parse_binary(Self::factor, &[Minus, Plus])
    }

    fn factor(&mut self) -> FoxResult<Expression> {
        use TokenType::*;
        self.parse_binary(Self::unary, &[Slash, Star])
    }

    fn unary(&mut self) -> FoxResult<Expression> {
        use TokenType::*;
        if self.matches(&[Bang, Minus]) {
            let operator = self.force_previous_token()?;
            let right = self.unary()?;
            return Ok(Expression::unary(Box::new(right), operator));
        }

        self.primary()
    }

    fn primary(&mut self) -> FoxResult<Expression> {
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

        if self.matches(&[Identifier]) {
            let prev = self.force_previous_token()?;
            let expr = Expression::variable(prev);
            return Ok(expr);
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

    fn force_previous_token(&self) -> FoxResult<Token> {
        let Some(token) = self.previous_token() else {
            return Err(self.error(ErrorKind::ExpectedOperator));
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

    // fn synchronize(&mut self) {
    //     self.advance();

    //     while let Some(current) = self.peek() {
    //         use TokenType::*;
    //         if self
    //             .previous_token()
    //             .map(|token| token.token_type == Semicolon)
    //             .unwrap_or(false)
    //         {
    //             break;
    //         }

    //         if matches!(
    //             current.token_type,
    //             Class | Fun | Var | For | If | While | Print | Return
    //         ) {
    //             break;
    //         }

    //         self.advance();
    //     }
    // }

    fn error(&self, error_kind: ErrorKind) -> FoxError {
        FoxError::token(error_kind, self.previous_token())
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
    // use super::*;
}
