use crate::{
    bullet::{BulletColor, BulletId, BulletType, StateId},
    syntax::Type,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Float(f32),
    Bool(bool),
    String(String),
}
impl Eq for Data {}

impl Data {
    pub fn r#type(&self) -> Type {
        match self {
            Data::Float(_) => Type::Float,
            Data::Bool(_) => Type::Bool,
            Data::String(_) => Type::String,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalOperation {
    Fire(usize),
    Die,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationQuery {
    Fire(usize, (f32, f32), BulletType, BulletColor, Vec<Data>),
    Die(BulletId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inst {
    // system words
    Term, // tells now the program reaches the end of program successfully
    Operate(ExternalOperation),
    // memory operations
    Read(usize, Type),
    Write(usize),
    ReadGlobal(usize, Type),
    WriteGlobal(usize),
    // embedded data
    Float(f32),
    Bool(bool),
    // state accessors
    RefRead(BulletId, StateId),
    RefWrite(BulletId, StateId),
    // arithmetics
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // comparators
    EqInt,
    EqFloat,
    Gt,
    Lt,
    Gte,
    Lte,
    // logical
    Not,
    LogOr,
    LogAnd,
    // stack operations
    Dup,
    Drop,
    Index,
    // control flows
    JumpIfFalse(i32),
    Jump(i32),
    // procedure call/return
    Call,
    Ret(usize),
}

pub fn get_vm_name(state_name: &str) -> Option<String> {
    match state_name {
        "px" => Some("Pos:X".to_string()),
        "py" => Some("Pos:Y".to_string()),
        "input_up" => Some("Input:Up".to_string()),
        "input_down" => Some("Input:Down".to_string()),
        "input_left" => Some("Input:Left".to_string()),
        "input_right" => Some("Input:Right".to_string()),
        "input_shot" => Some("Input:Shot".to_string()),
        "input_slow" => Some("Input:Slow".to_string()),
        _ => None,
    }
}
