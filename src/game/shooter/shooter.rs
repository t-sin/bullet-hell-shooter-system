use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use ggez::{graphics, Context, GameResult};

use lang_compiler::BulletCode;
use lang_component::{
    bullet::{BulletColor, BulletId, BulletType},
    vm::{Data, OperationQuery},
};

use super::{
    bullet::BulletState, bullet_codes::BulletCodes, bullet_pool::BulletPool, SceneDrawable,
};

pub struct Objects {
    pub player: Rc<RefCell<BulletState>>,
    pub bullets: BulletPool,
}

impl Objects {
    fn new(bullet_codes: &BulletCodes) -> Self {
        let bc = bullet_codes.by_name.get("player").unwrap();
        let player = BulletState::new(
            200.0,
            400.0,
            BulletType::Player,
            BulletColor::White,
            bc.clone(),
        );
        let player = Rc::new(RefCell::new(player));

        Self {
            player,
            bullets: BulletPool::new(),
        }
    }
}

pub struct Shooter {
    objects: Objects,
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
            objects: Objects::new(&bullet_codes),
            bullet_codes,
            op_queue: VecDeque::new(),
        }
    }

    pub fn input(&mut self, input: &Input, b: bool) {
        let mut player = self.objects.player.borrow_mut();
        match input {
            Input::Up => player.input.up = b,
            Input::Down => player.input.down = b,
            Input::Left => player.input.left = b,
            Input::Right => player.input.right = b,
            Input::Shot => player.input.shot = b,
            Input::Slow => player.input.slow = b,
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
        self.objects
            .bullets
            .fire(x, y, r#type, color, params, bullet_code)
    }

    fn kill(&mut self, id: usize) {
        self.objects.bullets.kill(id);
    }
}

impl Shooter {
    pub fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        {
            let mut player = self.objects.player.borrow_mut();
            player.update(BulletId::Player, &self.objects, &mut self.op_queue)?;
        }
        {
            let player = self.objects.player.clone();
            self.objects.bullets.update(player, &mut self.op_queue)?;
        }

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
        {
            let player = self.objects.player.borrow();
            player.draw(ctx, canvas)?;
        }
        self.objects.bullets.draw(ctx, canvas)?;

        Ok(())
    }
}
