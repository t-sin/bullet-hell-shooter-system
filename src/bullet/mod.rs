mod vm;

use vm::VM;

pub struct Pos2 {
    x: f32,
    y: f32,
}

pub struct Bullet {
    pos: Pos2,
    vm: VM,
}

impl Bullet {
    pub fn dummy() -> Self {
        Self {
            pos: Pos2 { x: 0.0, y: 0.0 },
            vm: VM::new(Vec::new(), Vec::new()),
        }
    }
}
