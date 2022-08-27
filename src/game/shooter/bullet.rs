use std::collections::HashMap;

use ggez::{
    graphics::{
        self,
        // BlendMode, Canvas,
        Color,
        DrawMode,
        DrawParam,
        Mesh,
        MeshBuilder,
    },
    mint::Point2,
    Context, GameResult,
};
use glam;

use lang_component::{
    bullet::{BulletColor, BulletType, State},
    syntax::Type,
    vm::Data,
};

use super::SceneDrawable;
use crate::constant;

pub struct Appearance {
    pub r#type: BulletType,
    pub color: BulletColor,
}

impl Appearance {
    pub fn new(r#type: BulletType, color: BulletColor) -> Self {
        Self { r#type, color }
    }
}

#[derive(Debug)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub shot: bool,
    pub slow: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            shot: false,
            slow: false,
        }
    }
}

pub struct BulletState {
    getters: &'static [&'static dyn Fn(&BulletState) -> Data],
    setters: &'static [&'static dyn Fn(&mut BulletState, Data) -> Result<(), Type>],
    pub enabled: bool,
    pub visible: bool,
    pub input: InputState,
    pub pos: glam::Vec2,
    pub appearance: Appearance,
}

macro_rules! set_float {
    ($state:expr, $target: expr, $d:expr) => {
        if let Data::Float(f) = $d {
            $target = f;
            Ok(())
        } else {
            Err(Type::Float)
        }
    };
}

macro_rules! set_bool {
    ($state:expr, $target: expr, $d:expr) => {
        if let Data::Bool(b) = $d {
            $target = b;
            Ok(())
        } else {
            Err(Type::Bool)
        }
    };
}

impl BulletState {
    const STATE_NAMES: [&'static str; 9] = [
        "enabled",
        "x",
        "y",
        "input_up",
        "input_down",
        "input_left",
        "input_right",
        "input_shot",
        "input_slow",
    ];
    const STATE_GETTERS: [&'static dyn Fn(&BulletState) -> Data; 9] = [
        &|state: &BulletState| Data::Bool(state.enabled),
        &|state: &BulletState| Data::Float(state.pos.x),
        &|state: &BulletState| Data::Float(state.pos.y),
        &|state: &BulletState| Data::Bool(state.input.up),
        &|state: &BulletState| Data::Bool(state.input.down),
        &|state: &BulletState| Data::Bool(state.input.left),
        &|state: &BulletState| Data::Bool(state.input.right),
        &|state: &BulletState| Data::Bool(state.input.shot),
        &|state: &BulletState| Data::Bool(state.input.slow),
    ];
    const STATE_SETTERS: [&'static dyn Fn(&mut BulletState, Data) -> Result<(), Type>; 9] = [
        &|state: &mut BulletState, d: Data| set_bool!(state, state.enabled, d),
        &|state: &mut BulletState, d: Data| set_float!(state, state.pos.x, d),
        &|state: &mut BulletState, d: Data| set_float!(state, state.pos.y, d),
        &|state: &mut BulletState, d: Data| set_bool!(state, state.input.up, d),
        &|state: &mut BulletState, d: Data| set_bool!(state, state.input.down, d),
        &|state: &mut BulletState, d: Data| set_bool!(state, state.input.left, d),
        &|state: &mut BulletState, d: Data| set_bool!(state, state.input.right, d),
        &|state: &mut BulletState, d: Data| set_bool!(state, state.input.shot, d),
        &|state: &mut BulletState, d: Data| set_bool!(state, state.input.slow, d),
    ];

    pub fn new(x: f32, y: f32, atype: BulletType, acolor: BulletColor) -> Self {
        Self {
            getters: &Self::STATE_GETTERS,
            setters: &Self::STATE_SETTERS,
            enabled: false,
            visible: false,
            input: InputState::default(),
            pos: glam::vec2(x, y),
            appearance: Appearance::new(atype, acolor),
        }
    }

    pub fn state_id_map() -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for (idx, s) in Self::STATE_NAMES.iter().enumerate() {
            map.insert(s.to_string(), idx);
        }
        map
    }
}

impl State for BulletState {
    fn get(&self, id: usize) -> Option<Data> {
        if let Some(getter) = self.getters.get(id) {
            Some(getter(self))
        } else {
            None
        }
    }

    fn set(&mut self, id: usize, data: Data) -> Result<(), Option<Type>> {
        if let Some(setter) = self.setters.get(id) {
            if let Err(r#type) = setter(self, data) {
                Err(Some(r#type))
            } else {
                Ok(())
            }
        } else {
            Err(None)
        }
    }
}

impl SceneDrawable for BulletState {
    fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult<()> {
        let color = match self.appearance.color {
            BulletColor::White => Color::from_rgb(255, 255, 255),
        };
        let pos = self.pos;
        let param = DrawParam::default()
            .color(color)
            .offset([-constant::SHOOTER_OFFSET_X, -constant::SHOOTER_OFFSET_Y]);

        match self.appearance.r#type {
            BulletType::Player => {
                static POINTS: [[f32; 2]; 3] = [[8.0, 7.0], [0.0, -12.0], [-8.0, 7.0]];

                let dest = glam::vec2(0.0, 0.0) + pos;
                let param = param.dest::<Point2<f32>>(dest.into());
                let mut mb = MeshBuilder::new();
                let mesh = mb.polygon(DrawMode::stroke(1.5), &POINTS, color)?.build();
                let mesh = Mesh::from_data(ctx, mesh);
                canvas.draw(&mesh, param);

                let mut mb = MeshBuilder::new();
                let hit_area = mb
                    .circle(DrawMode::stroke(1.0), [0.0, 0.0], 5.0, 1.0, color)?
                    //.circle(DrawMode::stroke(1.0), glam::vec2(0.0, 0.0), 3.0, 1.0, color)?
                    .build();
                let hit_area = Mesh::from_data(ctx, hit_area);
                canvas.draw(&hit_area, param);
            }
            BulletType::Bullet1 => {
                //                let dest = glam::vec2(-5.0, -5.0) + pos;
                //                let param = param.dest::<Point2<f32>>(dest.into());
                let mut mb = MeshBuilder::new();
                let bullet = mb
                    .circle(DrawMode::stroke(1.0), pos, 4.0, 1.0, color)?
                    .build();
                let bullet = Mesh::from_data(ctx, bullet);
                canvas.draw(&bullet, param);
            }
        };

        Ok(())
    }
}
