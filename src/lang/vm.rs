use lang_component::vm::*;

use super::Bullet;

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
    UnknownStateName(String),
}

fn to_fbool(b: bool) -> Data {
    if b {
        Data::Float(1.0)
    } else {
        Data::Float(0.0)
    }
}

fn run1(bullet: &mut Bullet) -> Result<Terminated, RuntimeError> {
    let pc = bullet.vm.pc;
    let inst = bullet.vm.code.get(pc);
    bullet.vm.pc += 1;

    println!("=============================");
    println!("pos = {:?}", bullet.pos);
    println!("stack = {:?}", bullet.vm.stack);
    println!("pc = {}, inst = {:?}", pc, inst);

    match inst {
        Some(inst) => {
            match inst {
                Inst::Term => Ok(Terminated(true)),
                Inst::Float(f) => {
                    bullet.vm.stack.push(Data::Float(*f));
                    Ok(Terminated(false))
                }
                Inst::Get(name) => {
                    match name.as_str() {
                        "Pos:X" => bullet.vm.stack.push(Data::Float(bullet.pos.x)),
                        "Pos:Y" => bullet.vm.stack.push(Data::Float(bullet.pos.y)),
                        "Input:Up" => bullet.vm.stack.push(to_fbool(bullet.input.up)),
                        "Input:Down" => bullet.vm.stack.push(to_fbool(bullet.input.down)),
                        "Input:Left" => bullet.vm.stack.push(to_fbool(bullet.input.left)),
                        "Input:Right" => bullet.vm.stack.push(to_fbool(bullet.input.right)),
                        "Input:Shot" => bullet.vm.stack.push(to_fbool(bullet.input.shot)),
                        "Input:Slow" => bullet.vm.stack.push(to_fbool(bullet.input.slow)),
                        _ => return Err(RuntimeError::UnknownStateName(name.to_owned())),
                    };
                    Ok(Terminated(false))
                }
                Inst::Set(name) => {
                    if let Some(d) = bullet.vm.stack.pop() {
                        match name.as_str() {
                            "Pos:X" => {
                                if let Data::Float(f) = d {
                                    //println!("SetPosX: pos.x <- {}", x);
                                    bullet.pos.x = f;
                                    Ok(Terminated(false))
                                } else {
                                    Err(RuntimeError::TypeMismatched(d, "float".to_owned()))
                                }
                            }
                            "Pos:Y" => {
                                if let Data::Float(f) = d {
                                    println!("SetPosX: pos.x <- {}", f);
                                    bullet.pos.y = f;
                                    Ok(Terminated(false))
                                } else {
                                    Err(RuntimeError::TypeMismatched(d, "float".to_owned()))
                                }
                            }
                            _ => return Err(RuntimeError::UnknownStateName(name.to_owned())),
                        }
                    } else {
                        Err(RuntimeError::StackUnderflow)
                    }
                }

                Inst::Add | Inst::Sub | Inst::Mul => {
                    if let Some(b) = bullet.vm.stack.pop() {
                        if let Some(a) = bullet.vm.stack.pop() {
                            let a = match a {
                                Data::Float(f) => f,
                            };
                            let b = match b {
                                Data::Float(f) => f,
                            };
                            bullet.vm.stack.push(Data::Float(match inst {
                                Inst::Add => a + b,
                                Inst::Sub => a - b,
                                Inst::Mul => a * b,
                                _ => unreachable!(),
                            }));

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
                    bullet.vm.pc += offset - 1;
                    Ok(Terminated(false))
                }
                Inst::JumpIfZero(offset) => {
                    if let Some(y) = bullet.vm.stack.pop() {
                        if let Data::Float(y) = y {
                            if y == 0.0 {
                                bullet.vm.pc += offset - 1;
                            }
                            Ok(Terminated(false))
                        } else {
                            Err(RuntimeError::TypeMismatched(y, "float".to_owned()))
                        }
                    } else {
                        Err(RuntimeError::StackUnderflow)
                    }
                }
            }
        }
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
