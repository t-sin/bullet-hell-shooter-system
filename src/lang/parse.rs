use nom::{
    branch::alt,
    combinator::{all_consuming, map, rest_len},
    multi::many0,
    sequence::tuple,
    Err, IResult,
};

use super::syntax_tree::*;
use super::tokenize::*;

// ErrorKinds for this bullet-hell lang.
#[derive(Debug)]
pub enum ErrorKind {
    Nom(nom::error::ErrorKind),
    UnexpectedToken(Token),
    UnexpectedEOF,
    CannotParseExpression,
    InvalidGlobalAssign,
    InvalidExpr,
    InvalidBinaryOperation,
    EmptyName,
    NotAnExprTerm,
    NeverReachedHere,
}

// Represents parser errors.
#[derive(Debug)]
pub struct ParseError<I> {
    input: I,
    kind: ErrorKind,
}

impl<I> ParseError<I> {
    fn new(input: I, kind: ErrorKind) -> Self {
        ParseError {
            input: input,
            kind: kind,
        }
    }
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
    /// converts nom::error::ErrorKind to mmmm's ParseError
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        ParseError {
            input: input,
            kind: ErrorKind::Nom(kind),
        }
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

type Input<'a> = &'a [Token];

type CombinatorResult<'a> = IResult<Input<'a>, &'a Token, ParseError<Input<'a>>>;

fn token<'a>(token: Token) -> impl Fn(Input<'a>) -> CombinatorResult<'a> {
    move |t: Input<'a>| {
        let (t, len) = rest_len(t)?;

        if len > 0 {
            if &t[0] == &token {
                Ok((&t[1..], &t[0]))
            } else {
                Err(Err::Error(ParseError::new(
                    &t[..],
                    ErrorKind::UnexpectedToken(t[0].clone()),
                )))
            }
        } else {
            Err(Err::Error(ParseError::new(
                &t[..],
                ErrorKind::UnexpectedEOF,
            )))
        }
    }
}

fn token_type<'a>(token: Token) -> impl Fn(&'a [Token]) -> CombinatorResult<'a> {
    move |t| {
        let (t, len) = rest_len(t)?;
        if len > 0 {
            if token_type_eq(&t[0], &token) {
                Ok((&t[1..], &t[0]))
            } else {
                Err(Err::Error(ParseError::new(
                    &t[..],
                    ErrorKind::UnexpectedToken(t[0].clone()),
                )))
            }
        } else {
            Err(Err::Error(ParseError::new(
                &t[..],
                ErrorKind::UnexpectedEOF,
            )))
        }
    }
}

fn parse_expr_term<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match alt((
        token_type(Token::Float(Float(0.0))),
        token_type(Token::String("".to_string())),
        token_type(Token::Ident("".to_string())),
    ))(t)
    {
        Ok((t, Token::Float(Float(f)))) => Ok((t, Expr::Float(*f))),
        Ok((t, Token::String(s))) => Ok((t, Expr::String(s.to_string()))),
        Ok((t, Token::Ident(name))) => {
            let name = name.to_string();
            if name.len() == 0 {
                Err(Err::Error(ParseError::new(t, ErrorKind::EmptyName)))
            } else {
                let expr = if let Some('$') = name.chars().nth(0) {
                    Expr::Symbol(Symbol::State(Name(name)))
                } else {
                    Expr::Symbol(Symbol::Var(Name(name)))
                };
                Ok((t, expr))
            }
        }
        _ => Err(Err::Error(ParseError::new(t, ErrorKind::NotAnExprTerm))),
    }
}

// '*', '/', '%'
fn parse_expr_op_level1<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match tuple((
        parse_expr,
        alt((
            token(Token::Op(Box::new(BinOp::Asterisk))),
            token(Token::Op(Box::new(BinOp::Slash))),
            token(Token::Op(Box::new(BinOp::Percent))),
        )),
        parse_expr,
    ))(t)
    {
        Ok((t, (expr1, Token::Op(op), expr2))) => match **op {
            BinOp::Asterisk => Ok((t, Expr::Op2(Op2::Mul, Box::new(expr1), Box::new(expr2)))),
            BinOp::Slash => Ok((t, Expr::Op2(Op2::Div, Box::new(expr1), Box::new(expr2)))),
            BinOp::Percent => Ok((t, Expr::Op2(Op2::Mod, Box::new(expr1), Box::new(expr2)))),
            _ => Err(Err::Error(ParseError::new(
                t,
                ErrorKind::InvalidBinaryOperation,
            ))),
        },
        _ => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidBinaryOperation,
        ))),
    }
}

