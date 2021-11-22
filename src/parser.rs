use crate::err::*;
use crate::s_time::STime;
use crate::tokenize::{self, Token, TokenType};

pub type ActionRes<'a> = Result<Action<'a>, ParseErr>;

#[derive(PartialEq, Debug)]
pub enum Action<'a> {
    ShortDate(usize, usize),
    LongDate(usize, usize, usize),
    SetJob(&'a str),

    Clockin(STime),
    Clockout(STime),
    End,
}

pub struct Parser<'a> {
    tk: tokenize::Tokenizer<'a>,
    next: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Self {
        Parser {
            tk: tokenize::Tokenizer::new(s),
            next: None,
        }
    }

    fn next_token(&mut self) -> tokenize::TokenRes<'a> {
        match self.next.take() {
            Some(t) => Ok(t),
            None => self.tk.next_token(),
        }
    }
    pub fn next_action(&mut self) -> ActionRes<'a> {
        let t = self.tk.next_token()?;
        match t.tt {
            TokenType::Number => self.from_number(t),
            TokenType::EOF => Ok(Action::End),
            _ => Err(t.as_err(ErrType::NotANumber)),
        }
    }
    /// Will process Dates and Clockins
    pub fn from_number(&mut self, num1: Token) -> ActionRes<'a> {
        let delim1 = self.next_token()?;
        if delim1.tt != TokenType::Colon && delim1.tt != TokenType::Slash {
            return Err(delim1.as_err(ErrType::NotSlashOrColon));
        }
        let num2 = self.next_token()?;
        if delim1.tt == TokenType::Colon {
            return Ok(Action::Clockin(STime::new(
                num1.num_val()?,
                num2.num_val()?,
            )));
        }
        let delim2 = self.tk.next_token()?;
        if delim2.tt != TokenType::Slash {
            self.next = Some(delim2);
            return Ok(Action::ShortDate(num1.num_val()?, num2.num_val()?));
        }
        let num3 = self.tk.next_token()?;
        Ok(Action::LongDate(
            num1.num_val()?,
            num2.num_val()?,
            num3.num_val()?,
        ))
    }
}
