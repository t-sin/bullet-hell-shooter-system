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
mod bullet_codes;
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
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Up => self.shooter.input(&Input::Up, true),
            KeyCode::Down => self.shooter.input(&Input::Down, true),
            KeyCode::Left => self.shooter.input(&Input::Left, true),
            KeyCode::Right => self.shooter.input(&Input::Right, true),
            KeyCode::Z => self.shooter.input(&Input::Shot, true),
            _ => (),
        }
        match keymods {
            KeyMods::SHIFT => self.shooter.input(&Input::Slow, true),
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        match keycode {
            KeyCode::Up => self.shooter.input(&Input::Up, false),
            KeyCode::Down => self.shooter.input(&Input::Down, false),
            KeyCode::Left => self.shooter.input(&Input::Left, false),
            KeyCode::Right => self.shooter.input(&Input::Right, false),
            KeyCode::Z => self.shooter.input(&Input::Shot, false),
            _ => (),
        }
        match keymods {
            KeyMods::SHIFT => self.shooter.input(&Input::Slow, false),
            _ => (),
        }
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
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
