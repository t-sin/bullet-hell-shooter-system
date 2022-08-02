use ggez::{
    graphics::{
        self,
        // BlendMode, Canvas,
        Color,
        DrawMode,
        DrawParam,
        MeshBuilder,
    },
    mint::Point2,
    Context, GameResult,
};
use glam;

use super::SceneDrawable;
use crate::bullet::{Bullet, BulletColor, BulletType};

impl SceneDrawable for Bullet {
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let color = match self.appearance.color {
            BulletColor::White => Color::from_rgb(255, 255, 255),
        };
        let pos = self.pos;
        let param = DrawParam::default().color(color);

        match self.appearance.r#type {
            BulletType::Player => {
                let points: [glam::Vec2; 3] = [
                    glam::vec2(10.0, 7.0),
                    glam::vec2(0.0, -12.0),
                    glam::vec2(-10.0, 7.0),
                ];

                let dest = glam::vec2(0.0, 0.0) + pos;
                let param = param.dest::<Point2<f32>>(dest.into());
                let mesh = MeshBuilder::new()
                    .polygon(DrawMode::stroke(1.5), &points, color)?
                    .build(ctx)?;
                graphics::draw(ctx, &mesh, param)?;

                let hit_area = MeshBuilder::new()
                    .circle(DrawMode::stroke(1.0), glam::vec2(0.0, 0.0), 5.0, 1.0, color)?
                    //.circle(DrawMode::stroke(1.0), glam::vec2(0.0, 0.0), 3.0, 1.0, color)?
                    .build(ctx)?;
                graphics::draw(ctx, &hit_area, param)?;
            }
            BulletType::Bullet1 => {
                let dest = glam::vec2(-5.0, -5.0) + pos;
                let param = param.dest::<Point2<f32>>(dest.into());
                let bullet = MeshBuilder::new()
                    .circle(DrawMode::stroke(1.0), pos, 8.0, 1.0, color)?
                    .build(ctx)?;
                graphics::draw(ctx, &bullet, param)?;
            }
        };

        Ok(())
    }
}
