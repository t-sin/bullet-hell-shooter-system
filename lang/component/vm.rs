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
