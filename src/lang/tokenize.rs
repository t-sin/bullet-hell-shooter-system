use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, none_of, one_of, space0, space1},
    combinator::{opt, peek},
    error::{Error, ErrorKind},
    multi::{many0, many1},
    sequence::tuple,
    Err, IResult,
};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delimiter {
    OpenParen,  // '('
    CloseParen, // ')'
    OpenBrace,  // '{'
    CloseBrace, // '}'
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
    }
}

fn tokenize_float(s: &str) -> IResult<&str, Token> {
    let (s, minus) = opt(char('-'))(s)?;

    match tuple((digit1, char('.'), digit1))(s) {
        Ok((s, (int, _, fract))) => {
            let fstr = format!("{}.{}", int, fract);
            let mut f = fstr.parse::<f32>().unwrap();
            if let Some(_) = minus {
                f = -1.0 * f;
            }

            Ok((s, Token::Float(Float(f))))
        }
        Err(err) => Err(err),
    }
}

fn tokenize_string(s: &str) -> IResult<&str, Token> {
    let (s, (_, string, _)) = tuple((char('"'), many0(none_of("\"")), char('"')))(s)?;
    let string: String = string.iter().collect();
    Ok((s, Token::String(string)))
}

fn tokenize_delimiter_str(s: &str) -> IResult<&str, &str> {
    alt((tag("("), tag(")"), tag("{"), tag("}"), tag("\n")))(s)
}

fn tokenize_delimiter(s: &str) -> IResult<&str, Token> {
    match tokenize_delimiter_str(s)? {
        (s, "(") => Ok((s, Token::Delim(Box::new(Delimiter::OpenParen)))),
        (s, ")") => Ok((s, Token::Delim(Box::new(Delimiter::CloseParen)))),
        (s, "{") => Ok((s, Token::Delim(Box::new(Delimiter::OpenBrace)))),
        (s, "}") => Ok((s, Token::Delim(Box::new(Delimiter::CloseBrace)))),
        (s, _) => Err(Err::Error(Error::new(s, ErrorKind::Char))),
    }
}

fn tokenize_keyword(s: &str) -> IResult<&str, Token> {
    match tuple((
        alt((
            tag("proc"),
            tag("return"),
            tag("if"),
            tag("else"),
            tag("let"),
        )),
        alt((space1, peek(tokenize_delimiter_str))),
    ))(s)?
    {
        (s, ("proc", _)) => Ok((s, Token::Keyword(Box::new(Keyword::Proc)))),
        (s, ("return", _)) => Ok((s, Token::Keyword(Box::new(Keyword::Return)))),
        (s, ("if", _)) => Ok((s, Token::Keyword(Box::new(Keyword::If)))),
        (s, ("else", _)) => Ok((s, Token::Keyword(Box::new(Keyword::Else)))),
        (s, ("let", _)) => Ok((s, Token::Keyword(Box::new(Keyword::Let)))),
        (s, _) => Err(Err::Error(Error::new(s, ErrorKind::Char))),
    }
}

fn tokenize_op(s: &str) -> IResult<&str, Token> {
    match alt((
        tag("+"),
        tag("-"),
        tag("*"),
        tag("/"),
        tag("%"),
        tag(">"),
        tag("<"),
        tag(">="),
        tag("<="),
        tag("=="),
    ))(s)?
    {
        (s, "+") => Ok((s, Token::Op(Box::new(BinOp::Plus)))),
        (s, "-") => Ok((s, Token::Op(Box::new(BinOp::Minus)))),
        (s, "*") => Ok((s, Token::Op(Box::new(BinOp::Asterisk)))),
        (s, "/") => Ok((s, Token::Op(Box::new(BinOp::Slash)))),
        (s, "%") => Ok((s, Token::Op(Box::new(BinOp::Percent)))),
        (s, ">") => Ok((s, Token::Op(Box::new(BinOp::Gt)))),
        (s, "<") => Ok((s, Token::Op(Box::new(BinOp::Lt)))),
        (s, ">=") => Ok((s, Token::Op(Box::new(BinOp::Gte)))),
        (s, "<=") => Ok((s, Token::Op(Box::new(BinOp::Lte)))),
        (s, "==") => Ok((s, Token::Op(Box::new(BinOp::Eq)))),
        (s, _) => Err(Err::Error(Error::new(s, ErrorKind::Char))),
    }
}

