pub mod bullet;
pub mod error;
pub mod run;

pub use error::*;
pub use run::*;

use lang_component::vm::{Data, Inst};

pub struct VM {
    pub pc: usize,
    pub code: Box<[Inst]>,
    pub stack: Vec<Data>,
    pub memory: Vec<u8>,
}
