use std::{collections::HashMap, rc::Rc};

use lang_compiler::compile;
use lang_component::{syntax::Signature, vm::Inst};

use super::bullet::BulletState;

const CODE_MAP: [(&str, &str); 1] = [(
    "player",
    r##"
    global slow_v = 4.0
    global fast_v = 7.0

    proc velocity() -> float {
      return if $input_slow { slow_v } else { fast_v }
    }

    proc main() {
      $x = $x - if $input_left { velocity() } else { 0.0 }
      $x = $x + if $input_right { velocity() } else { 0.0 }
      $y = $y - if $input_up { velocity() } else { 0.0 }
      $y = $y + if $input_down { velocity() } else { 0.0 }
    }
    "##,
)];

pub struct BulletCode {
    pub code: Rc<Vec<Inst>>,
    pub initial_memory: Vec<u8>,
    pub signature: Signature,
}

pub type BulletCodeMap = HashMap<String, BulletCode>;

pub fn compile_codes() -> BulletCodeMap {
    let mut map = HashMap::new();

    for (name, codestr) in CODE_MAP.iter() {
        let result = match compile(codestr.to_string(), BulletState::state_id_map()) {
            Ok(result) => result,
            Err(err) => panic!("compilation '{}' fails: {:?}", name, err),
        };

        let bc = BulletCode {
            code: Rc::new(result.code),
            initial_memory: result.memory,
            signature: result.signature,
        };

        map.insert(name.to_string(), bc);
    }

    map
}
