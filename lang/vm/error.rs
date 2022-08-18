use lang_component::{
    syntax::Type,
    vm::{Data, Inst},
};

#[derive(Debug)]
pub enum RuntimeError {
    OutOfCode(i32, Vec<Inst>),
    OutOfMemory(usize, Type),
    StackUnderflow,
    TypeMismatched(Data, Type),
    UnknownState(usize),
    CannotDecodeFloat(std::array::TryFromSliceError),
    ReturnStackUnderflow,
}