// '+', '-'
fn parse_expr_op_level2<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match tuple((
        parse_expr,
        alt((
            token(Token::Op(Box::new(BinOp::Plus))),
            token(Token::Op(Box::new(BinOp::Minus))),
        )),
        parse_expr_op_level1,
    ))(t)
    {
        Ok((t, (expr1, Token::Op(op), expr2))) => match **op {
            BinOp::Plus => Ok((t, Expr::Op2(Op2::Add, Box::new(expr1), Box::new(expr2)))),
            BinOp::Minus => Ok((t, Expr::Op2(Op2::Sub, Box::new(expr1), Box::new(expr2)))),
            _ => Err(Err::Error(ParseError::new(t, ErrorKind::NeverReachedHere))),
        },
        _ => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidBinaryOperation,
        ))),
    }
}

// '>', '<', '>=', '<=', '=='
fn parse_expr_op_level3<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match tuple((
        parse_expr,
        alt((
            token(Token::Op(Box::new(BinOp::Gt))),
            token(Token::Op(Box::new(BinOp::Lt))),
            token(Token::Op(Box::new(BinOp::Gte))),
            token(Token::Op(Box::new(BinOp::Lte))),
            token(Token::Op(Box::new(BinOp::Eq))),
        )),
        parse_expr_op_level2,
    ))(t)
    {
        Ok((t, (expr1, Token::Op(op), expr2))) => match **op {
            BinOp::Gt => Ok((t, Expr::Op2(Op2::Gt, Box::new(expr1), Box::new(expr2)))),
            BinOp::Lt => Ok((t, Expr::Op2(Op2::Lt, Box::new(expr1), Box::new(expr2)))),
            BinOp::Gte => Ok((t, Expr::Op2(Op2::Gte, Box::new(expr1), Box::new(expr2)))),
            BinOp::Lte => Ok((t, Expr::Op2(Op2::Lte, Box::new(expr1), Box::new(expr2)))),
            BinOp::Eq => Ok((t, Expr::Op2(Op2::Eq, Box::new(expr1), Box::new(expr2)))),
            _ => Err(Err::Error(ParseError::new(t, ErrorKind::NeverReachedHere))),
        },
        _ => Err(Err::Error(ParseError::new(t, ErrorKind::NeverReachedHere))),
    }
}

fn parse_expr<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match alt((parse_expr_op_level3, parse_expr_term))(t) {
        Ok((t, expr)) => Ok((t, expr)),
        _ => Err(Err::Error(ParseError::new(t, ErrorKind::InvalidExpr))),
    }
}

fn parse_global_assign<'a>(t: Input<'a>) -> IResult<Input<'a>, SyntaxTree, ParseError<Input<'a>>> {
    match tuple((
        token(Token::Keyword(Box::new(Keyword::Let))),
        token_type(Token::Ident("".to_string())),
        token(Token::Assign),
        parse_expr,
    ))(t)
    {
        Ok((t, (_, Token::Ident(target_name), _, expr))) => Ok((
            t,
            SyntaxTree::GlobalAssign(Symbol::Var(Name(target_name.to_string())), expr),
        )),
        _ => Err(Err::Error(ParseError {
            input: t,
            kind: ErrorKind::InvalidGlobalAssign,
        })),
    }
}

type Parse1Result<'a> = IResult<Input<'a>, Option<SyntaxTree>, ParseError<Input<'a>>>;

fn parse_1<'a>(t: Input<'a>) -> Parse1Result<'a> {
    let p = alt((
        // value(None, token_type_of(Token::LineComment("".to_string()))),
        map(token(Token::Newline), |_| None),
        map(parse_global_assign, |ga| Some(ga)),
    ))(t);
    println!("parse_1() = {:?}", p);
    p
}

type ParseResult<'a> = IResult<Input<'a>, Vec<SyntaxTree>, ParseError<Input<'a>>>;

