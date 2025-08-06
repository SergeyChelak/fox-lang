use crate::fox::{FoxResult, Object, Token};

macro_rules! define_ast {
    (
        $holder_type:ident accepting $visitor_type:ident {
            $(
                $option:ident($option_data:ident {
                    $(
                        $p_name:ident: $p_type:ty
                    ),*$(,)?
                }) init: $fn_init:ident,
                   visit: $fn_visit:ident
            ),*
            $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Hash, Eq, PartialEq)]
        pub enum $holder_type {
            $($option($option_data),)*
        }

        impl $holder_type {
            $(
                pub fn $fn_init($($p_name: $p_type,)*) -> Self {
                    Self::$option(
                        $option_data {
                            $($p_name,)*
                        }
                    )
                }
            )*

            pub fn accept<T>(&self, visitor: &mut dyn $visitor_type<T>) -> FoxResult<T> {
                match self {
                    $(
                        Self::$option(data) => data.accept(visitor),
                    )*
                }
            }
        }

        $(
            #[derive(Debug, Clone, Hash, Eq, PartialEq)]
            pub struct $option_data {
                $(pub $p_name: $p_type,)*
            }

            impl $option_data {
                fn accept<T>(&self, visitor: &mut dyn $visitor_type<T>) -> FoxResult<T> {
                    visitor.$fn_visit(self)
                }
            }
        )*

        pub trait $visitor_type<T> {
            $(
                fn $fn_visit(&mut self, data: &$option_data) -> FoxResult<T>;
            )*
        }
    };
}

define_ast!(
    Expression accepting ExpressionVisitor {
        Assign(
            AssignExpr {
                name: Token,
                value: Box<Expression>,
            }
        ) init: assign, visit: visit_assign,

        Binary(
            BinaryExpr {
                left: Box<Expression>,
                operator: Token,
                right: Box<Expression>
            }
        ) init: binary, visit: visit_binary,

        Call(
            CallExpr {
                callee: Box<Expression>,
                paren: Token,
                arguments: Vec<Expression>,
            }
        ) init: call, visit: visit_call,

        Get(
            GetExpr {
                object: Box<Expression>,
                name: Token,
            }
        ) init: get, visit: visit_get,

        Grouping(
            GroupingExpr {
                expression: Box<Expression>
            }
        ) init: grouping, visit: visit_grouping,

        Literal(
            LiteralExpr {
                value: Object
            }
        ) init: literal, visit: visit_literal,

        Logical(
            LogicalExpr {
                left: Box<Expression>,
                operator: Token,
                right: Box<Expression>,
            }
        ) init: logical, visit: visit_logical,

        Unary(UnaryExpr {
                expression: Box<Expression>,
                operator: Token
            }
        ) init: unary, visit: visit_unary,

        Variable(
            VariableExpr {
                name: Token
            }
        ) init: variable, visit: visit_variable,
    }
);

define_ast!(
    Statement accepting StatementVisitor {
        Block(
            BlockStmt {
                statements: Vec<Statement>,
            }
        ) init: block, visit: visit_block,

        Class(
            ClassStmt {
                name: Token,
                methods: Vec<Statement>,
            }
        ) init: class, visit: visit_class,

        Expression(
            ExpressionStmt {
                expression: Box<Expression>
            }
        ) init: expression, visit: visit_expression,

        Function(
            FunctionStmt {
                name: Token,
                params: Vec<Token>,
                body: Vec<Statement>,
            }
        ) init: function, visit: visit_function,

        If(
            IfStmt {
                condition: Box<Expression>,
                then_branch: Box<Statement>,
                else_branch: Option<Box<Statement>>,
            }
        ) init: if_stmt, visit: visit_if,

        Print(
            PrintStmt {
                expression: Box<Expression>
            }
        ) init: print, visit: visit_print,

        Return(
            ReturnStmt {
                keyword: Token,
                value: Option<Box<Expression>>,
            }
        ) init: ret_fn, visit: visit_return,

        Var(
            VarStmt {
                name: Token,
                initializer: Option<Box<Expression>>,
            }
        ) init: var, visit: visit_var,

        While(
            WhileStmt {
                condition: Box<Expression>,
                body: Box<Statement>,
            }
        ) init: while_stmt, visit: visit_while,
    }
);

#[cfg(test)]
mod test {
    //
}
