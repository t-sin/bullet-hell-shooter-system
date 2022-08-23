mod codegen;
mod parse;
mod tokenize;

use std::collections::HashMap;

use nom::{error::ErrorKind, Err};

use lang_component::vm::Inst;

use crate::{
    codegen::{codegen, CodegenError, CodegenResult},
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
    fn new(code: Vec<Inst>, memory: Vec<u8>) -> Self {
        Self {
            code: code,
            memory: memory,
        }
    }
}

#[derive(Debug)]
pub struct ObjectStates(pub HashMap<String, usize>);

impl ObjectStates {
    pub fn get_state_id(&self, name: &str) -> Option<usize> {
        self.0.get(name).copied()
    }
}

pub fn compile(
    source: String,
    state_map: HashMap<String, usize>,
) -> Result<CompileResult, CompileError> {
    match tokenize(&source[..]) {
        Ok((_, tokens)) => match parse(&tokens[..]) {
            Ok((_, stvec)) => match codegen(stvec, ObjectStates(state_map)) {
                Ok(CodegenResult {
                    code,
                    initial_memory,
                }) => Ok(CompileResult::new(code, initial_memory)),
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
