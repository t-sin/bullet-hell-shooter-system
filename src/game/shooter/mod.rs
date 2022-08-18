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

mod bullet;
mod bullet_pool;
mod player;
mod shooter;

use crate::{constant, game::Scene};

use shooter::{Input, Shooter};

trait SceneDrawable {
    fn draw(&self, ctx: &mut Context) -> GameResult<()>;
}

pub struct ShooterScene {
    shooter: Shooter,
}

impl ShooterScene {
    pub fn new() -> Self {
        Self {
            shooter: Shooter::new(),
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
        self.shooter.clear_input();
        if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.shooter.input(&Input::Up);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.shooter.input(&Input::Down);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.shooter.input(&Input::Left);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.shooter.input(&Input::Right);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Z) {
            self.shooter.input(&Input::Shot);
        }
        if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
            self.shooter.input(&Input::Slow);
        }

        self.shooter.update(ctx)?;

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

        self.shooter.draw(ctx)?;

        graphics::present(ctx)?;
        // graphics::set_canvas(ctx, None);
        // canvas.set_blend_mode(Some(BlendMode::Premultiplied));

        Ok(())
    }
}
