use crate::fox::{
    ErrorKind, FoxError, FoxResult, Object, TokenType, ast::*, environment::Environment,
};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Statement]) -> FoxResult<()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Statement) -> FoxResult<()> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &Expression) -> FoxResult<Object> {
        expr.accept(self)
    }
}

impl ExpressionVisitor<Object> for Interpreter {
    fn visit_binary(&mut self, data: &BinaryExpr) -> FoxResult<Object> {
        let left = self.evaluate(&data.left)?;
        let right = self.evaluate(&data.right)?;
        use Object::*;
        use TokenType::*;
        match (&data.operator.token_type, &left, &right) {
            (Minus, Double(l), Double(r)) => Ok(Object::Double(l - r)),
            (Slash, Double(l), Double(r)) => Ok(Object::Double(l / r)),
            (Star, Double(l), Double(r)) => Ok(Object::Double(l * r)),
            (Plus, Double(l), Double(r)) => Ok(Object::Double(l + r)),
            (Plus, Text(l), Text(r)) => Ok(Object::Text(l.to_owned() + r)),

            (Greater, Double(l), Double(r)) => Ok(Object::Bool(l > r)),
            (GreaterEqual, Double(l), Double(r)) => Ok(Object::Bool(l >= r)),

            (Less, Double(l), Double(r)) => Ok(Object::Bool(l < r)),
            (LessEqual, Double(l), Double(r)) => Ok(Object::Bool(l <= r)),

            (Minus, _, _)
            | (Star, _, _)
            | (Slash, _, _)
            | (Greater, _, _)
            | (GreaterEqual, _, _)
            | (Less, _, _)
            | (LessEqual, _, _) => Err(FoxError::token(
                ErrorKind::OperandMustBeNumber,
                Some(data.operator.clone()),
            )),

            (BangEqual, l, r) => Ok(Object::Bool(l != r)),
            (EqualEqual, l, r) => Ok(Object::Bool(l == r)),

            _ => Err(FoxError::token(
                ErrorKind::OperandsMustBeSameType,
                Some(data.operator.clone()),
            )),
        }
    }

    fn visit_grouping(&mut self, data: &GroupingExpr) -> FoxResult<Object> {
        self.evaluate(&data.expression)
    }

    fn visit_literal(&mut self, data: &LiteralExpr) -> FoxResult<Object> {
        Ok(data.value.clone())
    }

    fn visit_unary(&mut self, data: &UnaryExpr) -> FoxResult<Object> {
        let right = self.evaluate(&data.expression)?;

        use TokenType::*;
        match (&data.operator.token_type, &right) {
            (Minus, Object::Double(value)) => Ok(Object::Double(-value)),
            (Minus, _) => Err(FoxError::token(
                ErrorKind::OperandMustBeNumber,
                Some(data.operator.clone()),
            )),
            (Bang, r) => Ok(Object::Bool(!r.is_true())),
            _ => unreachable!(),
        }
    }

    fn visit_variable(&mut self, data: &VariableExpr) -> FoxResult<Object> {
        self.environment.get(&data.name)
    }

    fn visit_assign(&mut self, data: &AssignExpr) -> FoxResult<Object> {
        let value = self.evaluate(&data.value)?;
        self.environment.assign(&data.name, value.clone())?;
        Ok(value)
    }
}

impl StatementVisitor<()> for Interpreter {
    fn visit_expression(&mut self, data: &ExpressionStmt) -> FoxResult<()> {
        self.evaluate(&data.expression)?;
        Ok(())
    }

    fn visit_print(&mut self, data: &PrintStmt) -> FoxResult<()> {
        let value = self.evaluate(&data.expression)?;
        println!("{value}");
        Ok(())
    }

    fn visit_var(&mut self, data: &VarStmt) -> FoxResult<()> {
        let value = self.evaluate(&data.initializer)?;
        self.environment.define(&data.name.lexeme, value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    //
}
