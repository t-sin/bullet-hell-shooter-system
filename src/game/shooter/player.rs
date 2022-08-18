use ggez::{Context, GameError, GameResult};

use super::{bullet::BulletState, SceneDrawable};
use lang_compiler::compile;
use lang_vm::{
    bullet::{BulletColor, BulletType},
    VM,
};

pub struct Player {
    pub state: BulletState,
    pub vm: VM,
}

impl Player {
    pub fn new() -> Self {
        let state = BulletState::new(200.0, 400.0, BulletType::Player, BulletColor::White);

        let player_code = r##"
            global slow_v = 4.0
            global fast_v = 7.0

            proc velocity() -> float {
              return if $input_slow { slow_v } else { fast_v }
            }

            proc main() {
              $px = $px - if $input_left { velocity() } else { 0.0 }
              $px = $px + if $input_right { velocity() } else { 0.0 }
              $py = $py - if $input_up { velocity() } else { 0.0 }
              $py = $py + if $input_down { velocity() } else { 0.0 }
            }
            "##
        .to_string();
        let compiled_player = compile(player_code, BulletState::state_id_map());
        let compiled_player = compiled_player.unwrap();
        let mut vm = VM::new();
        eprintln!("VM code = {:?}", compiled_player);
        vm.set_code(compiled_player.code);
        vm.set_memory(compiled_player.memory);

        Self { state, vm }
    }

    pub fn update(&mut self) -> GameResult<()> {
        if let Err(err) = self.vm.run(&mut self.state) {
            return Err(GameError::CustomError(format!("error = {:?}", err)));
        }

        Ok(())
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        if let Err(err) = self.state.draw(ctx) {
            return Err(GameError::CustomError(format!("error = {:?}", err)));
        }

        Ok(())
    }
}
