use super::Bullet;

enum Data {
    Float(f32),
    Int(i32),
}

pub enum Inst {
    Nop,
}

pub struct VM {
    pc: usize,
    code: Vec<Inst>,
    stack: Vec<Data>,
    memory: Vec<u8>,
}

struct Terminated(bool);

#[derive(Debug)]
pub enum RuntimeError {}

fn run1(bullet: &mut Bullet) -> Result<Terminated, RuntimeError> {
    Ok(Terminated(true))
}

impl VM {
    pub fn new(code: Vec<Inst>, memory: Vec<u8>) -> Self {
        VM {
            pc: 0,
            code,
            stack: Vec::new(),
            memory,
        }
    }

    pub fn run(bullet: &mut Bullet) -> Result<(), RuntimeError> {
        loop {
            match run1(bullet) {
                Ok(terminated) => {
                    if terminated.0 {
                        break;
                    } else {
                        continue;
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(())
    }
}
