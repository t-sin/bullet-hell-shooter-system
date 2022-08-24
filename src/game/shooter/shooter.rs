use ggez::{event::EventHandler, Context, GameResult};

use super::{
    bullet::InputState,
    bullet_codes::{compile_codes, BulletCodeMap},
    bullet_pool::BulletPool,
    player::Player,
};

pub struct Shooter {
    pub player: Player,
    pub bullets: BulletPool,
    code_map: BulletCodeMap,
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

impl Shooter {
    pub fn new() -> Self {
        let code_map = compile_codes();

        Self {
            player: Player::new(&code_map),
            bullets: BulletPool::new(),
            code_map,
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

impl EventHandler for Shooter {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.player.update()?;
        self.bullets.update()?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.player.draw(ctx)?;
        self.bullets.draw(ctx)?;

        Ok(())
    }
}