fn tokenize_misc(s: &str) -> IResult<&str, Token> {
    match alt((char('='), char('\n')))(s)? {
        (s, '=') => Ok((s, Token::Assign)),
        (s, '\n') => Ok((s, Token::Newline)),
        (s, _) => Err(Err::Error(Error::new(s, ErrorKind::Char))),
    }
}

fn tokenize_ident(s: &str) -> IResult<&str, Token> {
    let digits = "0123456789";
    let alpha = "abcdefghijklmnopqrstuvwxyz";
    let alpha_cap = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let underbar = "_";
    let first_chars = ["$", alpha, alpha_cap, underbar].concat();
    let rest_chars = [digits, alpha, alpha_cap, underbar].concat();

    let (s, (first, rest)) = tuple((
        many1(one_of(&first_chars[..])),
        many0(one_of(&rest_chars[..])),
    ))(s)?;
    let first: String = first.into_iter().collect();
    let rest: String = rest.into_iter().collect();
    let ident: String = [first, rest].concat();

    Ok((s, Token::Ident(ident)))
}

pub fn tokenize(s: &str) -> IResult<&str, Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut input = s;

    loop {
        let (s, _) = opt(space0)(input)?;
        input = s;
        if s == "" {
            break;
        }

        let (s, token) = alt((
            tokenize_float,
            tokenize_string,
            tokenize_delimiter,
            tokenize_keyword,
            tokenize_op,
            tokenize_misc,
            tokenize_ident,
        ))(input)?;
        input = s;
        tokens.push(token);
    }
    tokens.push(Token::Eof);

    Ok((input, tokens))
}

#[cfg(test)]
mod tokenizer_test {
    use super::*;

    type TokenizeFn = dyn Fn(&str) -> IResult<&str, Token>;

    fn test_tokenize_fn(tokenize_fn: &TokenizeFn, expected: Token, input: &str) {
        if let Ok(("", result)) = tokenize_fn(input) {
            assert_eq!(expected, result);
        } else {
            println!("result = {:?}", tokenize_fn(input));
            assert!(false);
        }
    }

    fn test_tokenize_fn_fails(tokenize_fn: &TokenizeFn, input: &str) {
        if let Ok(("", _)) = tokenize_fn(input) {
            println!("result = {:?}", tokenize_fn(input));
            assert!(false);
        } else {
            assert!(true);
        }
    }

    #[test]
    fn test_tokenize_float_empty() {
        test_tokenize_fn_fails(&tokenize_float, "");
    }

    #[test]
    fn test_tokenize_float() {
        test_tokenize_fn(&tokenize_float, Token::Float(Float(42.0)), "42.0");
        test_tokenize_fn(&tokenize_float, Token::Float(Float(-12.0)), "-12.0");
    }

    #[test]
    fn test_tokenize_string_empty() {
        test_tokenize_fn_fails(&tokenize_string, "");
    }

    #[test]
    fn test_tokenize_string() {
        test_tokenize_fn(&tokenize_string, Token::String("".to_string()), "\"\"");
        test_tokenize_fn(
            &tokenize_string,
            Token::String("abc".to_string()),
            "\"abc\"",
        );
    }

    fn test_tokenize_1(expected: Vec<Token>, input: &str) {
        if let Ok(("", result)) = tokenize(input) {
            assert_eq!(expected, result);
        } else {
            println!("result = {:?}", tokenize(input));
            assert!(false);
        }
    }

    #[test]
    fn test_tokenize_complex1() {
        test_tokenize_1(
            vec![
                Token::Keyword(Box::new(Keyword::Proc)),
                Token::Ident("main".to_string()),
                Token::Delim(Box::new(Delimiter::OpenParen)),
                Token::Delim(Box::new(Delimiter::CloseParen)),
                Token::Delim(Box::new(Delimiter::OpenBrace)),
                Token::Keyword(Box::new(Keyword::Return)),
                Token::Delim(Box::new(Delimiter::CloseBrace)),
                Token::Eof,
            ],
            r"proc main() { return }",
        )
    }

