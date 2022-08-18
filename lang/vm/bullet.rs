use crate::Data;
use lang_component::syntax::Type;

#[derive(Debug, Clone, Copy)]
pub enum BulletType {
    Player,
    Bullet1,
}

#[derive(Debug, Clone, Copy)]
pub enum BulletColor {
    White,
}

pub trait State {
    fn get(&self, id: usize) -> Option<Data>;
    fn set(&mut self, id: usize, d: Data) -> Result<(), Option<Type>>;
}

pub enum Operation {
    PutBullet(f32, f32, String, BulletType, BulletColor),
}
