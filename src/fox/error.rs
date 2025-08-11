use std::fmt::Display;

use crate::fox::{Object, Source, Token};

use super::CodeLocation;

pub type FoxResult<T> = Result<T, FoxError>;

#[derive(Debug)]
pub struct FoxError {
    kind: ErrorKind,
    info: ErrorInfo,
}

impl FoxError {
    pub fn code_location(kind: ErrorKind, location: CodeLocation) -> Self {
        Self {
            kind,
            info: ErrorInfo::Code(location),
        }
    }

    pub fn token(kind: ErrorKind, token: Option<Token>) -> Self {
        let Some(token) = token else {
            return FoxError::error(kind);
        };
        Self {
            kind,
            info: ErrorInfo::Token(Box::new(token)),
        }
    }

    pub fn runtime(token: Option<Token>, message: &str) -> Self {
        let kind = ErrorKind::Runtime(message.to_string());
        Self::token(kind, token)
    }

    pub fn resolver(token: Option<Token>, message: &str) -> Self {
        let kind = ErrorKind::Resolver(message.to_string());
        Self::token(kind, token)
    }

    pub fn bug(message: &str) -> Self {
        let kind = ErrorKind::Bug(message.to_string());
        Self::token(kind, None)
    }

    pub fn error(kind: ErrorKind) -> Self {
        Self {
            kind,
            info: ErrorInfo::Empty,
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn info(&self) -> &ErrorInfo {
        &self.info
    }
}

#[derive(Clone, Debug)]
pub enum ErrorInfo {
    Empty,
    Code(CodeLocation),
    Token(Box<Token>),
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnexpectedCharacter,
    UnterminatedString,
    ExpressionExpected,
    ExpectedOperator,
    TooManyFunctionArguments,
    UndefinedVariable(String),
    InvalidAssignmentTarget,
    OperandMustBeNumber,
    Runtime(String),
    Parse(String),
    Resolver(String),
    Bug(String),
    Return(Object),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorKind::*;
        let text = match self {
            UnexpectedCharacter => "Unexpected character",
            UnterminatedString => "Unterminated string",
            ExpressionExpected => "Expect expression",
            ExpectedOperator => "Expect operator",
            TooManyFunctionArguments => "Can't have more than 255 arguments",
            UndefinedVariable(name) => &format!("Undefined variable {name}"),
            InvalidAssignmentTarget => "Invalid assignment target",
            OperandMustBeNumber => "Operand must be a number",
            Runtime(message) | Parse(message) | Resolver(message) => message,
            Bug(message) => &format!("[BUG] {message}"),
            Return(_) => unreachable!("Return shouldn't be an error"),
        };
        write!(f, "{text}")
    }
}

pub struct ErrorLine {
    line_number: usize,
    text: String,
    position: usize,
}

impl ErrorLine {
    pub fn with(code: &Source, location: &CodeLocation) -> Self {
        let (position, text) = Self::line_with_error(code, location);

        Self {
            line_number: location.line_number(),
            text,
            position,
        }
    }

    pub fn formatted(&self, message: &str) -> String {
        let mut lines: Vec<String> = Vec::new();
        let prefix = format!("{} |", self.line_number);
        lines.push(format!("{}{}", prefix, self.text));

        let arrow_idx = prefix.len() + self.position;
        let fill = " ".repeat(arrow_idx);
        lines.push(format!("{fill}▲"));

        if !message.is_empty() {
            let line = format!("{fill}└─ {message}");
            lines.push(line)
        }

        lines.join("\n")
    }

    fn line_with_error(code: &Source, location: &CodeLocation) -> (usize, String) {
        let mut left = location.absolute_position();
        let mut right = left;

        let is_terminator = |ch: char| -> bool { ch == '\n' || ch == '\r' };

        let len = code.len();
        let mut is_moving = true;
        while is_moving {
            is_moving = false;
            if left > 0 && !is_terminator(code[left - 1]) {
                is_moving = true;
                left -= 1;
            }

            if right < len - 1 && !is_terminator(code[right + 1]) {
                is_moving = true;
                right += 1;
            }
        }

        (
            location.absolute_position() - left,
            code[left..=right].iter().collect::<String>(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fetch_line() {
        let source = make_source();
        let marker = 'X';
        let position = source.iter().position(|x| *x == marker).unwrap();
        let location = CodeLocation::new(3, position);

        let el = ErrorLine::with(&source, &location);
        assert_eq!("consume(X_RIGHT_PAREN);", el.text.trim());

        let chars = el.text.chars().collect::<Vec<_>>();
        assert_eq!(chars[el.position], marker);
    }

    fn make_source() -> Vec<char> {
        r"
            if (match(LEFT_PAREN)) {
                  Expr expr = expression();
                  consume(X_RIGHT_PAREN);
                  return new Expr.Grouping(expr);
            }
        "
        .chars()
        .collect::<Vec<_>>()
    }
}
