pub type TokenRes<'a> = Result<Token<'a>, ParseErr>;

pub struct Token<'a> {
    s: &'a str,
    line: usize,
    col: usize,
    tt: TokenType,
}

pub enum TokenType {
    Time,
    Ident,
    Dollar,
    Number,
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
            //Some(c) if c >= '0' && c <= '9' => self.number(),
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
        self.t_start += 1;
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
        }
        self.make_token(self.s.len() - self.t_start, TokenType::Number)
    }
}

pub struct ParseErr {
    line: usize,
    col: usize,
    etype: ErrType,
}

pub enum ErrType {
    NoToken,
}
