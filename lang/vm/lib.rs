pub mod error;
pub mod interpret;
mod r#macro;

use lang_component::{
    bullet::{BulletId, StateId},
    vm::Data,
};

pub use error::*;
pub use interpret::*;

pub struct Continuation {
    pub pc: usize,
    pub stack: Vec<Data>,
    pub rstack: Vec<usize>,
    pub memory: Vec<u8>,
}

impl Continuation {
    pub fn new() -> Self {
        Self {
            pc: 0,
            stack: vec![],
            rstack: vec![],
            memory: Vec::from([0; 64]),
        }
    }
}

pub enum SuspendingReason {
    Terminated,
    Running,
    ToReadState(BulletId, StateId),
    ToWriteState(BulletId, StateId, Data),
}