    #[test]
    fn test_tokenize_complex2() {
        test_tokenize_1(
            vec![
                Token::Keyword(Box::new(Keyword::Let)),
                Token::Ident("v".to_string()),
                Token::Assign,
                Token::Delim(Box::new(Delimiter::OpenParen)),
                Token::Float(Float(1.0)),
                Token::Op(Box::new(BinOp::Plus)),
                Token::Float(Float(2.0)),
                Token::Delim(Box::new(Delimiter::CloseParen)),
                Token::Op(Box::new(BinOp::Asterisk)),
                Token::Float(Float(3.0)),
                Token::Eof,
            ],
            r"let v = (1.0 + 2.0) * 3.0",
        )
    }

    #[test]
    fn test_tokenize_complex3() {
        test_tokenize_1(
            vec![
                Token::Newline,
                Token::Keyword(Box::new(Keyword::Proc)),
                Token::Ident("main".to_string()),
                Token::Delim(Box::new(Delimiter::OpenParen)),
                Token::Delim(Box::new(Delimiter::CloseParen)),
                Token::Delim(Box::new(Delimiter::OpenBrace)),
                Token::Newline,
                Token::Keyword(Box::new(Keyword::Let)),
                Token::Ident("new_x".to_string()),
                Token::Assign,
                Token::Ident("$px".to_string()),
                Token::Op(Box::new(BinOp::Plus)),
                Token::Float(Float(1.0)),
                Token::Newline,
                Token::Newline,
                Token::Keyword(Box::new(Keyword::If)),
                Token::Ident("new_x".to_string()),
                Token::Op(Box::new(BinOp::Gt)),
                Token::Float(Float(420.0)),
                Token::Delim(Box::new(Delimiter::OpenBrace)),
                Token::Ident("$px".to_string()),
                Token::Assign,
                Token::Float(Float(420.0)),
                Token::Delim(Box::new(Delimiter::CloseBrace)),
                Token::Keyword(Box::new(Keyword::Else)),
                Token::Delim(Box::new(Delimiter::OpenBrace)),
                Token::Ident("$px".to_string()),
                Token::Assign,
                Token::Ident("new_x".to_string()),
                Token::Delim(Box::new(Delimiter::CloseBrace)),
                Token::Newline,
                Token::Delim(Box::new(Delimiter::CloseBrace)),
                Token::Newline,
                Token::Eof,
            ],
            r###"
              proc main() {
                let new_x = $px + 1.0

                if new_x > 420.0 { $px = 420.0 } else { $px = new_x }
              }
            "###,
        )
    }

    #[test]
    fn test_tokenize_complex4() {
        test_tokenize_1(
            vec![
                Token::Newline,
                Token::Keyword(Box::new(Keyword::Proc)),
                Token::Ident("main".to_string()),
                Token::Delim(Box::new(Delimiter::OpenParen)),
                Token::Delim(Box::new(Delimiter::CloseParen)),
                Token::Delim(Box::new(Delimiter::OpenBrace)),
                Token::Newline,
                Token::Keyword(Box::new(Keyword::Let)),
                Token::Ident("new_x".to_string()),
                Token::Assign,
                Token::Ident("$px".to_string()),
                Token::Op(Box::new(BinOp::Plus)),
                Token::Float(Float(1.0)),
                Token::Newline,
                Token::Keyword(Box::new(Keyword::Return)),
                Token::Newline,
                Token::Delim(Box::new(Delimiter::CloseBrace)),
                Token::Newline,
                Token::Eof,
            ],
            r###"
              proc main() {
                let new_x = $px + 1.0
                return
              }
            "###,
        )
    }

    // TODO: taking and returing values reqiures type signatures.
    // #[test]
    // fn test_tokenize_with_type_signatures() {}
}
