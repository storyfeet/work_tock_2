use crate::reader::Clock;
use crate::tokenize::TokenType;
use std::fmt;

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
    JobNotSet,
    YearNotSet,
    DateNotSet,
    DateNotValid,
    ClockinNotSet,
    MissingItem,
    MinutesOver60,
    Expected(TokenType),
}
impl std::error::Error for ErrType {}

impl fmt::Display for ErrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {:?}", self)
    }
}

#[derive(Debug)]
pub struct ClockErr {
    pub clock: Clock,
    pub etype: ClockErrType,
}

#[derive(Debug)]
pub enum ClockErrType {
    OutBeforeIn,
}

impl fmt::Display for ClockErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error with Clock: {:?}  -- {:?}", self.clock, self.etype,)
    }
}
impl std::error::Error for ClockErr {}
