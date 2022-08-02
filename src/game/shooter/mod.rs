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

use crate::{
    bullet::{vm::Inst, Appearance, Bullet, BulletColor, BulletType, Input},
    constant,
    game::Scene,
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

        let PLAYER_CODE = [
            Inst::Get("PosX".to_string()),
            Inst::Float(1.0),
            Inst::Add,
            Inst::Dup,
            Inst::Float(constant::SHOOTER_PLAYER_AREA_X2),
            Inst::Lt,
            Inst::JumpIfZero(1),
            Inst::Set("PosX".to_string()),
            Inst::Jump(2),
            Inst::Float(constant::SHOOTER_PLAYER_AREA_X2),
            Inst::Set("PosX".to_string()),
            Inst::Term,
        ];
        let mut player = Bullet::new(
            200.0,
            400.0,
            Appearance::new(BulletType::Player, BulletColor::White),
        );
        player.set_code(PLAYER_CODE.clone().into());

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
        self.player.input = Input::default();
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.player.input.right = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.player.input.left = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.player.input.up = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.player.input.down = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Z) {
            self.player.input.shot = true;
        }
        if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
            self.player.input.slow = true;
        }

        // dummy input processing for player
        // this will replaced by VM code
        // {
        //     let (dx, dy) = if self.player.input.slow {
        //         (4.0, 4.0)
        //     } else {
        //         (9.0, 9.0)
        //     };
        //     if self.player.input.right {
        //         self.player.pos.x += dx
        //     }
        //     if self.player.input.left {
        //         self.player.pos.x += -dx
        //     }
        //     if self.player.input.up {
        //         self.player.pos.y -= dy
        //     }
        //     if self.player.input.down {
        //         self.player.pos.y += dy
        //     }
        // }

        self.player.update();

        for bullet in self.bullets.iter_mut() {
            if bullet.enabled {
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
            if bullet.enabled {
                bullet.draw(ctx)?;
            }
        }

        graphics::present(ctx)?;
        // graphics::set_canvas(ctx, None);
        // canvas.set_blend_mode(Some(BlendMode::Premultiplied));

        Ok(())
    }
}
