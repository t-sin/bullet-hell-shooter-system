use std::{cell::RefCell, ops::DerefMut, rc::Rc};

use ggez::{Context, GameError, GameResult};

use lang_component::vm::Inst;
use lang_vm::{
    bullet::{BulletColor, BulletType},
    VM,
};

use super::{bullet::BulletState, SceneDrawable};

pub struct BulletPool {
    pub states: Vec<Rc<RefCell<BulletState>>>,
    pub vms: Vec<VM>,
    pub nexts: Vec<Option<usize>>,
    pub next_disabled: Option<usize>,
}

impl BulletPool {
    pub const BULLET_MAX: usize = 4000;
    const EMPTY_BULLET_CODE: [Inst; 1] = [Inst::Term];

    pub fn new() -> Self {
        let mut states = Vec::new();
        let mut nexts = Vec::new();
        let mut vms = Vec::new();

        for n in 0..Self::BULLET_MAX {
            let state = BulletState::new(0.0, 0.0, BulletType::Bullet1, BulletColor::White);
            states.push(Rc::new(RefCell::new(state)));
            nexts.push(if n == Self::BULLET_MAX - 1 {
                None
            } else {
                Some(n + 1)
            });
            let mut vm = VM::new();
            vm.set_code(Rc::new(Self::EMPTY_BULLET_CODE.to_vec()));
            vms.push(vm);
        }

        Self {
            states,
            nexts,
            vms,
            next_disabled: Some(0),
        }
    }

    pub fn update(&mut self) -> GameResult<()> {
        for idx in 0..Self::BULLET_MAX {
            let mut state = self.states[idx].borrow_mut();
            let state = state.deref_mut();
            let vm = &mut self.vms[idx];

            if state.enabled {
                if let Err(err) = vm.run(state) {
                    return Err(GameError::CustomError(format!("error = {:?}", err)));
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for idx in 0..Self::BULLET_MAX {
            let state = self.states[idx].borrow();

            if state.enabled {
                if let Err(err) = state.draw(ctx) {
                    return Err(GameError::CustomError(format!("error = {:?}", err)));
                }
            }
        }

        Ok(())
    }
}
