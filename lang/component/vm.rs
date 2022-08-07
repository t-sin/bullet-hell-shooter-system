#[derive(Debug, Copy, Clone)]
pub enum Data {
    Float(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inst {
    // system words
    Term, // tells now the program reaches the end of program successfully
    // embedded data
    Float(f32),
    // state accessors
    Get(String),
    Set(String),
    // arithmetics
    Add,
    Sub,
    Mul,
    // comparators
    EqInt,
    EqFloat,
    Lt,
    // logical
    Not,
    // stack operations
    Dup,
    // control flows
    JumpIfZero(usize),
    Jump(usize),
}
