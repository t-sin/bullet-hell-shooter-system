use std::collections::VecDeque;

use ggez::{Context, GameError, GameResult};

use lang_compiler::BulletCode;
use lang_component::{
    bullet::{BulletColor, BulletType},
    vm::{Data, OperationQuery},
};
use lang_vm::VM;

use super::{bullet::BulletState, bullet_codes::BulletCodes, SceneDrawable};

pub struct Player {
    pub state: BulletState,
    pub vm: VM,
}

impl Player {
    pub fn new(codes: &BulletCodes) -> Self {
        let state = BulletState::new(200.0, 400.0, BulletType::Player, BulletColor::White);
        let mut vm = VM::new();

        if let Some(bc) = codes.by_name.get("player") {
            vm.set_code(bc.code.clone());
            vm.set_memory(bc.initial_memory.clone());
        }

        Self { state, vm }
    }

    pub fn update(&mut self, op_queue: &mut VecDeque<OperationQuery>) -> GameResult<()> {
        if let Err(err) = self.vm.run(0, &mut self.state, op_queue) {
            return Err(GameError::CustomError(format!("error = {:?}", err)));
        }

        Ok(())
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        if let Err(err) = self.state.draw(ctx) {
            return Err(GameError::CustomError(format!("error = {:?}", err)));
        }

        Ok(())
    }
}

pub trait OperationProcessor {
    fn fire(
        &mut self,
        x: f32,
        y: f32,
        r#type: BulletType,
        color: BulletColor,
        params: Vec<Data>,
        bullet_code: &BulletCode,
    ) -> bool;

    fn kill(&mut self, _id: usize) {}
}
