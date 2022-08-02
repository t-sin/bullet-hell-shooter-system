mod game;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder,
};

use crate::game::BulletsGame;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

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
