use crate::fox::{ErrorKind, FoxError, FoxResult, Object, TokenType, ast::*};

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(&self, statements: &[Statement]) -> FoxResult<()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&self, stmt: &Statement) -> FoxResult<()> {
        stmt.accept(self)
    }

    fn evaluate(&self, expr: &Expression) -> FoxResult<Object> {
        expr.accept(self)
    }
}

impl ExpressionVisitor<Object> for Interpreter {
    fn visit_binary(&self, data: &BinaryData) -> FoxResult<Object> {
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

    fn visit_grouping(&self, data: &GroupingData) -> FoxResult<Object> {
        self.evaluate(&data.expression)
    }

    fn visit_literal(&self, data: &LiteralData) -> FoxResult<Object> {
        Ok(data.value.clone())
    }

    fn visit_unary(&self, data: &UnaryData) -> FoxResult<Object> {
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
}

impl StatementVisitor<()> for Interpreter {
    fn visit_expression(&self, data: &ExpressionData) -> FoxResult<()> {
        self.evaluate(&data.expression)?;
        Ok(())
    }

    fn visit_print(&self, data: &PrintData) -> FoxResult<()> {
        let value = self.evaluate(&data.expression)?;
        println!("{value}");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    //
}
