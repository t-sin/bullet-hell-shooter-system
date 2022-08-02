use ggez::{event::EventHandler, Context, GameResult};

use crate::{bullet::Bullet, game::Scene};

pub struct ShooterScene {
    bullets: Vec<Bullet>,
}

impl ShooterScene {
    pub fn new() -> Self {
        Self {
            bullets: Vec::new(),
        }
    }
}

impl Scene for ShooterScene {
    fn next(&self) -> Box<dyn Scene> {
        Box::new(ShooterScene::new())
    }
}

impl EventHandler for ShooterScene {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}
