use lang_component::{
    syntax::Type,
    vm::{Data, Inst},
};

pub type ExpectedTypeName = String;

#[derive(Debug)]
pub enum RuntimeError {
    OutOfCode(i32, Vec<Inst>),
    OutOfMemory(usize, Type),
    StackUnderflow,
    TypeMismatched(Data, ExpectedTypeName),
    UnknownStateName(String),
    CannotDecodeFloat(std::array::TryFromSliceError),
    ReturnStackUnderflow,
}
