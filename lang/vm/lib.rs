pub mod bullet;
pub mod error;
mod r#macro;
pub mod run;

pub use error::*;
pub use run::*;

use lang_component::vm::{Data, Inst};

pub struct VM {
    pub pc: usize,
    pub code: Box<[Inst]>,
    pub stack: Vec<Data>,
    pub rstack: Vec<usize>,
    pub memory: Vec<u8>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            pc: 0,
            code: Vec::new().into(),
            stack: Vec::new(),
            rstack: Vec::new(),
            memory: Vec::from([0; 128]),
        }
    }

    pub fn set_code(&mut self, code: Vec<Inst>) {
        self.code = code.into();
    }

    pub fn set_memory(&mut self, memory: Vec<u8>) {
        self.memory = memory;
    }
}
