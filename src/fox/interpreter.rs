use std::{collections::HashMap, rc::Rc};

use crate::fox::{
    ErrorKind, FoxError, FoxResult, KEYWORD_SUPER, KEYWORD_THIS, Object, TokenType,
    ast::*,
    class::{ClassInstance, INITIALIZER_NAME, MetaClass},
    environment::{Environment, SharedEnvironmentPtr},
    func::*,
    token::Token,
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
        env.define("clock", Object::BuiltinCallee(BuiltinFunc::clock()));
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

    fn func_arity_check(&self, token: &Token, arity: usize, args: &[Object]) -> FoxResult<()> {
        if args.len() != arity {
            let msg = format!("Expected {}  arguments but got {}", arity, args.len());
            return Err(FoxError::runtime(Some(token.clone()), &msg));
        }
        Ok(())
    }

    fn func_execute(&mut self, func: &Func, args: &[Object]) -> FoxResult<Object> {
        let mut env = Environment::with(Some(func.closure.clone()));

        func.decl
            .params
            .iter()
            .zip(args.iter())
            .for_each(|(token, object)| {
                env.define(&token.lexeme, object.clone());
            });

        let result = self.execute_block(&func.decl.body, env);
        if let Err(err) = result {
            return match err.kind() {
                ErrorKind::Return(_) if func.is_initializer => {
                    func.closure.borrow().get_at(0, KEYWORD_THIS)
                }
                ErrorKind::Return(value) => Ok(value.clone()),
                _ => Err(err),
            };
        }

        if func.is_initializer {
            return func.closure.borrow().get_at(0, KEYWORD_THIS);
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
        use TokenType::*;
        let result = match (&data.operator.token_type, &left, &right) {
            (Minus, l, r) => l.minus(r),
            (Slash, l, r) => l.divide(r),
            (Star, l, r) => l.multiply(r),
            (Plus, l, r) => l.plus(r),
            (Greater, l, r) => l.greater(r),
            (GreaterEqual, l, r) => l.greater_equal(r),
            (Less, l, r) => l.less(r),
            (LessEqual, l, r) => l.less_equal(r),
            (BangEqual, l, r) => Ok(Object::Bool(l != r)),
            (EqualEqual, l, r) => Ok(Object::Bool(l == r)),
            _ => Err("Type mismatch".to_string()),
        };
        result.map_err(|err| FoxError::runtime(Some(data.operator.clone()), &err))
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
        let mut args = Vec::new();
        for arg in &data.arguments {
            let expr = self.evaluate(arg)?;
            args.push(expr);
        }
        match eval {
            Object::BuiltinCallee(func) => {
                self.func_arity_check(&data.paren, func.arity(), &args)?;
                let value = (func.body)(&args);
                Ok(value)
            }
            Object::Callee(func) => {
                self.func_arity_check(&data.paren, func.arity(), &args)?;
                self.func_execute(&func, &args)
            }
            Object::Class(meta) => {
                self.func_arity_check(&data.paren, meta.arity(), &args)?;
                let constructor = MetaClass::constructor(meta);
                if let Some(func) = constructor.initializer {
                    self.func_execute(&func, &args)?;
                }
                Ok(Object::Instance(constructor.instance))
            }
            _ => Err(FoxError::runtime(
                Some(data.paren.clone()),
                "Can only call functions and classes",
            )),
        }
    }

    fn visit_get(&mut self, data: &GetExpr) -> FoxResult<Object> {
        let object = self.evaluate(&data.object)?;
        let Object::Instance(instance) = object else {
            let err = FoxError::runtime(Some(data.name.clone()), "Only instances have properties");
            return Err(err);
        };
        ClassInstance::get(instance, &data.name)
    }

    fn visit_set(&mut self, data: &SetExpr) -> FoxResult<Object> {
        let object = self.evaluate(&data.object)?;

        match object {
            Object::Instance(instance) => {
                let value = self.evaluate(&data.value)?;
                instance.borrow_mut().set(&data.name, value.clone());
                Ok(value)
            }
            _ => {
                let err = FoxError::runtime(Some(data.name.clone()), "Only instances have fields");
                Err(err)
            }
        }
    }

    fn visit_this(&mut self, data: &ThisExpr) -> FoxResult<Object> {
        let expr = Expression::This(data.clone());
        self.look_up_variable(&data.keyword, expr)
    }

    fn visit_super(&mut self, data: &SuperExpr) -> FoxResult<Object> {
        let expr = Expression::Super(data.clone());
        let Some(distance) = self.locals.get(&expr) else {
            return Err(FoxError::bug("Distance for super must be set"));
        };
        let superclass = self
            .environment
            .borrow()
            .get_at(*distance, KEYWORD_SUPER)?
            .as_meta_class()?;
        let object = self
            .environment
            .borrow()
            .get_at(distance - 1, KEYWORD_THIS)?
            .as_class_instance()?;
        let Some(method) = superclass.find_method(&data.method.lexeme) else {
            return Err(FoxError::runtime(
                Some(data.method.clone()),
                &format!("Undefined property '{}'", data.method.lexeme),
            ));
        };
        let func = method.bind(object);
        Ok(Object::Callee(func))
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
        let object = Func::new(Rc::new(data.clone()), self.environment.clone(), false);
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
        let mut superclass: Option<Rc<MetaClass>> = None;
        if let Some(expr) = &data.superclass {
            let eval = self.evaluate(expr)?;
            match eval {
                Object::Class(val) => {
                    superclass = Some(val.clone());
                }
                _ => {
                    let token = expr.as_variable()?.name.clone();
                    return Err(FoxError::runtime(Some(token), "Superclass must be a class"));
                }
            }
        }

        self.environment
            .borrow_mut()
            .define(&data.name.lexeme, Object::Nil);

        let enclosing = self.environment.clone();
        if let Some(obj) = &superclass {
            self.environment = Environment::with(Some(enclosing.clone())).shared_ptr();
            let value = Object::Class(obj.clone());
            self.environment.borrow_mut().define(KEYWORD_SUPER, value);
        }

        let mut methods = HashMap::new();
        for stmt in &data.methods {
            let func = stmt.as_function()?;
            let method = Func::new(
                Rc::new(func.clone()),
                self.environment.clone(),
                func.name.lexeme == INITIALIZER_NAME,
            );
            methods.insert(func.name.lexeme.clone(), method);
        }
        let class_data = MetaClass::new(&data.name.lexeme, superclass, methods);
        let class = Object::Class(std::rc::Rc::new(class_data));

        if data.superclass.is_some() {
            self.environment = enclosing;
        }

        self.environment.borrow_mut().assign(&data.name, class)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn binary_expr(l: Object, t_type: TokenType, r: Object) -> BinaryExpr {
        let left = Box::new(Expression::literal(l));
        let right = Box::new(Expression::literal(r));
        let operator = Token {
            token_type: t_type,
            lexeme: "Debug".to_string(),
            literal: Object::Nil,
            code_location: Default::default(),
        };
        BinaryExpr {
            left,
            operator,
            right,
        }
    }

    #[test]
    fn test_binary_double_plus() {
        let mut interpreter = Interpreter::new();
        let expr = binary_expr(Object::Double(2.0), TokenType::Plus, Object::Double(2.0));
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Double(4.0));
        // err
        let expr = binary_expr(Object::Double(3.0), TokenType::Plus, Object::Nil);
        let res = interpreter.visit_binary(&expr);
        assert!(res.is_err());
    }

    #[test]
    fn test_binary_string_plus() {
        let mut interpreter = Interpreter::new();
        let expr = binary_expr(
            Object::Text("hello,".to_string()),
            TokenType::Plus,
            Object::Text("fox lang".to_string()),
        );
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Text("hello,fox lang".to_string()));
    }

    #[test]
    fn test_binary_double_minus() {
        let mut interpreter = Interpreter::new();
        let expr = binary_expr(Object::Double(3.0), TokenType::Minus, Object::Double(2.0));
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Double(1.0));
        // err
        let expr = binary_expr(Object::Double(3.0), TokenType::Minus, Object::Nil);
        let res = interpreter.visit_binary(&expr);
        assert!(res.is_err());
    }

    #[test]
    fn test_binary_double_multiply() {
        let mut interpreter = Interpreter::new();
        // ok
        let expr = binary_expr(Object::Double(3.0), TokenType::Star, Object::Double(2.0));
        let result = interpreter.visit_binary(&expr);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj, Object::Double(6.0));
        // err
        let expr = binary_expr(Object::Double(3.0), TokenType::Star, Object::Nil);
        let result = interpreter.visit_binary(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_double_greater() {
        let mut interpreter = Interpreter::new();
        // true
        let expr = binary_expr(Object::Double(3.0), TokenType::Greater, Object::Double(2.0));
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(true));
        // false
        let expr = binary_expr(Object::Double(3.0), TokenType::Greater, Object::Double(3.0));
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(false));
        // false
        let expr = binary_expr(Object::Double(3.0), TokenType::Greater, Object::Double(4.0));
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(false));
        // err
        let expr = binary_expr(Object::Double(3.0), TokenType::Greater, Object::Nil);
        let result = interpreter.visit_binary(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_double_greater_equal() {
        let mut interpreter = Interpreter::new();
        // true
        let expr = binary_expr(
            Object::Double(3.0),
            TokenType::GreaterEqual,
            Object::Double(2.0),
        );
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(true));
        // true
        let expr = binary_expr(
            Object::Double(3.0),
            TokenType::GreaterEqual,
            Object::Double(3.0),
        );
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(true));
        // false
        let expr = binary_expr(
            Object::Double(3.0),
            TokenType::GreaterEqual,
            Object::Double(4.0),
        );
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(false));
        // err
        let expr = binary_expr(Object::Double(3.0), TokenType::GreaterEqual, Object::Nil);
        let result = interpreter.visit_binary(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_double_less() {
        let mut interpreter = Interpreter::new();
        // true
        let expr = binary_expr(Object::Double(3.0), TokenType::Less, Object::Double(4.0));
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(true));
        // false
        let expr = binary_expr(Object::Double(3.0), TokenType::Less, Object::Double(3.0));
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(false));
        // false
        let expr = binary_expr(Object::Double(3.0), TokenType::Less, Object::Double(2.0));
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(false));
        // err
        let expr = binary_expr(Object::Double(3.0), TokenType::Less, Object::Nil);
        let result = interpreter.visit_binary(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_double_less_equal() {
        let mut interpreter = Interpreter::new();
        // true
        let expr = binary_expr(
            Object::Double(3.0),
            TokenType::LessEqual,
            Object::Double(4.0),
        );
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(true));
        // true
        let expr = binary_expr(
            Object::Double(3.0),
            TokenType::LessEqual,
            Object::Double(3.0),
        );
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(true));
        // false
        let expr = binary_expr(
            Object::Double(3.0),
            TokenType::LessEqual,
            Object::Double(2.0),
        );
        let obj = interpreter.visit_binary(&expr).unwrap();
        assert_eq!(obj, Object::Bool(false));
        // err
        let expr = binary_expr(Object::Double(3.0), TokenType::LessEqual, Object::Nil);
        let result = interpreter.visit_binary(&expr);
        assert!(result.is_err());
    }
}
