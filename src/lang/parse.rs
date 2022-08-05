use nom::{
    branch::alt,
    combinator::{all_consuming, map, opt, peek, rest_len},
    multi::many0,
    sequence::{delimited, tuple},
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
    InvalidGlobalDefine,
    InvalidExpr,
    InvalidDefProc,
    InvalidLexicalDefine,
    EmptyName,
    NotAnExprTerm,
}

// Represents parser errors.
#[derive(Debug)]
pub struct ParseError<I> {
    input: I,
    kind: ErrorKind,
    parent: Option<Box<ParseError<I>>>,
}

impl<I> ParseError<I> {
    fn new(input: I, kind: ErrorKind, parent: Option<Box<ParseError<I>>>) -> Self {
        ParseError {
            input,
            kind,
            parent,
        }
    }
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
    /// converts nom::error::ErrorKind to mmmm's ParseError
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        ParseError {
            input: input,
            kind: ErrorKind::Nom(kind),
            parent: None,
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
                    None,
                )))
            }
        } else {
            Err(Err::Error(ParseError::new(
                &t[..],
                ErrorKind::UnexpectedEOF,
                None,
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
                    None,
                )))
            }
        } else {
            Err(Err::Error(ParseError::new(
                &t[..],
                ErrorKind::UnexpectedEOF,
                None,
            )))
        }
    }
}

fn token_op<'a>(t: Input<'a>) -> CombinatorResult<'a> {
    let (t, len) = rest_len(t)?;
    if len > 0 {
        if let Token::Op(_) = &t[0] {
            Ok((&t[1..], &t[0]))
        } else {
            Err(Err::Error(ParseError::new(
                &t[..],
                ErrorKind::UnexpectedToken(t[0].clone()),
                None,
            )))
        }
    } else {
        Err(Err::Error(ParseError::new(
            &t[..],
            ErrorKind::UnexpectedEOF,
            None,
        )))
    }
}

fn make_symbol(name: &str) -> Option<Symbol> {
    let name = name.to_string();
    if name.len() == 0 {
        None
    } else {
        let s = if let Some('$') = name.chars().nth(0) {
            Symbol::State(Name(name[1..].to_string()))
        } else {
            Symbol::Var(Name(name))
        };

        Some(s)
    }
}

fn parse_body_block_return<'a>(t: Input<'a>) -> IResult<Input<'a>, Body, ParseError<Input<'a>>> {
    match tuple((
        token(Token::Keyword(Box::new(Keyword::Return))),
        opt(parse_expr),
        alt((
            token(Token::Newline),
            peek(token(Token::Delim(Box::new(Delimiter::CloseBrace)))),
        )),
    ))(t)
    {
        Ok((t, (_, retval, _))) => Ok((t, Body::Return(retval))),
        Err(err) => Err(err),
    }
}

fn parse_body_block_lexical_define<'a>(
    t: Input<'a>,
) -> IResult<Input<'a>, Body, ParseError<Input<'a>>> {
    match tuple((
        token(Token::Keyword(Box::new(Keyword::Let))),
        token_type(Token::Ident("".to_string())),
        token(Token::Assign),
        parse_expr,
        alt((
            token(Token::Newline),
            peek(token(Token::Delim(Box::new(Delimiter::CloseBrace)))),
        )),
    ))(t)
    {
        Ok((t, (_, Token::Ident(target_name), _, expr, _))) => {
            if let Some(name) = make_symbol(target_name) {
                Ok((t, Body::LexicalDefine(name, expr)))
            } else {
                Err(Err::Error(ParseError::new(t, ErrorKind::EmptyName, None)))
            }
        }
        Ok((t, (_, _, _, _, _))) => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidLexicalDefine,
            None,
        ))),
        Err(Err::Error(err)) => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidLexicalDefine,
            Some(Box::new(err)),
        ))),
        Err(err) => Err(err),
    }
}

