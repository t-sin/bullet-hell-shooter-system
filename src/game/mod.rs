mod shooter;

use ggez::{event::EventHandler, input::keyboard::KeyInput, Context, GameResult};

use shooter::ShooterScene;

pub trait Scene: EventHandler {
    fn next(&self) -> Box<dyn Scene>;
}

pub struct BulletsGame {
    scene: Box<dyn Scene>,
}

impl BulletsGame {
    pub fn new(_ctx: &mut Context) -> Self {
        BulletsGame {
            scene: Box::new(ShooterScene::new()),
        }
    }
}

impl EventHandler for BulletsGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.scene.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.scene.draw(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyInput, repeat: bool) -> GameResult<()> {
        self.scene.key_down_event(ctx, key, repeat)
    }

    fn key_up_event(&mut self, ctx: &mut Context, key: KeyInput) -> GameResult<()> {
        self.scene.key_up_event(ctx, key)
    }
}
