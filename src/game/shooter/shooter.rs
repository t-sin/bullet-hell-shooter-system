use std::collections::VecDeque;

use ggez::{event::EventHandler, Context, GameResult};

use lang_compiler::compile;
use lang_component::vm::Inst;
use lang_vm::bullet::{BulletColor, BulletType, Operation, WriteState};

use super::{
    bullet::{Appearance, Bullet, InputState},
    SceneDrawable,
};

pub struct Shooter {
    pub player: Bullet,
    pub bullets: Vec<Bullet>,
    pub queued_ops: VecDeque<Operation>,
}

pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Shot,
    Slow,
}

fn init_player() -> Bullet {
    let player_code = r##"
            proc main() {
              let slow_v = 4.0
              let fast_v = 7.0
              $px = $px - if $input_left { if $input_slow { slow_v } else { fast_v } } else { 0.0 }
              $px = $px + if $input_right { if $input_slow { slow_v } else { fast_v } } else { 0.0 }
              $py = $py - if $input_up { if $input_slow { slow_v } else { fast_v } } else { 0.0 }
              $py = $py + if $input_down { if $input_slow { slow_v } else { fast_v } } else { 0.0 }
            }
            "##
    .to_string();
    let compiled_player_code = compile(player_code);
    let compiled_player_code = compiled_player_code.unwrap();
    eprintln!("VM code = {:?}", compiled_player_code);

    let mut player = Bullet::new(
        200.0,
        400.0,
        Appearance::new(BulletType::Player, BulletColor::White),
    );
    player.set_code(compiled_player_code.into());

    player
}

impl Shooter {
    pub fn new() -> Self {
        let bullet_code = [Inst::Term];

        let mut bullets = Vec::new();
        for _ in 0..2000 {
            let mut bullet = Bullet::new(
                0.0,
                0.0,
                Appearance::new(BulletType::Bullet1, BulletColor::White),
            );
            bullet.set_code(Vec::from(bullet_code.clone()));
            bullets.push(bullet);
        }

        Self {
            player: init_player(),
            bullets,
            queued_ops: VecDeque::new(),
        }
    }

    pub fn clear_input(&mut self) {
        self.player.state.input = InputState::default();
    }

    pub fn input(&mut self, input: &Input) {
        match input {
            Input::Up => self.player.state.set_input_up(true),
            Input::Down => self.player.state.set_input_down(true),
            Input::Left => self.player.state.set_input_left(true),
            Input::Right => self.player.state.set_input_right(true),
            Input::Shot => self.player.state.set_input_shot(true),
            Input::Slow => self.player.state.set_input_slow(true),
        }
    }
}

impl EventHandler for Shooter {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.player.update(&mut self.queued_ops);

        for bullet in self.bullets.iter_mut() {
            if bullet.state.enabled {
                bullet.update(&mut self.queued_ops);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.player.draw(ctx)?;

        for bullet in self.bullets.iter() {
            if bullet.state.enabled {
                bullet.draw(ctx)?;
            }
        }

        Ok(())
    }
}