fn parse_body_block_assignment<'a>(
    t: Input<'a>,
) -> IResult<Input<'a>, Body, ParseError<Input<'a>>> {
    match tuple((
        token_type(Token::Ident("".to_string())),
        token(Token::Assign),
        parse_expr,
        alt((
            token(Token::Newline),
            peek(token(Token::Delim(Box::new(Delimiter::CloseBrace)))),
        )),
    ))(t)
    {
        Ok((t, (Token::Ident(target_name), _, expr, _))) => {
            if let Some(name) = make_symbol(target_name) {
                Ok((t, Body::Assignment(name, expr)))
            } else {
                Err(Err::Error(ParseError::new(t, ErrorKind::EmptyName, None)))
            }
        }
        Ok((t, (_, _, _, _))) => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidLexicalDefine,
            None,
        ))),
        Err(Err::Error(err)) => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidLexicalDefine,
            Some(Box::new(err)),
        ))),
        Err(err) => Err(err),
    }
}

fn parse_body_block<'a>(t: Input<'a>) -> IResult<Input<'a>, Vec<Body>, ParseError<Input<'a>>> {
    let p = match delimited(
        token(Token::Delim(Box::new(Delimiter::OpenBrace))),
        tuple((
            many0(alt((
                map(parse_expr, |e| Some(Body::Expr(Box::new(e)))),
                map(parse_body_block_lexical_define, |ld| Some(ld)),
                map(parse_body_block_assignment, |a| Some(a)),
                map(token(Token::Newline), |_| None),
            ))),
            opt(parse_body_block_return),
        )),
        token(Token::Delim(Box::new(Delimiter::CloseBrace))),
    )(t)
    {
        Ok((t, (body, Some(ret)))) => {
            let mut body: Vec<Body> = body
                .into_iter()
                .filter(Option::is_some)
                .map(|o| o.unwrap())
                .collect();
            body.push(ret);
            Ok((t, body))
        }
        Ok((t, (body, _))) => Ok((
            t,
            body.into_iter()
                .filter(Option::is_some)
                .map(|o| o.unwrap())
                .collect(),
        )),
        Err(err) => Err(err),
    };
    println!("parse_defproc_body() = {:?}", p);
    p
}

fn parse_expr_paren<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match tuple((
        token(Token::Delim(Box::new(Delimiter::OpenParen))),
        parse_expr_1,
        token(Token::Delim(Box::new(Delimiter::CloseParen))),
    ))(t)
    {
        Ok((t, (_, expr, _))) => Ok((t, expr)),
        Err(err) => Err(err),
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
            if let Some(name) = make_symbol(name) {
                Ok((t, Expr::Symbol(name)))
            } else {
                Err(Err::Error(ParseError::new(t, ErrorKind::EmptyName, None)))
            }
        }
        Ok((_, _)) => unreachable!(),
        Err(Err::Error(err)) => {
            match peek(token(Token::Delim(Box::new(Delimiter::OpenParen))))(t) {
                Ok((t, _)) => parse_expr_paren(t),
                Err(_) => Err(Err::Error(ParseError::new(
                    t,
                    ErrorKind::NotAnExprTerm,
                    Some(Box::new(err)),
                ))),
            }
        }
        Err(err) => Err(err),
    }
}

fn parse_expr_op_level1_subexpr<'a>(
    t: Input<'a>,
) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    parse_expr_term(t)
}

// '*', '/', '%'
fn parse_expr_op_level1<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match tuple((
        parse_expr_op_level1_subexpr,
        alt((
            token(Token::Op(Box::new(BinOp::Asterisk))),
            token(Token::Op(Box::new(BinOp::Slash))),
            token(Token::Op(Box::new(BinOp::Percent))),
        )),
        parse_expr_op_level1_subexpr,
    ))(t)
    {
        Ok((t, (expr1, Token::Op(op), expr2))) => match **op {
            BinOp::Asterisk => Ok((t, Expr::Op2(Op2::Mul, Box::new(expr1), Box::new(expr2)))),
            BinOp::Slash => Ok((t, Expr::Op2(Op2::Div, Box::new(expr1), Box::new(expr2)))),
            BinOp::Percent => Ok((t, Expr::Op2(Op2::Mod, Box::new(expr1), Box::new(expr2)))),
            _ => unreachable!(),
        },
        Ok((_, (_, _, _))) => unreachable!(),
        Err(err) => Err(err),
    }
}

