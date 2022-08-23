use std::{collections::HashMap, rc::Rc};

use lang_compiler::{compile, BulletCode};

use super::bullet::BulletState;

const CODE_MAP: [(&str, &str); 2] = [
    (
        "bullet1",
        r##"
        proc main() {
          $y = $y - 5.5
        }
        "##,
    ),
    (
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

          fire("bullet1", $x, $y)
        }
        "##,
        //              if $input_shot { fire($x, $y) } else { false }
    ),
];

pub struct BulletCodes {
    pub by_name: HashMap<String, Rc<BulletCode>>,
    pub by_id: Vec<Rc<BulletCode>>,
}

impl BulletCodes {
    pub fn compile_codes() -> Self {
        let mut map = HashMap::new();
        let mut vec: Vec<Rc<BulletCode>> = Vec::new();
        let mut id = 0;

        for (name, codestr) in CODE_MAP.iter() {
            let result = match compile(codestr.to_string(), BulletState::state_id_map(), &vec) {
                Ok(result) => result,
                Err(err) => panic!("compilation '{}' fails: {:?}", name, err),
            };

            let bc = BulletCode {
                id: id,
                name: name.to_string(),
                code: Rc::new(result.code),
                initial_memory: result.memory,
                signature: result.signature,
            };
            let bc = Rc::new(bc);

            id += 1;
            map.insert(name.to_string(), bc.clone());
            vec.push(bc.clone());
        }

        Self {
            by_name: map,
            by_id: vec,
        }
    }
}
