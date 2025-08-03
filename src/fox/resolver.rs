use std::collections::HashMap;

use crate::fox::{FoxError, FoxResult, ast::*, interpreter::Interpreter, token::Token};

type Scope = HashMap<String, bool>;

#[derive(Clone, Copy)]
enum FuncType {
    None,
    Func,
}

pub struct Resolver<'l> {
    interpreter: &'l mut Interpreter,
    scopes: Vec<Scope>,
    current_function: FuncType,
}

impl<'l> Resolver<'l> {
    pub fn with(interpreter: &'l mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Default::default(),
            current_function: FuncType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) {
        _ = self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> FoxResult<()> {
        let Some(scope) = self.scopes.last_mut() else {
            return Ok(());
        };
        if scope.contains_key(&name.lexeme) {
            let err = FoxError::resolver(
                Some(name.clone()),
                "Already a variable with this name in this scope",
            );
            return Err(err);
        }
        scope.insert(name.lexeme.clone(), false);
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };
        scope.insert(name.lexeme.clone(), true);
    }

    pub fn resolve_statements(&mut self, statements: &[Statement]) -> FoxResult<()> {
        for stmt in statements {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Statement) -> FoxResult<()> {
        stmt.accept(self)
    }

    fn resolve_expr(&mut self, expr: &Expression) -> FoxResult<()> {
        expr.accept(self)
    }

    fn resolve_local(&mut self, expr: Expression, name: &Token) -> FoxResult<()> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, self.scopes.len() - i - 1)?;
                break;
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, func: &FunctionStmt, func_type: FuncType) -> FoxResult<()> {
        let enclosing_function = self.current_function;
        self.current_function = func_type;
        self.begin_scope();
        for param in &func.params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(&func.body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }
}

impl<'l> ExpressionVisitor<()> for Resolver<'l> {
    fn visit_assign(&mut self, data: &AssignExpr) -> FoxResult<()> {
        self.resolve_expr(&data.value)?;
        let expr = Expression::Assign(data.clone());
        self.resolve_local(expr, &data.name)
    }

    fn visit_binary(&mut self, data: &BinaryExpr) -> FoxResult<()> {
        self.resolve_expr(&data.left)?;
        self.resolve_expr(&data.right)
    }

    fn visit_call(&mut self, data: &CallExpr) -> FoxResult<()> {
        self.resolve_expr(&data.callee)?;
        for arg in &data.arguments {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }

    fn visit_grouping(&mut self, data: &GroupingExpr) -> FoxResult<()> {
        self.resolve_expr(&data.expression)
    }

    fn visit_literal(&mut self, _data: &LiteralExpr) -> FoxResult<()> {
        Ok(())
    }

    fn visit_logical(&mut self, data: &LogicalExpr) -> FoxResult<()> {
        self.resolve_expr(&data.left)?;
        self.resolve_expr(&data.right)
    }

    fn visit_unary(&mut self, data: &UnaryExpr) -> FoxResult<()> {
        self.resolve_expr(&data.expression)
    }

    fn visit_variable(&mut self, data: &VariableExpr) -> FoxResult<()> {
        if Some(&false)
            == self
                .scopes
                .last()
                .and_then(|scope| scope.get(&data.name.lexeme))
        {
            let err = FoxError::resolver(
                Some(data.name.clone()),
                "Can't read local variable in its own initializer",
            );
            return Err(err);
        }
        let expr = Expression::Variable(data.clone());
        self.resolve_local(expr, &data.name)
    }
}

impl<'l> StatementVisitor<()> for Resolver<'l> {
    fn visit_block(&mut self, data: &BlockStmt) -> FoxResult<()> {
        self.begin_scope();
        self.resolve_statements(&data.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_expression(&mut self, data: &ExpressionStmt) -> FoxResult<()> {
        self.resolve_expr(&data.expression)
    }

    fn visit_function(&mut self, data: &FunctionStmt) -> FoxResult<()> {
        self.declare(&data.name)?;
        self.define(&data.name);

        self.resolve_function(data, FuncType::Func)
    }

    fn visit_if(&mut self, data: &IfStmt) -> FoxResult<()> {
        self.resolve_expr(&data.condition)?;
        self.resolve_stmt(&data.then_branch)?;
        if let Some(else_branch) = &data.else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print(&mut self, data: &PrintStmt) -> FoxResult<()> {
        self.resolve_expr(&data.expression)
    }

    fn visit_return(&mut self, data: &ReturnStmt) -> FoxResult<()> {
        if matches!(self.current_function, FuncType::None) {
            let err = FoxError::resolver(
                Some(data.keyword.clone()),
                "Can't return from top-level code",
            );
            return Err(err);
        }
        if let Some(value) = &data.value {
            self.resolve_expr(value)?;
        }
        Ok(())
    }

    fn visit_var(&mut self, data: &VarStmt) -> FoxResult<()> {
        self.declare(&data.name)?;
        if let Some(expr) = &data.initializer {
            self.resolve_expr(expr)?;
        }
        self.define(&data.name);
        Ok(())
    }

    fn visit_while(&mut self, data: &WhileStmt) -> FoxResult<()> {
        self.resolve_expr(&data.condition)?;
        self.resolve_stmt(&data.body)
    }
}
