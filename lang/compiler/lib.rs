mod codegen;
mod parse;
mod tokenize;

use nom::{error::ErrorKind, Err};

use lang_component::vm::Inst;

use crate::{
    codegen::{codegen, CodegenState},
    parse::{parse, ParserError},
    tokenize::tokenize,
};

#[derive(Debug)]
pub struct TokenizerError {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub struct CompileError {
    pub tokenize: Option<TokenizerError>,
    pub parse: Option<ParserError>,
    pub codegen: Option<()>,
}

impl CompileError {
    fn new(
        tokenize: Option<TokenizerError>,
        parse: Option<ParserError>,
        codegen: Option<()>,
    ) -> Self {
        Self {
            tokenize,
            parse,
            codegen,
        }
    }
}

#[derive(Debug)]
pub struct CompileResult {
    pub code: Vec<Inst>,
    pub memory: Vec<u8>,
}

impl CompileResult {
    pub fn from_state(cs: CodegenState) -> Self {
        Self {
            code: cs.code,
            memory: cs.memory,
        }
    }
}

pub fn compile(source: String) -> Result<CompileResult, CompileError> {
    match tokenize(&source[..]) {
        Ok((_, tokens)) => match parse(&tokens[..]) {
            Ok((_, stvec)) => Ok(CompileResult::from_state(codegen(stvec))),
            Err(Err::Error(err)) => Err(CompileError::new(None, err.purge_input(), None)),
            Err(err) => panic!("parse error = {:?}", err),
        },
        Err(Err::Error(err)) => Err(CompileError::new(
            Some(TokenizerError { kind: err.code }),
            None,
            None,
        )),
        Err(err) => panic!("tokenizer error = {:?}", err),
    }
}
