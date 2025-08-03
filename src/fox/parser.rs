use crate::fox::{
    FoxError, FoxResult, Object, TokenType,
    ast::{Expression, Statement},
};

use super::{ErrorKind, Token};

const MAX_FUNCTION_ARGUMENT_COUNT: usize = 255;

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
        if self.matches(&[TokenType::Fun]) {
            return self.function("function");
        }
        if self.matches(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn function(&mut self, kind: &str) -> FoxResult<Statement> {
        let name = self.consume_token(TokenType::Identifier, &format!("Expect {kind} name"))?;
        self.consume_token(
            TokenType::LeftParenthesis,
            &format!("Expect '(' after {kind} name"),
        )?;
        let mut params = Vec::new();

        let mut next_param = !self.check_type(&TokenType::RightParenthesis);
        while next_param {
            if params.len() >= MAX_FUNCTION_ARGUMENT_COUNT {
                return Err(self.error(ErrorKind::TooManyFunctionArguments));
            }
            let param = self.consume_token(TokenType::Identifier, "Expect parameter name")?;
            params.push(param);
            next_param = self.matches(&[TokenType::Comma]);
        }
        self.consume_token(TokenType::RightParenthesis, "Expect ')' after parameters")?;

        self.consume_token(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {kind} body"),
        )?;

        let body = self.block()?;

        Ok(Statement::function(name, params, body))
    }

    fn var_declaration(&mut self) -> FoxResult<Statement> {
        let name = self.consume_token(TokenType::Identifier, "Expect variable name")?;

        let initializer = if self.matches(&[TokenType::Equal]) {
            let expr = self.expression()?;
            Some(Box::new(expr))
        } else {
            None
        };

        self.consume_token(
            TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(Statement::var(name, initializer))
    }

    fn statement(&mut self) -> FoxResult<Statement> {
        if self.matches(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.matches(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.matches(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.matches(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.matches(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.matches(&[TokenType::LeftBrace]) {
            let statements = self.block()?;
            return Ok(Statement::block(statements));
        }
        self.expression_statement()
    }

    fn return_statement(&mut self) -> FoxResult<Statement> {
        let keyword = self.force_previous_token()?;
        let mut value = None;
        if !self.check_type(&TokenType::Semicolon) {
            let expr = self.expression()?;
            value = Some(Box::new(expr));
        }
        self.consume_token(TokenType::Semicolon, "Expect ';' after return value")?;
        Ok(Statement::ret_fn(keyword, value))
    }

    fn for_statement(&mut self) -> FoxResult<Statement> {
        self.consume_token(TokenType::LeftParenthesis, "Expect '(' after 'for'")?;
        let initializer = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check_type(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume_token(TokenType::Semicolon, "Expected ';' after loop condition")?;

        let increment = if self.check_type(&TokenType::RightParenthesis) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume_token(
            TokenType::RightParenthesis,
            "Expected ')' after for clauses",
        )?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Statement::block(vec![body, Statement::expression(Box::new(increment))]);
        }

        if let Some(condition) = condition {
            body = Statement::while_stmt(Box::new(condition), Box::new(body));
        }

        if let Some(initializer) = initializer {
            body = Statement::block(vec![initializer, body])
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> FoxResult<Statement> {
        self.consume_token(TokenType::LeftParenthesis, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume_token(TokenType::RightParenthesis, "Expected ')' after condition")?;
        let body = self.statement()?;

        Ok(Statement::while_stmt(Box::new(condition), Box::new(body)))
    }

    fn if_statement(&mut self) -> FoxResult<Statement> {
        self.consume_token(TokenType::LeftParenthesis, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume_token(TokenType::RightParenthesis, "Expect ')' after if condition")?;

        let then_branch = self.statement()?;

        // I don't like this direct wrapping but it's a shortest code
        let else_branch = if self.matches(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Statement::if_stmt(
            Box::new(condition),
            Box::new(then_branch),
            else_branch,
        ))
    }

    fn block(&mut self) -> FoxResult<Vec<Statement>> {
        let mut statements = Vec::new();

        while !self.check_type(&TokenType::RightBrace) && !self.is_at_end() {
            let stmt = self.declaration()?;
            statements.push(stmt);
        }

        self.consume_token(TokenType::RightBrace, "Expected '}'")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> FoxResult<Statement> {
        let expr = self.expression()?;
        self.consume_token(TokenType::Semicolon, "Expected ';' after value")?;
        Ok(Statement::print(Box::new(expr)))
    }

    fn expression_statement(&mut self) -> FoxResult<Statement> {
        let expr = self.expression()?;
        self.consume_token(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::expression(Box::new(expr)))
    }

    fn expression(&mut self) -> FoxResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> FoxResult<Expression> {
        let expr = self.or()?;
        if !self.matches(&[TokenType::Equal]) {
            return Ok(expr);
        }

        let equals = self.force_previous_token()?;
        let value = self.assignment()?;

        match expr {
            Expression::Variable(data) => {
                let name = data.name;
                Ok(Expression::assign(name, Box::new(value)))
            }
            _ => {
                let err = FoxError::token(ErrorKind::InvalidAssignmentTarget, Some(equals));
                Err(err)
            }
        }
    }

    fn or(&mut self) -> FoxResult<Expression> {
        let mut expr = self.and()?;

        while self.matches(&[TokenType::Or]) {
            let operator = self.force_previous_token()?;
            let right = self.and()?;
            expr = Expression::logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> FoxResult<Expression> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::And]) {
            let operator = self.force_previous_token()?;
            let right = self.equality()?;
            expr = Expression::logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
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

        self.call()
    }

    fn call(&mut self) -> FoxResult<Expression> {
        let mut expr = self.primary()?;
        while self.matches(&[TokenType::LeftParenthesis]) {
            expr = self.finish_call(expr)?;
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> FoxResult<Expression> {
        let mut args = Vec::new();
        if !self.check_type(&TokenType::RightParenthesis) {
            loop {
                if args.len() >= MAX_FUNCTION_ARGUMENT_COUNT {
                    return Err(self.error(ErrorKind::TooManyFunctionArguments));
                }
                let expr = self.expression()?;
                args.push(expr);
                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren =
            self.consume_token(TokenType::RightParenthesis, "Expected ')' after arguments")?;

        Ok(Expression::call(Box::new(callee), paren, args))
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
            self.consume_token(TokenType::RightParenthesis, "Expected ')'")?;
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

    fn consume_token(&mut self, t_type: TokenType, message: &str) -> Result<Token, FoxError> {
        let token = if self.check_type(&t_type) {
            self.advance()
        } else {
            None
        };
        let Some(token) = token else {
            let kind = ErrorKind::Parse(message.to_string());
            let error = self.error(kind);
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
