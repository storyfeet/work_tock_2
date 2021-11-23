use crate::err::*;
use crate::s_time::STime;
use crate::tokenize::{self, Token, TokenType};

pub type ActionRes<'a> = Result<Action<'a>, ParseErr>;

#[derive(PartialEq, Debug)]
pub struct Action<'a> {
    pub line: usize,
    pub col: usize,
    pub ad: ActionData<'a>,
}

impl<'a> Action<'a> {
    pub fn as_err(&self, etype: ErrType) -> ParseErr {
        ParseErr {
            line: self.line,
            col: self.col,
            etype,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ActionData<'a> {
    Group(String, Vec<String>),
    ShortDate(u32, u32),
    LongDate(u32, u32, i32),
    SetJob(&'a str),
    SetYear(i32),
    ClearTags,
    ClearTag(&'a str),
    Tag(&'a str),
    Clockin(STime),
    Clockout(STime),
    End,
}

impl<'a> ActionData<'a> {
    fn as_action(self, tk: &Token<'a>) -> Action<'a> {
        Action {
            line: tk.line,
            col: tk.col,
            ad: self,
        }
    }
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

    fn next_token_as(&mut self, tt: tokenize::TokenType) -> tokenize::TokenRes<'a> {
        let tk = self.next_token()?;
        if tk.tt != tt {
            return Err(tk.as_err(ErrType::Expected(tt)));
        }
        Ok(tk)
    }

    fn try_next_token<F: Fn(&Token) -> bool>(&mut self, f: F) -> Option<Token<'a>> {
        let tk = self.next_token().ok()?;
        match f(&tk) {
            true => Some(tk),
            false => {
                self.next = Some(tk);
                None
            }
        }
    }

    pub fn next_action(&mut self) -> ActionRes<'a> {
        let mut t = self.next_token()?;
        while t.tt == TokenType::Sep {
            t = self.tk.next_token()?;
        }

        match t.tt {
            TokenType::Number => self.from_number(t),
            TokenType::EOF => Ok(ActionData::End.as_action(&t)),
            TokenType::Minus => self.clock_out(),
            TokenType::Ident => self.from_ident(t),
            TokenType::Tag => {
                Ok(ActionData::Tag(self.next_token_as(TokenType::Ident)?.s).as_action(&t))
            }
            TokenType::ClearTag => match self.try_next_token(|t| t.tt == TokenType::Ident) {
                Some(nt) => Ok(ActionData::ClearTag(nt.s).as_action(&t)),
                None => Ok(ActionData::ClearTags.as_action(&t)),
            },
            TokenType::Dollar => self.group(),
            _ => Err(t.as_err(ErrType::NotAnItem)),
        }
    }
    /// Will process Dates and Clockins
    pub fn from_number(&mut self, num1: Token<'a>) -> ActionRes<'a> {
        let delim1 = self.next_token()?;
        if delim1.tt != TokenType::Colon && delim1.tt != TokenType::Slash {
            return Err(delim1.as_err(ErrType::NotSlashOrColon));
        }
        let num2 = self.next_token()?;
        if delim1.tt == TokenType::Colon {
            return Ok(
                ActionData::Clockin(STime::new(num1.num_val()?, num2.num_val()?)).as_action(&num1),
            );
        }
        let _delim2 = match self.try_next_token(|t| t.tt == TokenType::Slash) {
            Some(s) => s,
            None => {
                return Ok(ActionData::ShortDate(num1.num_val()?, num2.num_val()?).as_action(&num1))
            }
        };
        let num3 = self.tk.next_token()?;
        Ok(
            ActionData::LongDate(num1.num_val()?, num2.num_val()?, num3.num_val()? as i32)
                .as_action(&num1),
        )
    }

    /// Follows a '-'
    pub fn clock_out(&mut self) -> ActionRes<'a> {
        let num1 = self.next_token_as(TokenType::Number)?;
        let _ = self.next_token_as(TokenType::Colon)?;
        let num2 = self.next_token_as(TokenType::Number)?;
        Ok(ActionData::Clockout(STime::new(num1.num_val()?, num2.num_val()?)).as_action(&num1))
    }

    pub fn group(&mut self) -> ActionRes<'a> {
        let name = self.next_token_as(TokenType::Ident)?;
        self.next_token_as(TokenType::SquareOpen)?;
        let mut members = Vec::new();
        loop {
            let next_member = self.next_token()?;
            match next_member.tt {
                TokenType::Ident => members.push(next_member.s.to_string()),
                TokenType::Sep => {}
                TokenType::SquareClose => {
                    return Ok(ActionData::Group(name.s.to_string(), members).as_action(&name))
                }
                TokenType::EOF => return Err(next_member.as_err(ErrType::UnexpectedEOF)),
                _ => return Err(next_member.as_err(ErrType::Expected(TokenType::Ident))),
            }
        }
    }

    pub fn from_ident(&mut self, t: Token<'a>) -> ActionRes<'a> {
        if let Some(eq) = self.try_next_token(|t| t.tt == TokenType::Equals) {
            match t.s {
                "year" => {
                    let yr = self.next_token_as(TokenType::Number)?;
                    return Ok(ActionData::SetYear(yr.num_val()? as i32).as_action(&t));
                }
                _ => return Err(eq.as_err(ErrType::NotYear)),
            }
        }
        Ok(ActionData::SetJob(t.s).as_action(&t))
    }
}
