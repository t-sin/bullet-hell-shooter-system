use std::{collections::HashMap, rc::Rc};

use lang_compiler::{compile, BulletCode};

const CODE_MAP: [(&str, &str); 2] = [
    (
        "bullet1",
        r##"

        proc die_out_of_screen() {
          if self.x < -10 || 610 < self.x || self.y < -10 || 850 < self.y {
            die()
          } else {
            false
          }
        }

        global vx = -1
        global vy = -1

        proc main() {
          die_out_of_screen()

          vx = if vx == -1 { (player.x - self.x) / 10 } else { vx }
          vy = if vy == -1 { (player.y - self.y) / 10 } else { vy }
          self.x = self.x + vx
          self.y = self.y + vy
        }
        "##,
    ),
    (
        "player",
        r##"
        global slow_v = 4.0
        global fast_v = 7.0

        proc velocity() -> float {
          return if player.input_slow { slow_v } else { fast_v }
        }

        proc main() {
          self.x = self.x - if player.input_left { velocity() } else { 0.0 }
          self.x = self.x + if player.input_right { velocity() } else { 0.0 }
          self.y = self.y - if player.input_up { velocity() } else { 0.0 }
          self.y = self.y + if player.input_down { velocity() } else { 0.0 }

          fire("bullet1", 200, 100)
        }
        "##,
        //              if self.input_shot { fire(self.x, self.y) } else { false }
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
            let result = match compile(codestr.to_string(), &vec) {
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

            eprintln!("[Bullet: {}] VM code = {:?}", bc.name, bc.code);

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
