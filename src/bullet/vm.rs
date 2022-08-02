use super::Bullet;

enum Data {
    Float(f32),
    Int(i32),
}

pub enum Inst {
    Nop,
}

pub struct VM {
    pc: usize,
    code: Vec<Inst>,
    stack: Vec<Data>,
    memory: Vec<u8>,
}

impl VM {
    pub fn new(code: Vec<Inst>, memory: Vec<u8>) -> Self {
        VM {
            pc: 0,
            code,
            stack: Vec::new(),
            memory,
        }
    }

    pub fn run(&mut self, bullet: &mut Bullet) {}
}
