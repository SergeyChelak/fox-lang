use std::collections::HashMap;

use crate::fox::{
    ErrorKind, FoxError, FoxResult, Object, TokenType,
    ast::*,
    environment::{Environment, SharedEnvironmentPtr},
    token::{ClassInstance, Func, MetaClass, Token},
};

pub struct Interpreter {
    environment: SharedEnvironmentPtr,
    globals: SharedEnvironmentPtr,
    locals: HashMap<Expression, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();
        // register builtin functions
        env.define("clock", Object::Callee(Func::clock()));
        let ptr = env.shared_ptr();

        Self {
            environment: ptr.clone(),
            globals: ptr,
            locals: HashMap::new(),
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

    fn execute_block(&mut self, statements: &[Statement], env: Environment) -> FoxResult<()> {
        let prev = self.environment.clone();

        self.environment = env.shared_ptr();

        // emulate the throw behavior
        let mut result: FoxResult<()> = FoxResult::Ok(());
        for stmt in statements {
            result = self.execute(stmt);
            if result.is_err() {
                break;
            }
        }

        self.environment = prev;
        result
    }

    fn func_call(&mut self, token: &Token, func: &Func, args: &[Object]) -> FoxResult<Object> {
        let arity = func.arity();
        if args.len() != arity {
            let msg = format!("Expected {}  arguments but got {}", arity, args.len());
            return Err(FoxError::runtime(Some(token.clone()), &msg));
        }

        match func {
            Func::Builtin { body, .. } => Ok(body(args)),
            Func::Declaration { decl, closure } => self.execute_func(decl, closure.clone(), args),
        }
    }

    fn execute_func(
        &mut self,
        func: &FunctionStmt,
        closure: SharedEnvironmentPtr,
        args: &[Object],
    ) -> FoxResult<Object> {
        let mut env = Environment::with(Some(closure));

        func.params
            .iter()
            .zip(args.iter())
            .for_each(|(token, object)| {
                env.define(&token.lexeme, object.clone());
            });

        let result = self.execute_block(&func.body, env);
        if let Err(err) = result {
            return match err.kind() {
                ErrorKind::Return(value) => Ok(value.clone()),
                _ => Err(err),
            };
        }
        Ok(Object::Nil)
    }

    pub fn resolve(&mut self, expr: Expression, depth: usize) -> FoxResult<()> {
        self.locals.insert(expr, depth);
        Ok(())
    }

    fn look_up_variable(&self, name: &Token, expr: Expression) -> FoxResult<Object> {
        if let Some(distance) = self.locals.get(&expr) {
            self.environment.borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals.borrow().get(name)
        }
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
        let expr = Expression::Variable(data.clone());
        self.look_up_variable(&data.name, expr)
    }

    fn visit_assign(&mut self, data: &AssignExpr) -> FoxResult<Object> {
        let value = self.evaluate(&data.value)?;
        let expr = Expression::Assign(data.clone());
        if let Some(distance) = self.locals.get(&expr) {
            self.environment
                .borrow_mut()
                .assign_at(*distance, &data.name, value.clone())?;
        } else {
            self.globals
                .borrow_mut()
                .assign(&data.name, value.clone())?;
        }
        Ok(value)
    }

    fn visit_logical(&mut self, data: &LogicalExpr) -> FoxResult<Object> {
        let left = self.evaluate(&data.left)?;

        match data.operator.token_type {
            TokenType::Or if left.is_true() => Ok(left),
            TokenType::And if !left.is_true() => Ok(left),
            _ => self.evaluate(&data.right),
        }
    }

    fn visit_call(&mut self, data: &CallExpr) -> FoxResult<Object> {
        let eval = self.evaluate(&data.callee)?;
        match eval {
            Object::Callee(func) => {
                let mut args = Vec::new();
                for arg in &data.arguments {
                    let expr = self.evaluate(arg)?;
                    args.push(expr);
                }
                self.func_call(&data.paren, &func, &args)
            }
            Object::Class(meta) => {
                let obj = ClassInstance::new(meta.clone());
                Ok(Object::Instance(obj))
            }
            _ => Err(FoxError::runtime(
                Some(data.paren.clone()),
                "Can only call functions and classes",
            )),
        }
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
        let value = if let Some(init) = &data.initializer {
            self.evaluate(init)?
        } else {
            Object::Nil
        };

        self.environment
            .borrow_mut()
            .define(&data.name.lexeme, value);
        Ok(())
    }

    fn visit_block(&mut self, data: &BlockStmt) -> FoxResult<()> {
        let env = Environment::with(Some(self.environment.clone()));
        self.execute_block(&data.statements, env)
    }

    fn visit_if(&mut self, data: &IfStmt) -> FoxResult<()> {
        if self.evaluate(&data.condition)?.is_true() {
            self.execute(&data.then_branch)
        } else if let Some(else_branch) = &data.else_branch {
            self.execute(else_branch)
        } else {
            Ok(())
        }
    }

    fn visit_while(&mut self, data: &WhileStmt) -> FoxResult<()> {
        while self.evaluate(&data.condition)?.is_true() {
            self.execute(&data.body)?;
        }
        Ok(())
    }

    fn visit_function(&mut self, data: &FunctionStmt) -> FoxResult<()> {
        let object = Func::Declaration {
            decl: Box::new(data.clone()),
            closure: self.environment.clone(),
        };
        self.environment
            .borrow_mut()
            .define(&data.name.lexeme, Object::Callee(object));
        Ok(())
    }

    fn visit_return(&mut self, data: &ReturnStmt) -> FoxResult<()> {
        let value = if let Some(val) = &data.value {
            self.evaluate(val)?
        } else {
            Object::Nil
        };
        Err(FoxError::error(ErrorKind::Return(value)))
    }

    fn visit_class(&mut self, data: &ClassStmt) -> FoxResult<()> {
        self.environment
            .borrow_mut()
            .define(&data.name.lexeme, Object::Nil);
        let class_data = MetaClass::new(&data.name.lexeme);
        let class = Object::Class(std::rc::Rc::new(class_data));
        self.environment.borrow_mut().assign(&data.name, class)
    }
}

#[cfg(test)]
mod test {
    //
}
