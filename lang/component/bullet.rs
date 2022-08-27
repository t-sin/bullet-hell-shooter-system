use crate::{syntax::Type, vm::Data};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BulletType {
    Player,
    Bullet1,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BulletColor {
    White,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BulletId {
    Player,
    Enemy(usize),
    Bullet(usize),
}

pub trait Reference {
    fn refer(&self, bid: &BulletId, sid: &StateId) -> Data;
}

pub trait State {
    fn get(&self, id: usize) -> Option<Data>;
    fn set(&mut self, id: usize, d: Data) -> Result<(), Option<Type>>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateId {
    PosX,
    PosY,
}
