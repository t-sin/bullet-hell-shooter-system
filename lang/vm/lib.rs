pub mod bullet;
pub mod error;
mod r#macro;
pub mod run;

use std::rc::Rc;

use lang_component::vm::{Data, Inst};

pub use error::*;
pub use run::*;

pub struct VM {
    pub pc: usize,
    pub code: Rc<Vec<Inst>>,
    pub stack: Vec<Data>,
    pub rstack: Vec<usize>,
    pub memory: Vec<u8>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            pc: 0,
            code: Rc::new(Vec::new()),
            stack: Vec::new(),
            rstack: Vec::new(),
            memory: Vec::from([0; 128]),
        }
    }

    pub fn set_code(&mut self, code: Rc<Vec<Inst>>) {
        self.code = code;
    }

    pub fn set_memory(&mut self, memory: Vec<u8>) {
        self.memory = memory;
    }
}
