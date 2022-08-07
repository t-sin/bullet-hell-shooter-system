pub enum BulletType {
    Player,
    Bullet1,
}

pub enum BulletColor {
    White,
}

pub trait ReadState {
    fn pos_x(&self) -> f32;
    fn pos_y(&self) -> f32;
    fn input_up(&self) -> bool;
    fn input_down(&self) -> bool;
    fn input_left(&self) -> bool;
    fn input_right(&self) -> bool;
    fn input_shot(&self) -> bool;
    fn input_slow(&self) -> bool;
}

pub trait WriteState: ReadState {
    fn set_pos_x(&mut self, f: f32);
    fn set_pos_y(&mut self, f: f32);
    fn set_input_up(&mut self, b: bool);
    fn set_input_down(&mut self, b: bool);
    fn set_input_left(&mut self, b: bool);
    fn set_input_right(&mut self, b: bool);
    fn set_input_shot(&mut self, b: bool);
    fn set_input_slow(&mut self, b: bool);
}
