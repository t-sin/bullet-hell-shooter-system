use ggez::{event::EventHandler, Context, GameResult};

use crate::game::Scene;

pub struct ShooterScene {}

impl ShooterScene {
    pub fn new() -> Self {
        Self {}
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
