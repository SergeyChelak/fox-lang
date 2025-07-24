use crate::fox::{FoxResult, Object, Token};

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
                pub fn $fn_init($($p_name: $p_type,)*) -> Self {
                    Self::$option(
                        $option_data {
                            $($p_name,)*
                        }
                    )
                }
            )*

            pub fn accept<T>(&self, visitor: &dyn $visitor_type<T>) -> FoxResult<T> {
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
                fn accept<T>(&self, visitor: &dyn $visitor_type<T>) -> FoxResult<T> {
                    visitor.$fn_visit(self)
                }
            }
        )*

        pub trait $visitor_type<T> {
            $(
                fn $fn_visit(&self, data: &$option_data) -> FoxResult<T>;
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

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: &Expression) -> FoxResult<String> {
        expr.accept(self)
    }

    fn parenthesize(&self, name: &str, expressions: &[&Box<Expression>]) -> FoxResult<String> {
        // assert!(!name.is_empty());
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
    fn visit_binary(&self, data: &BinaryData) -> FoxResult<String> {
        let exprs = [&data.left, &data.right];
        self.parenthesize(data.operator.lexeme.as_str(), &exprs)
    }

    fn visit_grouping(&self, data: &GroupingData) -> FoxResult<String> {
        self.parenthesize("group", &[&data.expression])
    }

    fn visit_literal(&self, data: &LiteralData) -> FoxResult<String> {
        Ok(format!("{}", data.value))
    }

    fn visit_unary(&self, data: &UnaryData) -> FoxResult<String> {
        self.parenthesize(data.operator.lexeme.as_str(), &[&data.expression])
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
