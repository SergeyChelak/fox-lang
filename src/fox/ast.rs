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
        Binary(
            BinaryData {
                left: Box<Expression>,
                operator: Token,
                right: Box<Expression>
            }
        ) init: binary, visit: visit_binary,

        Grouping(
            GroupingData {
                expression: Box<Expression>
            }
        ) init: grouping, visit: visit_grouping,

        Literal(
            LiteralData {
                value: Object
            }
        ) init: literal, visit: visit_literal,

        Unary(UnaryData {
                expression: Box<Expression>,
                operator: Token
            }
        ) init: unary, visit: visit_unary,

        Variable(
            VariableData {
                name: Token
            }
        ) init: variable, visit: visit_variable,
    }
);

define_ast!(
    Statement accepting StatementVisitor {
        Expression(
            ExpressionData {
                expression: Box<Expression>
            }
        ) init: expression, visit: visit_expression,

        Print(
            PrintData {
                expression: Box<Expression>
            }
        ) init: print, visit: visit_print,

        Var(
            VarData {
                name: Token,
                initializer: Box<Expression>,
            }
        ) init: var, visit: visit_var,
    }
);

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &Expression) -> FoxResult<String> {
        expr.accept(self)
    }

    fn parenthesize(&mut self, name: &str, expressions: &[&Expression]) -> FoxResult<String> {
        let mut result = format!("({name}");

        for expr in expressions {
            let value = expr.accept(self)?;
            result.push(' ');
            result.push_str(&value);
        }

        result.push(')');
        Ok(result)
    }
}

impl ExpressionVisitor<String> for AstPrinter {
    fn visit_binary(&mut self, data: &BinaryData) -> FoxResult<String> {
        let exprs = [data.left.as_ref(), data.right.as_ref()];
        self.parenthesize(data.operator.lexeme.as_str(), &exprs)
    }

    fn visit_grouping(&mut self, data: &GroupingData) -> FoxResult<String> {
        self.parenthesize("group", &[&data.expression])
    }

    fn visit_literal(&mut self, data: &LiteralData) -> FoxResult<String> {
        Ok(format!("{}", data.value))
    }

    fn visit_unary(&mut self, data: &UnaryData) -> FoxResult<String> {
        self.parenthesize(data.operator.lexeme.as_str(), &[&data.expression])
    }

    fn visit_variable(&mut self, _data: &VariableData) -> FoxResult<String> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fox::TokenType;

    #[test]
    fn test_ast_printer() {
        let expr = Expression::binary(
            Box::new(Expression::unary(
                Box::new(Expression::literal(Object::Double(123.0))),
                Token::new(TokenType::Minus, "-", Object::Nil, Default::default()),
            )),
            Token::new(TokenType::Star, "*", Object::Nil, Default::default()),
            Box::new(Expression::grouping(Box::new(Expression::literal(
                Object::Double(45.67),
            )))),
        );

        let value = AstPrinter.print(&expr).unwrap();
        assert_eq!("(* (- 123) (group 45.67))", value);
    }
}
