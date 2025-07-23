use crate::fox::{Error, Object, Token};

macro_rules! ast_expressions {
    (
        $holder_type:ident accepting $visitor_type:ident {
            $(
                $option:ident($option_data:ident {
                    $(
                        $p_name:ident: $p_type:ty
                    ),*$(,)?
                }) with $fn_visit:ident)
            ,*
            $(,)?
        }
    ) => {
        pub enum $holder_type {
            $($option($option_data),)*
        }

        impl $holder_type {
            $(
                fn $option($($p_name: $p_type,)*) -> Self {
                    Self::$option(
                        $option_data {
                            $($p_name,)*
                        }
                    )
                }
            )*

            fn accept<T>(&self, visitor: &dyn $visitor_type<T>) -> Result<T, Error> {
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
                fn accept<T>(&self, visitor: &dyn $visitor_type<T>) -> Result<T, Error> {
                    visitor.$fn_visit(self)
                }
            }
        )*

        pub trait $visitor_type<T> {
            $(
                fn $fn_visit(&self, data: &$option_data) -> Result<T, Error>;
            )*
        }
    };
}

ast_expressions!(
    Expression accepting ExpressionVisitor {
        Binary(
            BinaryData {
                left: Box<Expression>,
                right: Box<Expression>,
                operator: Token
            }
        ) with visit_binary,

        Grouping(
            GroupingData {
                expression: Box<Expression>
            }
        ) with visit_grouping,

        Literal(
            LiteralData {
                value: Object
            }
        ) with visit_literal,

        Unary(UnaryData {
                expression: Box<Expression>,
                operator: Token
            }
        ) with visit_unary
    }
);

#[cfg(test)]
mod test {
    use super::*;

    struct AstPrinter {
        //
    }

    impl AstPrinter {
        fn parenthesize(&self, name: &str, expressions: &[&Box<Expression>]) -> String {
            todo!()
        }
    }

    impl ExpressionVisitor<String> for AstPrinter {
        fn visit_binary(&self, data: &BinaryData) -> Result<String, Error> {
            let exprs = [&data.left, &data.right];
            Ok(self.parenthesize(data.operator.lexeme.as_str(), &exprs))
        }

        fn visit_grouping(&self, data: &GroupingData) -> Result<String, Error> {
            Ok(self.parenthesize("groupng", &[&data.expression]))
        }

        fn visit_literal(&self, data: &LiteralData) -> Result<String, Error> {
            todo!()
        }

        fn visit_unary(&self, data: &UnaryData) -> Result<String, Error> {
            Ok(self.parenthesize(data.operator.lexeme.as_str(), &[&data.expression]))
        }
    }

    #[test]
    fn test_ast_printer() {
        // let expr = Expression::Binary(
        //     Expression::Unary(Expression::Literal(Object::Double(123.0)), operator)
        // )
    }
}
