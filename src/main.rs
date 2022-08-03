mod constant;
mod game;
mod lang;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder,
};

use crate::game::BulletsGame;

fn main() {
    let title = format!("some nice game v{}", env!("CARGO_PKG_VERSION"));
    let author = "t-sin";

    let window_setup = WindowSetup::default().title(&title);
    let window_mode = WindowMode::default()
        .dimensions(constant::WIDTH, constant::HEIGHT)
        .resizable(false);

    let (mut ctx, event_loop) = ContextBuilder::new(&title, author)
        .window_setup(window_setup)
        .window_mode(window_mode)
        .build()
        .expect("cannot create ggez context.");
    let game = BulletsGame::new(&mut ctx);

    event::run(ctx, event_loop, game);
}
