use glam;

mod vm;
use vm::VM;

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

pub struct Bullet {
    pub enabled: bool,
    pub input: Input,
    pub pos: glam::Vec2,
    vm: VM,
    pub appearance: Appearance,
}

impl Bullet {
    pub fn new(x: f32, y: f32, a: Appearance) -> Self {
        Self {
            enabled: false,
            input: Input::default(),
            pos: glam::vec2(x, y),
            vm: VM::new(Vec::new(), Vec::new()),
            appearance: a,
        }
    }
}
