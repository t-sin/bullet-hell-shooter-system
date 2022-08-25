use std::{cell::RefCell, collections::VecDeque, ops::DerefMut, rc::Rc};

use ggez::{graphics, timer, Context, GameError, GameResult};

use lang_compiler::BulletCode;
use lang_component::{
    bullet::{BulletColor, BulletType},
    vm::{Data, Inst, OperationQuery},
};
use lang_vm::VM;

use super::{bullet::BulletState, shooter::OperationProcessor, SceneDrawable};

pub struct BulletPool {
    pub states: Vec<Rc<RefCell<BulletState>>>,
    pub vms: Vec<VM>,
    pub nexts: Vec<Option<usize>>,
    pub first_disabled: Option<usize>,
    enabled_count: usize,
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
            first_disabled: Some(0),
            enabled_count: 0,
        }
    }

    pub fn update(&mut self, op_queue: &mut VecDeque<OperationQuery>) -> GameResult<()> {
        for idx in 0..Self::BULLET_MAX {
            let mut state = self.states[idx].borrow_mut();
            let state = state.deref_mut();
            let vm = &mut self.vms[idx];

            if state.enabled {
                if let Err(err) = vm.run(idx, state, op_queue) {
                    return Err(GameError::CustomError(format!("error = {:?}", err)));
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for idx in 0..Self::BULLET_MAX {
            let state = self.states[idx].borrow();

            if state.visible {
                if let Err(err) = state.draw(ctx) {
                    return Err(GameError::CustomError(format!("error = {:?}", err)));
                }
            }
        }

        let debug_msg = format!(
            r##"
fps: {}
object num: {}
"##,
            timer::fps(ctx),
            self.enabled_count
        );
        let debug_msg = graphics::Text::new(debug_msg);
        let param = graphics::DrawParam::default().dest(glam::vec2(10.0, 0.0));
        graphics::draw(ctx, &debug_msg, param)?;

        Ok(())
    }
}

impl OperationProcessor for BulletPool {
    fn fire(
        &mut self,
        x: f32,
        y: f32,
        r#type: BulletType,
        color: BulletColor,
        _params: Vec<Data>,
        bullet_code: Rc<BulletCode>,
    ) -> bool {
        if self.first_disabled.is_none() {
            return false;
        }

        let idx = self.first_disabled.unwrap();

        self.first_disabled = self.nexts[idx];
        self.nexts[idx] = None;
        self.enabled_count += 1;

        let mut state = self.states[idx].borrow_mut();
        state.enabled = true;
        state.visible = true;
        state.pos.x = x;
        state.pos.y = y;
        state.appearance.r#type = r#type;
        state.appearance.color = color;

        let vm = &mut self.vms[idx];
        vm.code = bullet_code.code.clone();
        vm.memory
            .as_mut_slice()
            .copy_from_slice(bullet_code.initial_memory.as_slice());
        vm.stack.clear();

        true
    }

    fn kill(&mut self, id: usize) {
        self.enabled_count -= 1;

        let mut state = self.states[id].borrow_mut();
        state.enabled = false;
        state.visible = false;

        let next_disabled = self.first_disabled;
        self.nexts[id] = next_disabled;
        self.first_disabled = Some(id);
    }
}
