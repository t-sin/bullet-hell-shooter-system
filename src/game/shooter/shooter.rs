use ggez::{event::EventHandler, Context, GameResult};

use super::{bullet::InputState, bullet_pool::BulletPool, player::Player};

pub struct Shooter {
    pub player: Player,
    pub bullets: BulletPool,
}

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
        Self {
            player: Player::new(),
            bullets: BulletPool::new(),
        }
    }

    pub fn clear_input(&mut self) {
        self.player.state.input = InputState::default();
    }

    pub fn input(&mut self, input: &Input) {
        match input {
            Input::Up => self.player.state.input.up = true,
            Input::Down => self.player.state.input.down = true,
            Input::Left => self.player.state.input.left = true,
            Input::Right => self.player.state.input.right = true,
            Input::Shot => self.player.state.input.shot = true,
            Input::Slow => self.player.state.input.slow = true,
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
