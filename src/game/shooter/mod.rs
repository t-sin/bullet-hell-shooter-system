use ggez::{
    event::EventHandler,
    graphics::{
        self,
        // BlendMode, Canvas,
        Color,
        DrawMode,
        DrawParam,
        Rect,
    },
    input::keyboard::{KeyCode, KeyInput, KeyMods},
    Context, GameResult,
};
use glam;

mod bullet;
mod bullet_codes;
mod bullet_pool;
mod shooter;

use crate::{constant, game::Scene};

use shooter::{Input, Shooter};

trait SceneDrawable {
    fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult<()>;
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
        input: KeyInput,
        _repeat: bool,
    ) -> GameResult<()> {
        match input.keycode {
            Some(KeyCode::Up) => self.shooter.input(&Input::Up, true),
            Some(KeyCode::Down) => self.shooter.input(&Input::Down, true),
            Some(KeyCode::Left) => self.shooter.input(&Input::Left, true),
            Some(KeyCode::Right) => self.shooter.input(&Input::Right, true),
            Some(KeyCode::Z) => self.shooter.input(&Input::Shot, true),
            _ => (),
        }
        match input.mods {
            KeyMods::SHIFT => self.shooter.input(&Input::Slow, true),
            _ => (),
        }

        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult<()> {
        match input.keycode {
            Some(KeyCode::Up) => self.shooter.input(&Input::Up, false),
            Some(KeyCode::Down) => self.shooter.input(&Input::Down, false),
            Some(KeyCode::Left) => self.shooter.input(&Input::Left, false),
            Some(KeyCode::Right) => self.shooter.input(&Input::Right, false),
            Some(KeyCode::Z) => self.shooter.input(&Input::Shot, false),
            _ => (),
        }
        match input.mods {
            KeyMods::SHIFT => self.shooter.input(&Input::Slow, false),
            _ => (),
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.shooter.update(ctx)?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::CanvasLoadOp::Clear([0.0, 0.1, 0.1, 1.0].into()),
        );

        static BG_RECT: Rect = Rect::new(0.0, 0.0, constant::WIDTH, constant::HEIGHT);
        static BG_COLOR: Color = Color::new(0.0, 0.1, 0.1, 1.0);
        let mut mb = graphics::MeshBuilder::new();
        let bg = mb.rectangle(DrawMode::fill(), BG_RECT, BG_COLOR)?;
        let bg = graphics::Mesh::from_data(ctx, bg.build());
        canvas.draw(&bg, DrawParam::default().dest(glam::vec2(0.0, 0.0)));

        static SHOOTER_AREA_RECT: Rect = Rect::new(
            constant::SHOOTER_OFFSET_X,
            constant::SHOOTER_OFFSET_Y,
            constant::SHOOTER_WIDTH,
            constant::SHOOTER_HEIGHT,
        );
        static SHOOTER_AREA_COLOR: Color = Color::new(0.0, 0.07, 0.1, 0.98);
        static SHOOTER_BORDER_COLOR: Color = Color::new(0.0, 0.6, 0.8, 1.0);
        let mut mb = graphics::MeshBuilder::new();
        let shooter_area = mb.rectangle(DrawMode::fill(), SHOOTER_AREA_RECT, SHOOTER_AREA_COLOR)?;
        let shooter_area = graphics::Mesh::from_data(ctx, shooter_area.build());
        let mut mb = graphics::MeshBuilder::new();
        let shooter_border = mb.rectangle(
            DrawMode::stroke(1.0),
            SHOOTER_AREA_RECT,
            SHOOTER_BORDER_COLOR,
        )?;
        let shooter_border = graphics::Mesh::from_data(ctx, shooter_border.build());
        canvas.draw(&shooter_area, DrawParam::default());
        canvas.draw(&shooter_border, DrawParam::default());

        self.shooter.draw(ctx, &mut canvas)?;

        canvas.finish(ctx)?;

        Ok(())
    }
}
