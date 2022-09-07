mod codegen;
mod parse;
mod tokenize;

use std::collections::HashMap;

use nom::{error::ErrorKind, Err};

use lang_component::{
    syntax::{Signature, SyntaxTree},
    vm::Inst,
};

use crate::{
    codegen::{codegen, CodegenError, ProcType, UnresolvedProc},
    parse::{parse, ParserError},
    tokenize::tokenize,
};

#[derive(Debug, Clone)]
pub struct ProcArchetype {
    pub name: String,
    pub r#type: ProcType,
    pub memory: Vec<u8>,
    pub signature: Signature,
    pub offset: usize,
}

impl ProcArchetype {
    pub fn new(name: &str, r#type: ProcType) -> Self {
        Self {
            name: name.to_string(),
            r#type,
            memory: Vec::from([0; 128]),
            signature: Signature::new(Vec::new(), None),
            offset: 0,
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
    pub procs: HashMap<String, ProcArchetype>,
}

impl CompileResult {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            procs: HashMap::new(),
        }
    }
}

pub fn compile(sources: Vec<(String, String)>) -> Result<CompileResult, CompileError> {
    let mut unresolved_procs: Vec<UnresolvedProc> = Vec::new();

    for (filename, source) in sources.iter() {
        let result = tokenize(&source[..]);
        let tokens = match result {
            Ok((_, tokens)) => tokens,
            Err(Err::Error(err)) => {
                return Err(CompileError::TokenizeError(TokenizerError {
                    kind: err.code,
                }))
            }
            Err(err) => panic!("tokenizer error = {:?}", err),
        };

        let result = parse(&tokens[..]);
        let stvec = match result {
            Ok((_, stvec)) => stvec,
            Err(Err::Error(err)) => {
                return Err(CompileError::ParseError(err.purge_input().unwrap()))
            }
            Err(err) => panic!("parse error = {:?}", err),
        };

        match codegen(filename, &stvec) {
            Ok(mut unresolved) => unresolved_procs.append(&mut unresolved),
            Err(err) => return Err(CompileError::CodegenError(err)),
        };
    }

    let mut result = CompileResult::new();
    Ok(result)
}