fn parse_expr_op_level2_subexpr<'a>(
    t: Input<'a>,
) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    alt((parse_expr_op_level1, parse_expr_term))(t)
}

// '+', '-'
fn parse_expr_op_level2<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match tuple((
        parse_expr_op_level2_subexpr,
        alt((
            token(Token::Op(Box::new(BinOp::Plus))),
            token(Token::Op(Box::new(BinOp::Minus))),
        )),
        parse_expr_op_level2_subexpr,
    ))(t)
    {
        Ok((t, (expr1, Token::Op(op), expr2))) => match **op {
            BinOp::Plus => Ok((t, Expr::Op2(Op2::Add, Box::new(expr1), Box::new(expr2)))),
            BinOp::Minus => Ok((t, Expr::Op2(Op2::Sub, Box::new(expr1), Box::new(expr2)))),
            _ => unreachable!(),
        },
        Ok((_, (_, _, _))) => unreachable!(),
        Err(err) => Err(err),
    }
}

fn parse_expr_op_level3_subexpr<'a>(
    t: Input<'a>,
) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    alt((parse_expr_op_level2, parse_expr_op_level1, parse_expr_term))(t)
}

// '>', '<', '>=', '<=', '=='
fn parse_expr_op_level3<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match tuple((
        parse_expr_op_level3_subexpr,
        alt((
            token(Token::Op(Box::new(BinOp::Gt))),
            token(Token::Op(Box::new(BinOp::Lt))),
            token(Token::Op(Box::new(BinOp::Gte))),
            token(Token::Op(Box::new(BinOp::Lte))),
            token(Token::Op(Box::new(BinOp::Eq))),
        )),
        parse_expr_op_level3_subexpr,
    ))(t)
    {
        Ok((t, (expr1, Token::Op(op), expr2))) => match **op {
            BinOp::Gt => Ok((t, Expr::Op2(Op2::Gt, Box::new(expr1), Box::new(expr2)))),
            BinOp::Lt => Ok((t, Expr::Op2(Op2::Lt, Box::new(expr1), Box::new(expr2)))),
            BinOp::Gte => Ok((t, Expr::Op2(Op2::Gte, Box::new(expr1), Box::new(expr2)))),
            BinOp::Lte => Ok((t, Expr::Op2(Op2::Lte, Box::new(expr1), Box::new(expr2)))),
            BinOp::Eq => Ok((t, Expr::Op2(Op2::Eq, Box::new(expr1), Box::new(expr2)))),
            _ => unreachable!(),
        },
        Ok((_, (_, _, _))) => unreachable!(),
        Err(err) => Err(err),
    }
}

fn parse_expr_1<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    match tuple((
        parse_expr_term,
        peek(alt((
            // token(Token::Delim(Box::new(Delimiter::OpenParen))),
            token(Token::Delim(Box::new(Delimiter::CloseParen))),
            token(Token::Delim(Box::new(Delimiter::CloseBrace))),
            token(Token::Newline),
            token(Token::Eof),
        ))),
    ))(t)
    {
        Ok((t, (expr, _))) => Ok((t, expr)),
        Err(_) => alt((
            parse_expr_op_level3,
            parse_expr_op_level2,
            parse_expr_op_level1,
        ))(t),
    }
}

fn parse_expr_if<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    let p = match tuple((
        token(Token::Keyword(Box::new(Keyword::If))),
        alt((parse_expr, parse_expr_term)),
        parse_body_block,
        token(Token::Keyword(Box::new(Keyword::Else))),
        parse_body_block,
    ))(t)
    {
        Ok((t, (_, cond_clause, true_clause, _, false_clause))) => Ok((
            t,
            Expr::If(Box::new(cond_clause), true_clause, false_clause),
        )),
        Err(err) => Err(err),
    };
    println!("parse_expr_if() = {:?}", p);
    p
}

