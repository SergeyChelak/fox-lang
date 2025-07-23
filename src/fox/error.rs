use super::CodeLocation;

#[derive(Debug)]
pub struct FoxError {
    kind: ErrorKind,
    info: ErrorInfo,
}

impl FoxError {
    pub fn code(kind: ErrorKind, location: CodeLocation) -> Self {
        Self {
            kind,
            info: ErrorInfo::Code(location),
        }
    }

    pub fn error(kind: ErrorKind) -> Self {
        Self {
            kind,
            info: ErrorInfo::Empty,
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorInfo {
    Empty,
    Code(CodeLocation),
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    UnexpectedCharacter,
    UnterminatedString,
    InputOutput,
}
