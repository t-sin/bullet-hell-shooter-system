mod codegen;
mod parse;
mod tokenize;

use nom::{error::ErrorKind, Err};

use lang_component::vm::Inst;

use crate::{
    codegen::{codegen, CodegenError, CodegenState},
    parse::{parse, ParserError},
    tokenize::tokenize,
};

#[derive(Debug)]
pub struct TokenizerError {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum CompileError {
    TokenizeError(TokenizerError),
    ParseError(ParserError),
    CodegenError(CodegenError),
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
            Ok((_, stvec)) => match codegen(stvec) {
                Ok(state) => Ok(CompileResult::from_state(state)),
                Err(err) => Err(CompileError::CodegenError(err)),
            },
            Err(Err::Error(err)) => Err(CompileError::ParseError(err.purge_input().unwrap())),
            Err(err) => panic!("parse error = {:?}", err),
        },
        Err(Err::Error(err)) => Err(CompileError::TokenizeError(TokenizerError {
            kind: err.code,
        })),
        Err(err) => panic!("tokenizer error = {:?}", err),
    }
}
