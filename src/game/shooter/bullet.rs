use std::{collections::VecDeque, rc::Rc};

use ggez::{
    graphics::{
        self,
        // BlendMode, Canvas,
        Color,
        DrawMode,
        DrawParam,
        Mesh,
        MeshBuilder,
    },
    mint::Point2,
    Context, GameError, GameResult,
};
use glam;

use lang_compiler::BulletCode;
use lang_component::{
    bullet::{BulletColor, BulletId, BulletType, StateIO, StateId},
    vm::{Data, OperationQuery},
};
use lang_vm::{SuspendingReason, VM};

use super::{shooter::Objects, SceneDrawable};
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

#[derive(Debug)]
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

pub struct Bullet {
    pub enabled: bool,
    pub visible: bool,
    pub input: InputState,
    pub pos: glam::Vec2,
    pub appearance: Appearance,
    pub vm: VM,
}

impl Bullet {
    pub fn new(x: f32, y: f32, atype: BulletType, acolor: BulletColor, bc: Rc<BulletCode>) -> Self {
        let mut vm = VM::new();
        vm.set_code(bc.code.clone());
        vm.set_memory(bc.initial_memory.clone());

        Self {
            enabled: false,
            visible: false,
            input: InputState::default(),
            pos: glam::vec2(x, y),
            appearance: Appearance::new(atype, acolor),
            vm,
        }
    }

    pub fn update(
        &mut self,
        id: BulletId,
        objects: &Objects,
        op_queue: &mut VecDeque<OperationQuery>,
    ) -> GameResult<()> {
        let mut reason = self.vm.start(0, op_queue);

        loop {
            match reason {
                Ok(SuspendingReason::Terminated) => break,
                Ok(SuspendingReason::Running) => unreachable!(),
                Ok(SuspendingReason::ToReadState(bid, sid)) => match bid {
                    BulletId::Itself => self.vm.push_data(self.read(&bid, &sid)),
                    BulletId::Player => {
                        let d = if id == BulletId::Player {
                            self.read(&bid, &sid)
                        } else {
                            let p = objects.player.borrow();
                            p.read(&bid, &sid)
                        };
                        self.vm.push_data(d);
                    }
                    _ => todo!(),
                },
                Ok(SuspendingReason::ToWriteState(bid, sid, d)) => {
                    self.write(&bid, &sid, d);
                }
                Err(err) => return Err(GameError::CustomError(format!("error = {:?}", err))),
            }

            reason = self.vm.resume(0, op_queue);
        }

        Ok(())
    }
}

impl StateIO for Bullet {
    fn read(&self, _bid: &BulletId, sid: &StateId) -> Data {
        match sid {
            StateId::PosX => Data::Float(self.pos.x),
            StateId::PosY => Data::Float(self.pos.y),
            StateId::InputUp => Data::Bool(self.input.up),
            StateId::InputDown => Data::Bool(self.input.down),
            StateId::InputLeft => Data::Bool(self.input.left),
            StateId::InputRight => Data::Bool(self.input.right),
            StateId::InputShot => Data::Bool(self.input.shot),
            StateId::InputSlow => Data::Bool(self.input.slow),
            StateId::Enabled => Data::Bool(self.enabled),
        }
    }

    fn write(&mut self, _bid: &BulletId, sid: &StateId, d: Data) {
        match sid {
            StateId::PosX => {
                if let Data::Float(f) = d {
                    self.pos.x = f
                }
            }
            StateId::PosY => {
                if let Data::Float(f) = d {
                    self.pos.y = f
                }
            }
            StateId::InputUp => {
                if let Data::Bool(b) = d {
                    self.input.up = b
                }
            }
            StateId::InputDown => {
                if let Data::Bool(b) = d {
                    self.input.down = b
                }
            }
            StateId::InputLeft => {
                if let Data::Bool(b) = d {
                    self.input.left = b
                }
            }
            StateId::InputRight => {
                if let Data::Bool(b) = d {
                    self.input.right = b
                }
            }
            StateId::InputShot => {
                if let Data::Bool(b) = d {
                    self.input.shot = b
                }
            }
            StateId::InputSlow => {
                if let Data::Bool(b) = d {
                    self.input.slow = b
                }
            }
            StateId::Enabled => {
                if let Data::Bool(b) = d {
                    self.enabled = b
                }
            }
        }
    }
}

impl SceneDrawable for Bullet {
    fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult<()> {
        let color = match self.appearance.color {
            BulletColor::White => Color::from_rgb(255, 255, 255),
        };
        let pos = self.pos;
        let param = DrawParam::default()
            .color(color)
            .offset([-constant::SHOOTER_OFFSET_X, -constant::SHOOTER_OFFSET_Y]);

        match self.appearance.r#type {
            BulletType::Player => {
                static POINTS: [[f32; 2]; 3] = [[8.0, 7.0], [0.0, -12.0], [-8.0, 7.0]];

                let dest = glam::vec2(0.0, 0.0) + pos;
                let param = param.dest::<Point2<f32>>(dest.into());
                let mut mb = MeshBuilder::new();
                let mesh = mb.polygon(DrawMode::stroke(1.5), &POINTS, color)?.build();
                let mesh = Mesh::from_data(ctx, mesh);
                canvas.draw(&mesh, param);

                let mut mb = MeshBuilder::new();
                let hit_area = mb
                    .circle(DrawMode::stroke(1.0), [0.0, 0.0], 5.0, 1.0, color)?
                    //.circle(DrawMode::stroke(1.0), glam::vec2(0.0, 0.0), 3.0, 1.0, color)?
                    .build();
                let hit_area = Mesh::from_data(ctx, hit_area);
                canvas.draw(&hit_area, param);
            }
            BulletType::Bullet1 => {
                //                let dest = glam::vec2(-5.0, -5.0) + pos;
                //                let param = param.dest::<Point2<f32>>(dest.into());
                let mut mb = MeshBuilder::new();
                let bullet = mb
                    .circle(DrawMode::stroke(1.0), pos, 4.0, 1.0, color)?
                    .build();
                let bullet = Mesh::from_data(ctx, bullet);
                canvas.draw(&bullet, param);
            }
        };

        Ok(())
    }
}
