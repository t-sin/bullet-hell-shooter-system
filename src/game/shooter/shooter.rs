use std::{collections::VecDeque, rc::Rc};

use ggez::{graphics, Context, GameResult};

use lang_compiler::BulletCode;
use lang_component::{
    bullet::{BulletColor, BulletType},
    vm::{Data, OperationQuery},
};

use super::{bullet_codes::BulletCodes, bullet_pool::BulletPool, player::Player};

pub struct Shooter {
    pub player: Player,
    pub bullets: BulletPool,
    bullet_codes: BulletCodes,
    op_queue: VecDeque<OperationQuery>,
}

#[derive(Debug)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Shot,
    Slow,
}

pub trait OperationProcessor {
    fn fire(
        &mut self,
        x: f32,
        y: f32,
        r#type: BulletType,
        color: BulletColor,
        params: Vec<Data>,
        bullet_code: Rc<BulletCode>,
    ) -> bool;

    fn kill(&mut self, id: usize);
}

impl Shooter {
    pub fn new() -> Self {
        let bullet_codes = BulletCodes::compile_codes();

        Self {
            player: Player::new(&bullet_codes),
            bullets: BulletPool::new(),
            bullet_codes,
            op_queue: VecDeque::new(),
        }
    }

    pub fn input(&mut self, input: &Input, b: bool) {
        match input {
            Input::Up => self.player.state.input.up = b,
            Input::Down => self.player.state.input.down = b,
            Input::Left => self.player.state.input.left = b,
            Input::Right => self.player.state.input.right = b,
            Input::Shot => self.player.state.input.shot = b,
            Input::Slow => self.player.state.input.slow = b,
        }
    }
}

impl OperationProcessor for Shooter {
    fn fire(
        &mut self,
        x: f32,
        y: f32,
        r#type: BulletType,
        color: BulletColor,
        params: Vec<Data>,
        bullet_code: Rc<BulletCode>,
    ) -> bool {
        self.bullets.fire(x, y, r#type, color, params, bullet_code)
    }

    fn kill(&mut self, id: usize) {
        self.bullets.kill(id);
    }
}

impl Shooter {
    pub fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.player.update(&mut self.op_queue)?;
        self.bullets.update(&mut self.op_queue)?;

        loop {
            if let Some(op) = self.op_queue.pop_back() {
                match op {
                    OperationQuery::Fire(bullet_id, (x, y), r#type, color, params) => {
                        let bc = self.bullet_codes.by_id[bullet_id].clone();
                        self.fire(x, y, r#type, color, params, bc);
                    }
                    OperationQuery::Die(bullet_id) => {
                        self.kill(bullet_id);
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult<()> {
        self.player.draw(ctx, canvas)?;
        self.bullets.draw(ctx, canvas)?;

        Ok(())
    }
}
