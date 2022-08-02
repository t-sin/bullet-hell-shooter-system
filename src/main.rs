use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::{Context, ContextBuilder, GameResult};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

struct BulletsGame {}

impl BulletsGame {
    pub fn new(_ctx: &mut Context) -> Self {
        BulletsGame {}
    }
}

impl EventHandler for BulletsGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}

fn main() {
    let title = format!("some nice game v{}", env!("CARGO_PKG_VERSION"));
    let author = "t-sin";

    let window_setup = WindowSetup::default().title(&title);
    let window_mode = WindowMode::default()
        .dimensions(WIDTH, HEIGHT)
        .resizable(false);

    let (mut ctx, event_loop) = ContextBuilder::new(&title, author)
        .window_setup(window_setup)
        .window_mode(window_mode)
        .build()
        .expect("cannot create ggez context.");
    let game = BulletsGame::new(&mut ctx);

    event::run(ctx, event_loop, game);
}
