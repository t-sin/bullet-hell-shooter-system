use std::collections::VecDeque;

use ggez::{graphics, Context, GameError, GameResult};

use lang_compiler::BulletCode;
use lang_component::{
    bullet::{BulletColor, BulletId, BulletType, StateIO, StateId},
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
        let mut reason = self.vm.start(0, op_queue);

        loop {
            match reason {
                Ok(SuspendingReason::Terminated) => break,
                Ok(SuspendingReason::Running) => unreachable!(),
                Ok(SuspendingReason::ToReadState(bid, sid)) => {
                    self.vm.push_data(self.read(&bid, &sid));
                }
                Ok(SuspendingReason::ToWriteState(bid, sid, d)) => {
                    self.state.write(&bid, &sid, d);
                }
                Err(err) => return Err(GameError::CustomError(format!("error = {:?}", err))),
            }

            reason = self.vm.resume(0, op_queue);
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

impl StateIO for Player {
    fn read(&self, bid: &BulletId, sid: &StateId) -> Data {
        if !matches!(bid, BulletId::Player | BulletId::Itself) {
            panic!("I'm not a {:?}", bid);
        }

        self.state.read(bid, sid)
    }

    fn write(&mut self, bid: &BulletId, sid: &StateId, d: Data) {
        if !matches!(bid, BulletId::Player | BulletId::Itself) {
            panic!("I'm not a {:?}", bid);
        }

        self.state.write(bid, sid, d);
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
