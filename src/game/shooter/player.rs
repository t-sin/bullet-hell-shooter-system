use std::collections::VecDeque;

use ggez::{graphics, Context, GameError, GameResult};

use lang_compiler::BulletCode;
use lang_component::{
    bullet::{BulletColor, BulletId, BulletType, Reference, StateId},
    vm::{Data, OperationQuery},
};
use lang_vm::{SuspendingReason, VM};

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
        let mut reason = self.vm.start(0, &mut self.state, op_queue);

        loop {
            match reason {
                Ok(SuspendingReason::Terminated) => break,
                Ok(SuspendingReason::Running) => unreachable!(),
                Ok(SuspendingReason::ToReferenceABullet(bullet, state)) => {
                    self.vm.push_data(self.refer(&bullet, &state));
                }
                Err(err) => return Err(GameError::CustomError(format!("error = {:?}", err))),
            }

            reason = self.vm.resume(0, &mut self.state, op_queue);
        }

        Ok(())
    }

    pub fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult<()> {
        if let Err(err) = self.state.draw(ctx, canvas) {
            return Err(GameError::CustomError(format!("error = {:?}", err)));
        }

        Ok(())
    }
}

impl Reference for Player {
    fn refer(&self, bid: &BulletId, sid: &StateId) -> Data {
        if !matches!(bid, BulletId::Player) {
            panic!("I'm not a {:?}", bid);
        }

        match sid {
            StateId::PosX => Data::Float(self.state.pos.x),
            StateId::PosY => Data::Float(self.state.pos.y),
            StateId::InputUp => Data::Bool(self.state.input.up),
            StateId::InputDown => Data::Bool(self.state.input.down),
            StateId::InputLeft => Data::Bool(self.state.input.left),
            StateId::InputRight => Data::Bool(self.state.input.right),
            StateId::InputShot => Data::Bool(self.state.input.shot),
            StateId::InputSlow => Data::Bool(self.state.input.slow),
            StateId::Enabled => Data::Bool(self.state.enabled),
        }
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