pub fn parse<'a>(t: Input<'a>) -> ParseResult<'a> {
    match all_consuming(many0(parse_1))(t) {
        Ok((rest, stvec)) => Ok((
            rest,
            stvec
                .into_iter()
                .filter(|o| if let None = o { false } else { true })
                .map(|o| o.unwrap())
                .collect(),
        )),
        Err(err) => {
            println!("parse() = {:?}", err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod parser_test {
    use super::*;

    fn test_parse_1(expected: SyntaxTree, string: &str) {
        println!("text: {:?}", string);
        if let Ok(("", tokens)) = tokenize(string) {
            println!("tokens: {:?}", tokens);

            match parse(&tokens) {
                Ok((&[], vec)) => {
                    assert_eq!(1, vec.len());

                    if let Some(st) = vec.iter().nth(0) {
                        assert_eq!(*st, expected);
                    } else {
                        println!("This test case itself is wrong....");
                        assert!(false);
                    }
                }
                err => {
                    println!("{:?}", err);
                    assert!(false);
                }
            }
        } else {
            println!("This test case itself is wrong....");
            assert!(false);
        }
    }

    #[test]
    fn test_parse_global_assign() {
        test_parse_1(
            SyntaxTree::GlobalAssign(Symbol::Var(Name("a".to_string())), Expr::Float(42.0)),
            "let a = 42.0",
        );
    }

    #[test]
    fn test_parse_expr_simple_op() {
        test_parse_1(
            SyntaxTree::GlobalAssign(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Eq,
                    Box::new(Expr::Float(1.0)),
                    Box::new(Expr::Float(2.0)),
                ),
            ),
            "let a = 1.0 == 2.0",
        );
        test_parse_1(
            SyntaxTree::GlobalAssign(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Add,
                    Box::new(Expr::Float(1.0)),
                    Box::new(Expr::Float(2.0)),
                ),
            ),
            "let a = 2.0 + 1.0",
        );
        test_parse_1(
            SyntaxTree::GlobalAssign(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Mul,
                    Box::new(Expr::Float(1.0)),
                    Box::new(Expr::Float(2.0)),
                ),
            ),
            "let a = 2.0 * 1.0",
        );
    }

    #[test]
    fn test_parse_expr_op_precedence() {
        test_parse_1(
            SyntaxTree::GlobalAssign(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Eq,
                    Box::new(Expr::Op2(
                        Op2::Add,
                        Box::new(Expr::Float(1.0)),
                        Box::new(Expr::Float(2.0)),
                    )),
                    Box::new(Expr::Float(3.0)),
                ),
            ),
            "let a = 1.0 == 2.0 + 3.0",
        );
        test_parse_1(
            SyntaxTree::GlobalAssign(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Eq,
                    Box::new(Expr::Op2(
                        Op2::Mul,
                        Box::new(Expr::Float(-1.0)),
                        Box::new(Expr::Float(1.0)),
                    )),
                    Box::new(Expr::Op2(
                        Op2::Add,
                        Box::new(Expr::Float(2.0)),
                        Box::new(Expr::Float(3.0)),
                    )),
                ),
            ),
            "let a = -1.0 * 1.0 == 2.0 + 3.0",
        );
        test_parse_1(
            SyntaxTree::GlobalAssign(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Eq,
                    Box::new(Expr::Op2(
                        Op2::Add,
                        Box::new(Expr::Op2(
                            Op2::Mul,
                            Box::new(Expr::Float(1.0)),
                            Box::new(Expr::Float(2.0)),
                        )),
                        Box::new(Expr::Float(3.0)),
                    )),
                    Box::new(Expr::Float(4.0)),
                ),
            ),
            "let a = 1.0 * 2.0 + 3.0 == 4.0",
        );
        test_parse_1(
            SyntaxTree::GlobalAssign(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Eq,
                    Box::new(Expr::Float(1.0)),
                    Box::new(Expr::Op2(
                        Op2::Add,
                        Box::new(Expr::Float(2.0)),
                        Box::new(Expr::Op2(
                            Op2::Mul,
                            Box::new(Expr::Float(3.0)),
                            Box::new(Expr::Float(4.0)),
                        )),
                    )),
                ),
            ),
            "let a = 1.0 == 2.0 + 3.0 * 4.0",
        );
    }
}
