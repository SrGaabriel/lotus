use std::str::Chars;
use strum::FromRepr;

pub const EOF_CHAR: char = '\0';

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum TokenKind {
    Whitespace,
    LineComment,
    BlockComment { terminated: bool },

    Identifier,
    DefKw,
    OpIdentifier,

    Number,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Semicolon,
    Comma,
    Dot,
    DefEq,

    Unknown,
    Eof,
}

impl TokenKind {
    pub const fn as_index(self) -> u8 {
        unsafe { *(&raw const self).cast::<u8>() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'input> {
    pub kind: TokenKind,
    pub text: &'input str,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy)]
struct RawToken {
    kind: TokenKind,
    len: u32,
}

impl RawToken {
    const fn new(kind: TokenKind, len: u32) -> Self {
        Self { kind, len }
    }
}

pub struct Lexer<'input> {
    input: &'input str,
    cursor: Cursor<'input>,
    offset: u32,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            cursor: Cursor::new(input),
            offset: 0,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.cursor.advance_token();
        if raw.kind == TokenKind::Eof {
            return None;
        }
        let start = self.offset as usize;
        let end = start + raw.len as usize;
        let text = &self.input[start..end];
        let offset = self.offset;
        self.offset += raw.len;
        Some(Token {
            kind: raw.kind,
            text,
            offset,
        })
    }
}

pub struct Cursor<'a> {
    len_remaining: u32,
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            len_remaining: input.len() as u32,
            chars: input.chars(),
        }
    }

    fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn pos_in_token(&self) -> u32 {
        self.len_remaining - self.chars.as_str().len() as u32
    }

    fn reset_pos(&mut self) {
        self.len_remaining = self.chars.as_str().len() as u32;
    }

    fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn eat_while(&mut self, mut pred: impl FnMut(char) -> bool) {
        while pred(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    fn read_while(&mut self, mut pred: impl FnMut(char) -> bool) -> String {
        let mut result = String::new();
        while pred(self.first()) && !self.is_eof() {
            result.push(self.bump().unwrap());
        }
        result
    }

    fn advance_token(&mut self) -> RawToken {
        let Some(c) = self.bump() else {
            return RawToken::new(TokenKind::Eof, 0);
        };

        let kind = match c {
            c if is_whitespace(c) => self.whitespace(),

            '/' if matches!(self.first(), '/' | '*') => match self.first() {
                '/' => self.line_comment(),
                '*' => self.block_comment(),
                _ => unreachable!(),
            },
            ':' if self.first() == '=' => {
                self.bump();
                TokenKind::DefEq
            }

            c if is_id_start(c) => self.ident(),
            c if is_op_char(c) => self.op_ident(),
            c if c.is_ascii_digit() => {
                self.eat_while(|c| c.is_ascii_digit());
                TokenKind::Number
            }

            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,

            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,

            _ => TokenKind::Unknown,
        };

        let len = self.pos_in_token();
        self.reset_pos();
        RawToken::new(kind, len)
    }

    fn whitespace(&mut self) -> TokenKind {
        self.eat_while(is_whitespace);
        TokenKind::Whitespace
    }

    fn line_comment(&mut self) -> TokenKind {
        self.bump();
        self.eat_while(|c| c != '\n');
        TokenKind::LineComment
    }

    fn block_comment(&mut self) -> TokenKind {
        self.bump();
        let mut depth: usize = 1;
        while let Some(c) = self.bump() {
            match c {
                '/' if self.first() == '*' => {
                    self.bump();
                    depth += 1;
                }
                '*' if self.first() == '/' => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        TokenKind::BlockComment {
            terminated: depth == 0,
        }
    }

    fn ident(&mut self) -> TokenKind {
        let ident = self.read_while(is_id_continue);
        match &*ident {
            "def" => TokenKind::DefKw,
            _ => TokenKind::Identifier,
        }
    }

    fn op_ident(&mut self) -> TokenKind {
        self.eat_while(is_op_char);
        TokenKind::OpIdentifier
    }
}

fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'
            | '\u{000A}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{000D}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

fn is_id_start(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

fn is_id_continue(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}

fn is_op_char(c: char) -> bool {
    matches!(
        c,
        '!' | '#'
            | '$'
            | '%'
            | '&'
            | '*'
            | '+'
            | '/'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '\\'
            | '^'
            | '|'
            | '-'
            | '~'
            | ':'
    )
}
