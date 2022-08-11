use std::collections::VecDeque;

use ggez::{
    graphics::{
        self,
        // BlendMode, Canvas,
        Color,
        DrawMode,
        DrawParam,
        MeshBuilder,
    },
    mint::Point2,
    Context, GameResult,
};
use glam;

use lang_component::vm::Inst;
use lang_vm::{
    bullet::{BulletColor, BulletType, Operation, ReadState, WriteState},
    VM,
};

use super::SceneDrawable;
use crate::constant;

pub struct Appearance {
    pub r#type: BulletType,
    pub color: BulletColor,
}

impl Appearance {
    pub fn new(r#type: BulletType, color: BulletColor) -> Self {
        Self { r#type, color }
    }
}

pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub shot: bool,
    pub slow: bool,
}

impl Default for InputState {
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
    pub input: InputState,
    pub pos: glam::Vec2,
}

impl ReadState for BulletState {
    fn pos_x(&self) -> f32 {
        self.pos.x
    }
    fn pos_y(&self) -> f32 {
        self.pos.y
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

impl WriteState for BulletState {
    fn set_pos_x(&mut self, f: f32) {
        self.pos.x = f;
    }
    fn set_pos_y(&mut self, f: f32) {
        self.pos.y = f;
    }
    fn set_input_up(&mut self, b: bool) {
        self.input.up = b;
    }
    fn set_input_down(&mut self, b: bool) {
        self.input.down = b;
    }
    fn set_input_left(&mut self, b: bool) {
        self.input.left = b;
    }
    fn set_input_right(&mut self, b: bool) {
        self.input.right = b;
    }
    fn set_input_shot(&mut self, b: bool) {
        self.input.shot = b;
    }
    fn set_input_slow(&mut self, b: bool) {
        self.input.slow = b;
    }
}

pub struct Bullet {
    pub next: Option<usize>,
    pub state: BulletState,
    vm: VM,
    pub appearance: Appearance,
}

impl Bullet {
    pub fn new(x: f32, y: f32, a: Appearance, next: Option<usize>) -> Self {
        Self {
            next,
            state: BulletState {
                enabled: false,
                input: InputState::default(),
                pos: glam::vec2(x, y),
            },
            vm: VM::new(),
            appearance: a,
        }
    }

    pub fn set_code(&mut self, code: Vec<Inst>) {
        self.vm.set_code(code);
    }

    pub fn set_memory(&mut self, memory: Vec<u8>) {
        self.vm.set_memory(memory);
    }

    pub fn update(&mut self, ops_queue: &mut VecDeque<Operation>) {
        if let Err(err) = VM::run(&mut self.vm, &mut self.state, ops_queue) {
            println!("VM runtime error: {:?}", err);
        }
    }
}

impl SceneDrawable for Bullet {
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let color = match self.appearance.color {
            BulletColor::White => Color::from_rgb(255, 255, 255),
        };
        let pos = self.state.pos;
        let param = DrawParam::default()
            .color(color)
            .offset([-constant::SHOOTER_OFFSET_X, -constant::SHOOTER_OFFSET_Y]);

        match self.appearance.r#type {
            BulletType::Player => {
                static POINTS: [[f32; 2]; 3] = [[8.0, 7.0], [0.0, -12.0], [-8.0, 7.0]];

                let dest = glam::vec2(0.0, 0.0) + pos;
                let param = param.dest::<Point2<f32>>(dest.into());
                let mesh = MeshBuilder::new()
                    .polygon(DrawMode::stroke(1.5), &POINTS, color)?
                    .build(ctx)?;
                graphics::draw(ctx, &mesh, param)?;

                let hit_area = MeshBuilder::new()
                    .circle(DrawMode::stroke(1.0), [0.0, 0.0], 5.0, 1.0, color)?
                    //.circle(DrawMode::stroke(1.0), glam::vec2(0.0, 0.0), 3.0, 1.0, color)?
                    .build(ctx)?;
                graphics::draw(ctx, &hit_area, param)?;
            }
            BulletType::Bullet1 => {
                let dest = glam::vec2(-5.0, -5.0) + pos;
                let param = param.dest::<Point2<f32>>(dest.into());
                let bullet = MeshBuilder::new()
                    .circle(DrawMode::stroke(1.0), pos, 8.0, 1.0, color)?
                    .build(ctx)?;
                graphics::draw(ctx, &bullet, param)?;
            }
        };

        Ok(())
    }
}
