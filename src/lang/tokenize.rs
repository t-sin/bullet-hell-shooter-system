use nom::{
    character::complete::{char, digit1, none_of},
    combinator::opt,
    multi::many0,
    sequence::tuple,
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Float(f32);
impl Eq for Float {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Float(Float),
    String(String),
}

pub fn tokenize_float(s: &str) -> IResult<&str, Token> {
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

pub fn tokenize(s: &str) -> IResult<&str, Vec<Token>> {
    let tokens: Vec<Token> = Vec::new();

    Ok((s, tokens))
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
}
