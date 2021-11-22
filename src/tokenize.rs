use crate::err::*;
pub type TokenRes<'a> = Result<Token<'a>, ParseErr>;

use std::str::FromStr;
#[derive(Debug)]
pub struct Token<'a> {
    pub s: &'a str,
    pub line: usize,
    pub col: usize,
    pub tt: TokenType,
}

impl<'a> Token<'a> {
    pub fn as_err(self, etype: ErrType) -> ParseErr {
        ParseErr {
            line: self.line,
            col: self.col,
            etype,
        }
    }

    pub fn num_val(self) -> Result<usize, ParseErr> {
        usize::from_str(self.s).map_err(|_| self.as_err(ErrType::NotANumber))
    }
}

#[derive(PartialEq, Debug)]
pub enum TokenType {
    Time,
    Ident,
    Dollar,
    Number,
    Colon,
    Slash,
    Minus,
    Comma,
    SquareOpen,
    SquareClose,
    EOF,
}

pub struct Tokenizer<'a> {
    s: &'a str,
    t_start: usize,
    line: usize,
    col: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Tokenizer {
            s: s,
            t_start: 0,
            line: 1,
            col: 0,
        }
    }

    pub fn chars(&mut self) -> std::str::CharIndices<'a> {
        self.s[self.t_start..].char_indices()
    }

    fn whitespace(&mut self) {
        let mut tmp = self.s[self.t_start..].char_indices();
        while let Some((i, c)) = tmp.next() {
            self.col += 1;
            match c {
                '\n' => {
                    self.line += 1;
                    self.col = 0
                }
                ' ' | '\t' | '\r' => {}
                '#' => {
                    while let Some((_, c)) = tmp.next() {
                        if c == '\n' {
                            self.col = 0;
                            self.line += 1;
                            break;
                        }
                    }
                }
                _ => {
                    self.t_start += i;
                    return;
                }
            }
        }
        self.t_start = self.s.len();
    }

    pub fn next_token(&mut self) -> TokenRes<'a> {
        self.whitespace();
        match self.s[self.t_start..].chars().next() {
            Some('$') => self.make_token(1, TokenType::Dollar),
            Some(':') => self.make_token(1, TokenType::Colon),
            Some('/') => self.make_token(1, TokenType::Slash),
            Some('[') => self.make_token(1, TokenType::SquareOpen),
            Some(']') => self.make_token(1, TokenType::SquareClose),
            Some('-') => self.make_token(1, TokenType::Minus),
            Some(',') => self.make_token(1, TokenType::Comma),
            Some(c) if c >= '0' && c <= '9' => self.number(),
            Some(c) if c.is_alphabetic() => self.ident(),
            Some(_) => self.make_err(ErrType::NoToken),
            None => self.make_token(0, TokenType::EOF),
        }
    }

    pub fn make_token(&mut self, len: usize, tt: TokenType) -> TokenRes<'a> {
        let res = Token {
            s: &self.s[self.t_start..self.t_start + len],
            line: self.line,
            col: self.col,
            tt,
        };
        self.t_start += len;
        Ok(res)
    }

    pub fn make_err(&mut self, etype: ErrType) -> TokenRes<'a> {
        Err(ParseErr {
            line: self.line,
            col: self.col,
            etype,
        })
    }

    pub fn number(&mut self) -> TokenRes<'a> {
        let mut tmp = self.chars();
        while let Some((i, c)) = tmp.next() {
            if c < '0' || c > '9' {
                return self.make_token(i, TokenType::Number);
            }
            self.col += 1;
        }
        self.make_token(self.s.len() - self.t_start, TokenType::Number)
    }

    pub fn ident(&mut self) -> TokenRes<'a> {
        let mut tmp = self.chars();
        while let Some((i, c)) = tmp.next() {
            if !c.is_alphabetic() {
                return self.make_token(i, TokenType::Ident);
            }
            self.col += 1;
        }
        self.make_token(self.s.len() - self.t_start, TokenType::Ident)
    }
}
