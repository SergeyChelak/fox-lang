use crate::fox::{FoxError, Object, Token};

macro_rules! ast_expressions {
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
                fn $fn_init($($p_name: $p_type,)*) -> Self {
                    Self::$option(
                        $option_data {
                            $($p_name,)*
                        }
                    )
                }
            )*

            fn accept<T>(&self, visitor: &dyn $visitor_type<T>) -> Result<T, FoxError> {
                match self {
                    $(
                        Self::$option(data) => data.accept(visitor),
                    )*
                }
            }
        }

        $(
            pub struct $option_data {
                $($p_name: $p_type,)*
            }

            impl $option_data {
                fn accept<T>(&self, visitor: &dyn $visitor_type<T>) -> Result<T, FoxError> {
                    visitor.$fn_visit(self)
                }
            }
        )*

        pub trait $visitor_type<T> {
            $(
                fn $fn_visit(&self, data: &$option_data) -> Result<T, FoxError>;
            )*
        }
    };
}

ast_expressions!(
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
        ) init: unary, visit: visit_unary
    }
);

#[cfg(test)]
mod test {
    use crate::fox::TokenType;

    use super::*;

    struct AstPrinter {
        //
    }

    impl AstPrinter {
        fn print(&self, expr: &Expression) -> Result<String, FoxError> {
            expr.accept(self)
        }

        fn parenthesize(
            &self,
            name: &str,
            expressions: &[&Box<Expression>],
        ) -> Result<String, FoxError> {
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
        fn visit_binary(&self, data: &BinaryData) -> Result<String, FoxError> {
            let exprs = [&data.left, &data.right];
            Ok(self.parenthesize(data.operator.lexeme.as_str(), &exprs)?)
        }

        fn visit_grouping(&self, data: &GroupingData) -> Result<String, FoxError> {
            Ok(self.parenthesize("group", &[&data.expression])?)
        }

        fn visit_literal(&self, data: &LiteralData) -> Result<String, FoxError> {
            Ok(format!("{}", data.value))
        }

        fn visit_unary(&self, data: &UnaryData) -> Result<String, FoxError> {
            Ok(self.parenthesize(data.operator.lexeme.as_str(), &[&data.expression])?)
        }
    }

    #[test]
    fn test_ast_printer() {
        let expr = Expression::binary(
            Box::new(Expression::unary(
                Box::new(Expression::literal(Object::Double(123.0))),
                Token::new(TokenType::Minus, "-", Object::Empty, Default::default()),
            )),
            Token::new(TokenType::Star, "*", Object::Empty, Default::default()),
            Box::new(Expression::grouping(Box::new(Expression::literal(
                Object::Double(45.67),
            )))),
        );

        let printer = AstPrinter {};
        let value = printer.print(&expr).unwrap();
        assert_eq!("(* (- 123) (group 45.67))", value);
    }
}
