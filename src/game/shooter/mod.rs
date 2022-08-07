use ggez::{
    event::EventHandler,
    graphics::{
        self,
        // BlendMode, Canvas,
        Color,
        DrawMode,
        DrawParam,
        MeshBuilder,
        Rect,
    },
    input::keyboard::{self, KeyCode, KeyMods},
    Context, GameResult,
};
use glam;

use lang_component::vm::Inst;

use crate::{
    constant,
    game::Scene,
    lang::{compile, Appearance, Bullet, BulletColor, BulletType, Input},
};

mod bullet;

trait SceneDrawable {
    fn draw(&self, ctx: &mut Context) -> GameResult<()>;
}

pub struct ShooterScene {
    player: Bullet,
    bullets: Vec<Bullet>,
}

impl ShooterScene {
    pub fn new() -> Self {
        let DO_NOTHING_CODE = [Inst::Term];

        let mut bullets = Vec::new();
        for _ in 0..2000 {
            let mut bullet = Bullet::new(
                0.0,
                0.0,
                Appearance::new(BulletType::Bullet1, BulletColor::White),
            );
            bullet.set_code(Vec::from(DO_NOTHING_CODE.clone()));
            bullets.push(bullet);
        }

        let player_code = r##"
            proc main() {
              $px = $px - if $input_left { if $input_slow { 4.0 } else { 7.0 } } else { 0.0 }
              $px = $px + if $input_right { if $input_slow { 4.0 } else { 7.0 } } else { 0.0 }
              $py = $py - if $input_up { if $input_slow { 4.0 } else { 7.0 } } else { 0.0 }
              $py = $py + if $input_down { if $input_slow { 4.0 } else { 7.0 } } else { 0.0 }
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

        Self {
            player,
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
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.player.state.input = Input::default();
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.player.state.input.right = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.player.state.input.left = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.player.state.input.up = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.player.state.input.down = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Z) {
            self.player.state.input.shot = true;
        }
        if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
            self.player.state.input.slow = true;
        }

        self.player.update();

        for bullet in self.bullets.iter_mut() {
            if bullet.state.enabled {
                bullet.update();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // let mut canvas = Canvas::new(
        //     ctx,
        //     constant::WIDTH as u16,
        //     constant::HEIGHT as u16,
        //     conf::NumSamples::One,
        //     graphics::get_window_color_format(ctx),
        // )?;
        // graphics::set_canvas(ctx, Some(&canvas));

        static BG_RECT: Rect = Rect::new(0.0, 0.0, constant::WIDTH, constant::HEIGHT);
        static BG_COLOR: Color = Color::new(0.0, 0.1, 0.1, 1.0);
        let bg = MeshBuilder::new()
            .rectangle(DrawMode::fill(), BG_RECT, BG_COLOR)?
            .build(ctx)?;
        graphics::draw(ctx, &bg, DrawParam::default().dest(glam::vec2(0.0, 0.0)))?;

        static SHOOTER_AREA_RECT: Rect = Rect::new(
            constant::SHOOTER_OFFSET_X,
            constant::SHOOTER_OFFSET_Y,
            constant::SHOOTER_WIDTH,
            constant::SHOOTER_HEIGHT,
        );
        static SHOOTER_AREA_COLOR: Color = Color::new(0.0, 0.07, 0.1, 0.98);
        static SHOOTER_BORDER_COLOR: Color = Color::new(0.0, 0.6, 0.8, 1.0);
        let shooter_area = MeshBuilder::new()
            .rectangle(DrawMode::fill(), SHOOTER_AREA_RECT, SHOOTER_AREA_COLOR)?
            .build(ctx)?;
        let shooter_border = MeshBuilder::new()
            .rectangle(
                DrawMode::stroke(1.0),
                SHOOTER_AREA_RECT,
                SHOOTER_BORDER_COLOR,
            )?
            .build(ctx)?;
        graphics::draw(ctx, &shooter_area, DrawParam::default())?;
        graphics::draw(ctx, &shooter_border, DrawParam::default())?;

        self.player.draw(ctx)?;

        for bullet in self.bullets.iter() {
            if bullet.state.enabled {
                bullet.draw(ctx)?;
            }
        }

        graphics::present(ctx)?;
        // graphics::set_canvas(ctx, None);
        // canvas.set_blend_mode(Some(BlendMode::Premultiplied));

        Ok(())
    }
}
