use crate::tokenize::TokenType;
use std::fmt;

pub type BoxErr = Box<dyn std::error::Error>;

pub trait CanErr<T> {
    fn as_err(self) -> Result<T, BoxErr>;
}

impl<T, E: std::error::Error + 'static> CanErr<T> for Result<T, E> {
    fn as_err(self) -> Result<T, BoxErr> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[derive(Debug)]
pub struct ParseErr {
    pub line: usize,
    pub col: usize,
    pub etype: ErrType,
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error {:?} at l:{},c:{}",
            self.etype, self.line, self.col
        )
    }
}

impl std::error::Error for ParseErr {}

#[derive(Debug)]
pub enum ErrType {
    NoToken,
    NotANumber,
    NotAnItem,
    NotSlashOrColon,
    NotATime,
    NotYear,
    UnexpectedEOF,
    YearNotSet,
    DateNotSet,
    Expected(TokenType),
}