fn parse_expr<'a>(t: Input<'a>) -> IResult<Input<'a>, Expr, ParseError<Input<'a>>> {
    let p = alt((parse_expr_1, parse_expr_if))(t);
    println!("parse_expr() = {:?}", p);
    p
}

fn parse_global_define<'a>(t: Input<'a>) -> IResult<Input<'a>, SyntaxTree, ParseError<Input<'a>>> {
    match tuple((
        token(Token::Keyword(Box::new(Keyword::Let))),
        token_type(Token::Ident("".to_string())),
        token(Token::Assign),
        parse_expr,
        alt((token(Token::Newline), peek(token(Token::Eof)))),
    ))(t)
    {
        Ok((t, (_, Token::Ident(target_name), _, expr, _))) => {
            if let Some(name) = make_symbol(target_name) {
                Ok((t, SyntaxTree::GlobalDefine(name, expr)))
            } else {
                Err(Err::Error(ParseError::new(t, ErrorKind::EmptyName, None)))
            }
        }
        Ok((t, (_, _, _, _, _))) => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidGlobalDefine,
            None,
        ))),
        Err(Err::Error(err)) => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidGlobalDefine,
            Some(Box::new(err)),
        ))),
        Err(err) => Err(err),
    }
}

fn parse_defproc_args<'a>(t: Input<'a>) -> IResult<Input<'a>, Vec<Arg>, ParseError<Input<'a>>> {
    match tuple((
        token(Token::Delim(Box::new(Delimiter::OpenParen))),
        token(Token::Delim(Box::new(Delimiter::CloseParen))),
    ))(t)
    {
        Ok((t, (_, _))) => Ok((t, vec![])),
        Err(err) => Err(err),
    }
}

fn parse_defproc<'a>(t: Input<'a>) -> IResult<Input<'a>, SyntaxTree, ParseError<Input<'a>>> {
    let p = match tuple((
        token(Token::Keyword(Box::new(Keyword::Proc))),
        token_type(Token::Ident("".to_string())),
        parse_defproc_args,
        parse_body_block,
        alt((token(Token::Newline), peek(token(Token::Eof)))),
    ))(t)
    {
        Ok((t, (_, Token::Ident(name), args, body, _))) => {
            Ok((t, SyntaxTree::DefProc(Name(name.to_string()), args, body)))
        }
        Ok((t, (_, _, _, _, _))) => Err(Err::Error(ParseError::new(
            t,
            ErrorKind::InvalidDefProc,
            None,
        ))),
        Err(err) => Err(err),
    };
    println!("parse_defproc() = {:?}", p);
    p
}

type Parse1Result<'a> = IResult<Input<'a>, Option<SyntaxTree>, ParseError<Input<'a>>>;

fn parse_1<'a>(t: Input<'a>) -> Parse1Result<'a> {
    let p = alt((
        // value(None, token_type_of(Token::LineComment("".to_string()))),
        map(token(Token::Newline), |_| None),
        map(parse_global_define, |ga| Some(ga)),
        map(parse_defproc, |f| Some(f)),
    ))(t);
    println!("parse_1() = {:?}", p);
    p
}

type ParseResult<'a> = IResult<Input<'a>, Vec<SyntaxTree>, ParseError<Input<'a>>>;

