use std::collections::HashMap;

use crate::fox::{FoxResult, ast::*, interpreter::Interpreter};

type Scope = HashMap<String, bool>;

pub struct Resolver<'l> {
    interpreter: &'l Interpreter,
    scopes: Vec<Scope>,
}

impl<'l> Resolver<'l> {
    pub fn with(interpreter: &'l mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Default::default(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) {
        _ = self.scopes.pop();
    }

    fn resolve_statements(&mut self, statements: &[Statement]) -> FoxResult<()> {
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
}

impl<'l> ExpressionVisitor<()> for Resolver<'l> {
    fn visit_assign(&mut self, data: &AssignExpr) -> FoxResult<()> {
        todo!()
    }

    fn visit_binary(&mut self, data: &BinaryExpr) -> FoxResult<()> {
        todo!()
    }

    fn visit_call(&mut self, data: &CallExpr) -> FoxResult<()> {
        todo!()
    }

    fn visit_grouping(&mut self, data: &GroupingExpr) -> FoxResult<()> {
        todo!()
    }

    fn visit_literal(&mut self, data: &LiteralExpr) -> FoxResult<()> {
        todo!()
    }

    fn visit_logical(&mut self, data: &LogicalExpr) -> FoxResult<()> {
        todo!()
    }

    fn visit_unary(&mut self, data: &UnaryExpr) -> FoxResult<()> {
        todo!()
    }

    fn visit_variable(&mut self, data: &VariableExpr) -> FoxResult<()> {
        todo!()
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
        todo!()
    }

    fn visit_function(&mut self, data: &FunctionStmt) -> FoxResult<()> {
        todo!()
    }

    fn visit_if(&mut self, data: &IfStmt) -> FoxResult<()> {
        todo!()
    }

    fn visit_print(&mut self, data: &PrintStmt) -> FoxResult<()> {
        todo!()
    }

    fn visit_return(&mut self, data: &ReturnStmt) -> FoxResult<()> {
        todo!()
    }

    fn visit_var(&mut self, data: &VarStmt) -> FoxResult<()> {
        todo!()
    }

    fn visit_while(&mut self, data: &WhileStmt) -> FoxResult<()> {
        todo!()
    }
}
