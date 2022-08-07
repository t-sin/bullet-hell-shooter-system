use lang_component::vm::{Data, Inst};

pub type ExpectedTypeName = String;

#[derive(Debug)]
pub enum RuntimeError {
    OutOfCode(usize, Vec<Inst>),
    StackUnderflow,
    TypeMismatched(Data, ExpectedTypeName),
    UnknownStateName(String),
}
