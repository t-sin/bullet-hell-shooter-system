use super::Bullet;

enum Data {
    Float(f32),
    Int(i32),
}

#[derive(Debug, Copy, Clone)]
pub enum Inst {
    Term, // tells now the program reaches the end of program successfully
}

pub struct VM {
    pc: usize,
    code: Box<[Inst]>,
    stack: Vec<Data>,
    memory: Vec<u8>,
}

struct Terminated(bool);

#[derive(Debug)]
pub enum RuntimeError {
    OutOfCode(usize, Vec<Inst>),
}

fn run1(bullet: &mut Bullet) -> Result<Terminated, RuntimeError> {
    match bullet.vm.code.get(bullet.vm.pc) {
        Some(Inst::Term) => Ok(Terminated(true)),
        None => Err(RuntimeError::OutOfCode(
            bullet.vm.pc,
            Vec::from(bullet.vm.code.clone()),
        )),
    }
}

impl VM {
    pub fn new(memory: Vec<u8>) -> Self {
        VM {
            pc: 0,
            code: Vec::new().into(),
            stack: Vec::new(),
            memory,
        }
    }

    pub fn set_code(&mut self, code: Vec<Inst>) {
        self.code = code.into();
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
