use super::Bullet;

#[derive(Debug, Copy, Clone)]
enum Data {
    Float(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum Inst {
    // system words
    Term, // tells now the program reaches the end of program successfully
    // embedded data
    Float(f32),
    // position accessors
    GetPosX,
    GetPosY,
    SetPosX,
    SetPosY,
    // arithmetics
    Add,
    // comparators
    EqInt,
    EqFloat,
    Lt,
    // logical
    Not,
    // stack operations
    Dup,
    // control flows
    JumpIfZero(usize),
    Jump(usize),
}

pub struct VM {
    pc: usize,
    code: Box<[Inst]>,
    stack: Vec<Data>,
    memory: Vec<u8>,
}

struct Terminated(bool);

type ExpectedTypeName = String;

#[derive(Debug)]
pub enum RuntimeError {
    OutOfCode(usize, Vec<Inst>),
    StackUnderflow,
    TypeMismatched(Data, ExpectedTypeName),
}

fn run1(bullet: &mut Bullet) -> Result<Terminated, RuntimeError> {
    let pc = bullet.vm.pc;
    let inst = bullet.vm.code.get(pc);
    bullet.vm.pc += 1;

    //println!("pc = {}, inst = {:?}", pc, inst);

    match inst {
        Some(inst) => match inst {
            Inst::Term => Ok(Terminated(true)),
            Inst::Float(f) => {
                bullet.vm.stack.push(Data::Float(*f));
                Ok(Terminated(false))
            }
            Inst::GetPosX => {
                bullet.vm.stack.push(Data::Float(bullet.pos.x));
                //println!("GetPosX: pos.x -> {}", bullet.pos.x);
                Ok(Terminated(false))
            }
            Inst::GetPosY => {
                bullet.vm.stack.push(Data::Float(bullet.pos.y));
                Ok(Terminated(false))
            }
            Inst::SetPosX => {
                if let Some(x) = bullet.vm.stack.pop() {
                    if let Data::Float(x) = x {
                        //println!("SetPosX: pos.x <- {}", x);
                        bullet.pos.x = x;
                        Ok(Terminated(false))
                    } else {
                        Err(RuntimeError::TypeMismatched(x, "float".to_owned()))
                    }
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
            Inst::SetPosY => {
                if let Some(y) = bullet.vm.stack.pop() {
                    if let Data::Float(y) = y {
                        bullet.pos.y = y;
                        Ok(Terminated(false))
                    } else {
                        Err(RuntimeError::TypeMismatched(y, "float".to_owned()))
                    }
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
            Inst::Add => {
                if let Some(a) = bullet.vm.stack.pop() {
                    if let Some(b) = bullet.vm.stack.pop() {
                        let a = match a {
                            Data::Float(f) => f,
                        };
                        let b = match b {
                            Data::Float(f) => f,
                        };
                        bullet.vm.stack.push(Data::Float(a + b));

                        Ok(Terminated(false))
                    } else {
                        Err(RuntimeError::StackUnderflow)
                    }
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
            Inst::EqInt => {
                if let Some(a) = bullet.vm.stack.pop() {
                    if let Some(b) = bullet.vm.stack.pop() {
                        let a = match a {
                            Data::Float(f) => f as i32,
                        };
                        let b = match b {
                            Data::Float(f) => f as i32,
                        };

                        if a == b {
                            bullet.vm.stack.push(Data::Float(1.0));
                        } else {
                            bullet.vm.stack.push(Data::Float(0.0));
                        }

                        Ok(Terminated(false))
                    } else {
                        Err(RuntimeError::StackUnderflow)
                    }
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
            Inst::EqFloat => {
                if let Some(a) = bullet.vm.stack.pop() {
                    if let Some(b) = bullet.vm.stack.pop() {
                        let a = match a {
                            Data::Float(f) => f,
                        };
                        let b = match b {
                            Data::Float(f) => f,
                        };

                        if a == b {
                            bullet.vm.stack.push(Data::Float(1.0));
                        } else {
                            bullet.vm.stack.push(Data::Float(0.0));
                        }

                        Ok(Terminated(false))
                    } else {
                        Err(RuntimeError::StackUnderflow)
                    }
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
            Inst::Lt => {
                if let Some(a) = bullet.vm.stack.pop() {
                    if let Some(b) = bullet.vm.stack.pop() {
                        let a = match a {
                            Data::Float(f) => f,
                        };
                        let b = match b {
                            Data::Float(f) => f,
                        };

                        if a > b {
                            bullet.vm.stack.push(Data::Float(1.0));
                        } else {
                            bullet.vm.stack.push(Data::Float(0.0));
                        }

                        Ok(Terminated(false))
                    } else {
                        Err(RuntimeError::StackUnderflow)
                    }
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
            Inst::Not => {
                if let Some(x) = bullet.vm.stack.pop() {
                    if let Data::Float(x) = x {
                        if x == 0.0 {
                            bullet.vm.stack.push(Data::Float(1.0));
                        } else {
                            bullet.vm.stack.push(Data::Float(0.0));
                        }
                        Ok(Terminated(false))
                    } else {
                        Err(RuntimeError::TypeMismatched(x, "float".to_owned()))
                    }
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
            Inst::Dup => {
                if let Some(x) = bullet.vm.stack.pop() {
                    bullet.vm.stack.push(x.clone());
                    bullet.vm.stack.push(x);
                    Ok(Terminated(false))
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
            Inst::Jump(offset) => {
                bullet.vm.pc += offset;
                Ok(Terminated(false))
            }
            Inst::JumpIfZero(offset) => {
                if let Some(y) = bullet.vm.stack.pop() {
                    if let Data::Float(y) = y {
                        if y == 0.0 {
                            bullet.vm.pc += offset;
                        }
                        Ok(Terminated(false))
                    } else {
                        Err(RuntimeError::TypeMismatched(y, "float".to_owned()))
                    }
                } else {
                    Err(RuntimeError::StackUnderflow)
                }
            }
        },
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
        bullet.vm.pc = 0;

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