pub fn parse<'a>(t: Input<'a>) -> ParseResult<'a> {
    match all_consuming(tuple((many0(parse_1), token(Token::Eof))))(t) {
        Ok((rest, (stvec, _))) => Ok((
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
            SyntaxTree::GlobalDefine(Symbol::Var(Name("a".to_string())), Expr::Float(42.0)),
            "let a = 42.0",
        );
    }

    #[test]
    fn test_parse_expr_simple_op() {
        test_parse_1(
            SyntaxTree::GlobalDefine(
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
            SyntaxTree::GlobalDefine(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Add,
                    Box::new(Expr::Float(1.0)),
                    Box::new(Expr::Float(2.0)),
                ),
            ),
            "let a = 1.0 + 2.0",
        );
        test_parse_1(
            SyntaxTree::GlobalDefine(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Mul,
                    Box::new(Expr::Float(1.0)),
                    Box::new(Expr::Float(2.0)),
                ),
            ),
            "let a = 1.0 * 2.0",
        );
    }

    #[test]
    fn test_parse_expr_op_multiple_terms() {
        test_parse_1(
            SyntaxTree::GlobalDefine(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Add,
                    Box::new(Expr::Float(1.0)),
                    Box::new(Expr::Op2(
                        Op2::Sub,
                        Box::new(Expr::Float(2.0)),
                        Box::new(Expr::Float(3.0)),
                    )),
                ),
            ),
            "let a = 1.0 + 2.0 + 3.0",
        );
    }

    #[test]
    fn test_parse_expr_op_precedence() {
        test_parse_1(
            SyntaxTree::GlobalDefine(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Eq,
                    Box::new(Expr::Float(1.0)),
                    Box::new(Expr::Op2(
                        Op2::Add,
                        Box::new(Expr::Float(2.0)),
                        Box::new(Expr::Float(3.0)),
                    )),
                ),
            ),
            "let a = 1.0 == 2.0 + 3.0",
        );

        test_parse_1(
            SyntaxTree::GlobalDefine(
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
            SyntaxTree::GlobalDefine(
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
            SyntaxTree::GlobalDefine(
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

    #[test]
    fn test_parse_expr_op_precedence_with_paren() {
        test_parse_1(
            SyntaxTree::GlobalDefine(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Add,
                    Box::new(Expr::Op2(
                        Op2::Eq,
                        Box::new(Expr::Float(1.0)),
                        Box::new(Expr::Float(2.0)),
                    )),
                    Box::new(Expr::Op2(
                        Op2::Mul,
                        Box::new(Expr::Float(3.0)),
                        Box::new(Expr::Float(4.0)),
                    )),
                ),
            ),
            "let a = (1.0 == 2.0) + 3.0 * 4.0",
        );
        test_parse_1(
            SyntaxTree::GlobalDefine(
                Symbol::Var(Name("a".to_string())),
                Expr::Op2(
                    Op2::Mul,
                    Box::new(Expr::Op2(
                        Op2::Add,
                        Box::new(Expr::Op2(
                            Op2::Eq,
                            Box::new(Expr::Float(1.0)),
                            Box::new(Expr::Float(2.0)),
                        )),
                        Box::new(Expr::Float(3.0)),
                    )),
                    Box::new(Expr::Float(4.0)),
                ),
            ),
            "let a = ((1.0 == 2.0) + 3.0) * 4.0",
        );
    }

    #[test]
    fn test_parse_fn_main() {
        test_parse_1(
            SyntaxTree::DefProc(Name("main".to_string()), vec![], vec![Body::Return(None)]),
            "proc main() { return }",
        );

        test_parse_1(
            SyntaxTree::DefProc(
                Name("main".to_string()),
                vec![],
                vec![
                    Body::Assignment(
                        Symbol::State(Name("px".to_string())),
                        Expr::Op2(
                            Op2::Add,
                            Box::new(Expr::Symbol(Symbol::State(Name("px".to_string())))),
                            Box::new(Expr::Float(5.0)),
                        ),
                    ),
                    Body::Return(None),
                ],
            ),
            r###"
            proc main() {
              $px = $px + 5.0
              return
            }
            "###,
        );
    }

    #[test]
    fn test_parse_fn_if_expr() {
        test_parse_1(
            SyntaxTree::DefProc(
                Name("main".to_string()),
                vec![],
                vec![Body::LexicalDefine(
                    Symbol::Var(Name("dp".to_string())),
                    Expr::If(
                        Box::new(Expr::Symbol(Symbol::State(Name("input_slow".to_string())))),
                        vec![Body::Expr(Box::new(Expr::Float(4.0)))],
                        vec![Body::Expr(Box::new(Expr::Float(7.0)))],
                    ),
                )],
            ),
            "proc main() { let dp = if $input_slow { 4.0 } else { 7.0 } }",
        );
    }
}
