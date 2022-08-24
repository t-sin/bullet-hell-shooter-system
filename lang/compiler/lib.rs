mod codegen;
mod parse;
mod tokenize;

use std::{collections::HashMap, rc::Rc};

use nom::{error::ErrorKind, Err};

use lang_component::{syntax::Signature, vm::Inst};

use crate::{
    codegen::{codegen, CodegenError, CodegenResult},
    parse::{parse, ParserError},
    tokenize::tokenize,
};

#[derive(Debug, Clone)]
pub struct BulletCode {
    pub id: usize,
    pub name: String,
    pub code: Rc<Vec<Inst>>,
    pub initial_memory: Vec<u8>,
    pub signature: Signature,
}

impl BulletCode {
    #[allow(dead_code)]
    fn new(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            code: Rc::new(Vec::new()),
            initial_memory: Vec::new(),
            signature: Signature::new(Vec::new(), None),
        }
    }
}

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
    pub signature: Signature,
}

impl CompileResult {
    fn new(code: Vec<Inst>, memory: Vec<u8>, signature: Signature) -> Self {
        Self {
            code,
            memory,
            signature,
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
    code_vec: &Vec<Rc<BulletCode>>,
) -> Result<CompileResult, CompileError> {
    match tokenize(&source[..]) {
        Ok((_, tokens)) => match parse(&tokens[..]) {
            Ok((_, stvec)) => match codegen(stvec, ObjectStates(state_map), code_vec) {
                Ok(CodegenResult {
                    code,
                    memory,
                    signature,
                }) => Ok(CompileResult::new(code, memory, signature)),
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
