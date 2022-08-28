use std::{cell::RefCell, collections::VecDeque, ops::DerefMut, rc::Rc};

use ggez::{graphics, Context, GameError, GameResult};

use lang_compiler::BulletCode;
use lang_component::{
    bullet::{BulletColor, BulletId, BulletType, StateIO},
    vm::{Data, Inst, OperationQuery},
};
use lang_vm::SuspendingReason;

use super::{bullet::Bullet, shooter::OperationProcessor, SceneDrawable};

pub struct BulletPool {
    pub states: Vec<Rc<RefCell<Bullet>>>,
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

        for n in 0..Self::BULLET_MAX {
            let mut state = Bullet::new(
                0.0,
                0.0,
                BulletType::Bullet1,
                BulletColor::White,
                Rc::new(BulletCode::new("")),
            );
            state.vm.set_code(Rc::new(Self::EMPTY_BULLET_CODE.to_vec()));

            states.push(Rc::new(RefCell::new(state)));
            nexts.push(if n == Self::BULLET_MAX - 1 {
                None
            } else {
                Some(n + 1)
            });
        }

        Self {
            states,
            nexts,
            first_disabled: Some(0),
            enabled_count: 0,
        }
    }

    pub fn update(
        &mut self,
        player: Rc<RefCell<Bullet>>,
        op_queue: &mut VecDeque<OperationQuery>,
    ) -> GameResult<()> {
        for idx in 0..Self::BULLET_MAX {
            let mut state = self.states[idx].borrow_mut();
            let state = state.deref_mut();

            if state.enabled {
                let mut reason = state.vm.start(idx, op_queue);

                loop {
                    match reason {
                        Ok(SuspendingReason::Terminated) => break,
                        Ok(SuspendingReason::Running) => unreachable!(),
                        Ok(SuspendingReason::ToReadState(bid, sid)) => {
                            let d = match bid {
                                BulletId::Player => {
                                    let player = player.borrow();
                                    player.read(&bid, &sid)
                                }
                                BulletId::Itself => state.read(&bid, &sid),
                                _ => todo!("bullet {:?} is not implemented yet", bid),
                            };

                            state.vm.push_data(d);
                        }
                        Ok(SuspendingReason::ToWriteState(bid, sid, d)) => match bid {
                            BulletId::Itself => state.write(&bid, &sid, d),
                            _ => todo!("bullet {:?} is not implemented yet", bid),
                        },
                        Err(err) => {
                            return Err(GameError::CustomError(format!("error = {:?}", err)))
                        }
                    }

                    reason = state.vm.resume(idx, op_queue);
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult<()> {
        for idx in 0..Self::BULLET_MAX {
            let state = self.states[idx].borrow();

            if state.visible {
                if let Err(err) = state.draw(ctx, canvas) {
                    return Err(GameError::CustomError(format!("error = {:?}", err)));
                }
            }
        }

        let debug_msg = format!(
            r##"
fps: {}
object num: {}
"##,
            ctx.time.fps(),
            self.enabled_count
        );
        let debug_msg = graphics::Text::new(debug_msg);
        let param = graphics::DrawParam::default().dest(glam::vec2(10.0, 0.0));
        canvas.draw(&debug_msg, param);

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

        let vm = &mut state.vm;
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
