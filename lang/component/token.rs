use crate::syntax::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Float(pub f32);
impl Eq for Float {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Proc,
    Return,
    If,
    Else,
    Let,
    Global,
    Player,
    SelfKw,
}

impl From<Keyword> for String {
    fn from(kw: Keyword) -> String {
        match kw {
            Keyword::Proc => "proc".to_string(),
            Keyword::Return => "return".to_string(),
            Keyword::If => "if".to_string(),
            Keyword::Else => "else".to_string(),
            Keyword::Let => "let".to_string(),
            Keyword::Global => "global".to_string(),
            Keyword::Player => "player".to_string(),
            Keyword::SelfKw => "self".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delimiter {
    OpenParen,  // '('
    CloseParen, // ')'
    OpenBrace,  // '{'
    CloseBrace, // '}'
    Colon,      // ':'
    Camma,      // ','
    Arrow,      // '->'
    Dot,        // '.'
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Asterisk, // '*'
    Slash,    // '/'
    Percent,  // '%'
    Plus,     // '+'
    Minus,    // '-'
    Gt,       // '>'
    Lt,       // '<'
    Gte,      // '>='
    Lte,      // '<='
    Eq,       // '=='
    LogOr,    // '||'
    LogAnd,   // '&&'
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Float(Float),
    String(String),
    Keyword(Box<Keyword>),
    Delim(Box<Delimiter>),
    Op(Box<BinOp>),
    Assign,
    Newline,
    Ident(String),
    Eof,
    True,
    False,
    Type(Box<Type>),
}

pub fn token_type_eq(t1: &Token, t2: &Token) -> bool {
    match t1 {
        Token::Float(_) => matches!(t2, Token::Float(_)),
        Token::String(_) => matches!(t2, Token::String(_)),
        Token::Ident(_) => matches!(t2, Token::Ident(_)),
        Token::Assign => matches!(t2, Token::Assign),
        Token::Newline => matches!(t2, Token::Newline),
        Token::Keyword(kw1) => matches!(t2, Token::Keyword(kw2) if kw1 == kw2),
        Token::Delim(delim1) => matches!(t2, Token::Delim(delim2) if  delim1 == delim2),
        Token::Op(op1) => matches!(t2, Token::Op(op2) if  op1 == op2),
        Token::Eof => matches!(t2, Token::Eof),
        Token::True => matches!(t2, Token::True),
        Token::False => matches!(t2, Token::False),
        Token::Type(_) => matches!(t2, Token::Type(_)),
    }
}
