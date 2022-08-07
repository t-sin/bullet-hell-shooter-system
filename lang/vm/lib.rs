pub mod error;
pub mod run;

use error::*;
use run::*;

use lang_component::vm::{Data, Inst};

pub struct VM {
    pub pc: usize,
    pub code: Box<[Inst]>,
    pub stack: Vec<Data>,
    pub memory: Vec<u8>,
}

pub trait State {
    fn pos_x(&self) -> f32;
    fn set_pos_x(&mut self, f: f32);
    fn pos_y(&self) -> f32;
    fn set_pos_y(&mut self, f: f32);

    fn input_up(&self) -> bool;
    fn input_down(&self) -> bool;
    fn input_left(&self) -> bool;
    fn input_right(&self) -> bool;
    fn input_shot(&self) -> bool;
    fn input_slow(&self) -> bool;
}
