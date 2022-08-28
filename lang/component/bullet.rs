use crate::{token::Keyword, vm::Data};

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
    Itself,
    Player,
    Enemy(usize),
    Bullet(usize),
}

impl TryFrom<Keyword> for BulletId {
    type Error = ();

    fn try_from(kw: Keyword) -> Result<Self, Self::Error> {
        match kw {
            Keyword::SelfKw => Ok(BulletId::Itself),
            Keyword::Player => Ok(BulletId::Player),
            _ => Err(()),
        }
    }
}

pub trait StateIO {
    fn read(&self, bid: &BulletId, sid: &StateId) -> Data;
    fn write(&mut self, bid: &BulletId, sid: &StateId, d: Data);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateId {
    PosX,
    PosY,
    InputUp,
    InputDown,
    InputLeft,
    InputRight,
    InputShot,
    InputSlow,
    Enabled,
}

impl TryFrom<&str> for StateId {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "x" => Ok(StateId::PosX),
            "y" => Ok(StateId::PosY),
            "input_up" => Ok(StateId::InputUp),
            "input_down" => Ok(StateId::InputDown),
            "input_left" => Ok(StateId::InputLeft),
            "input_right" => Ok(StateId::InputRight),
            "input_shot" => Ok(StateId::InputShot),
            "input_slow" => Ok(StateId::InputSlow),
            "input_enabled" => Ok(StateId::Enabled),
            _ => Err(()),
        }
    }
}
