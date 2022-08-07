use glam;

use lang_compiler::{compile, CompileError, TokenizerError};
use lang_component::vm::Inst;
use lang_vm::{State, VM};

pub enum BulletType {
    Player,
    Bullet1,
}

pub enum BulletColor {
    White,
}

pub struct Appearance {
    pub r#type: BulletType,
    pub color: BulletColor,
}

impl Appearance {
    pub fn new(r#type: BulletType, color: BulletColor) -> Self {
        Self { r#type, color }
    }
}

pub struct Input {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub shot: bool,
    pub slow: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            shot: false,
            slow: false,
        }
    }
}

pub struct BulletState {
    pub enabled: bool,
    pub input: Input,
    pub pos: glam::Vec2,
}

impl State for BulletState {
    fn pos_x(&self) -> f32 {
        self.pos.x
    }
    fn set_pos_x(&mut self, f: f32) {
        self.pos.x = f;
    }
    fn pos_y(&self) -> f32 {
        self.pos.y
    }
    fn set_pos_y(&mut self, f: f32) {
        self.pos.y = f;
    }

    fn input_up(&self) -> bool {
        self.input.up
    }
    fn input_down(&self) -> bool {
        self.input.down
    }
    fn input_left(&self) -> bool {
        self.input.left
    }
    fn input_right(&self) -> bool {
        self.input.right
    }
    fn input_shot(&self) -> bool {
        self.input.shot
    }
    fn input_slow(&self) -> bool {
        self.input.slow
    }
}

pub struct Bullet {
    pub state: BulletState,
    vm: VM,
    pub appearance: Appearance,
}

impl Bullet {
    pub fn new(x: f32, y: f32, a: Appearance) -> Self {
        Self {
            state: BulletState {
                enabled: false,
                input: Input::default(),
                pos: glam::vec2(x, y),
            },
            vm: VM::new(Vec::new()),
            appearance: a,
        }
    }

    pub fn set_code(&mut self, code: Vec<Inst>) {
        self.vm.set_code(code);
    }

    pub fn update(&mut self) {
        if let Err(err) = VM::run(&mut self.vm, &mut self.state) {
            println!("VM runtime error: {:?}", err);
        }
    }
}
