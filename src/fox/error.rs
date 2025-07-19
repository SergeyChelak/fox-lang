use super::CodeLocation;

pub struct Error {
    kind: ErrorKind,
    code_location: Option<CodeLocation>,
}

impl Error {
    pub fn new(kind: ErrorKind, code_location: Option<CodeLocation>) -> Self {
        Self {
            kind,
            code_location,
        }
    }

    pub fn error_kind(kind: ErrorKind) -> Self {
        Self::new(kind, None)
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind()
    }
}

#[derive(Clone, Copy)]
pub enum ErrorKind {
    UnexpectedCharacter,
}
