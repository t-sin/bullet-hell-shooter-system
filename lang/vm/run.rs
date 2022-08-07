use std::collections::VecDeque;

use lang_component::vm::{Data, Inst};

use crate::{
    bullet::{BulletColor, BulletType, Operation, WriteState},
    error::RuntimeError,
    VM,
};

pub struct Terminated(pub bool);

fn to_fbool(b: bool) -> Data {
    if b {
        Data::Float(1.0)
    } else {
        Data::Float(0.0)
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

    pub fn run(
        &mut self,
        state: &mut dyn WriteState,
        ops_queue: &mut VecDeque<Operation>,
    ) -> Result<(), RuntimeError> {
        self.pc = 0;

        loop {
            match self.run1(state, ops_queue) {
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

    fn run1(
        &mut self,
        state: &mut dyn WriteState,
        ops_queue: &mut VecDeque<Operation>,
    ) -> Result<Terminated, RuntimeError> {
        let pc = self.pc;
        let inst = self.code.get(pc);
        self.pc += 1;

        println!("=============================");
        println!("pos = ({}, {})", state.pos_x(), state.pos_y());
        println!("stack = {:?}", self.stack);
        println!("pc = {}, inst = {:?}", pc, inst);

        match inst {
            Some(inst) => {
                match inst {
                    Inst::Term => Ok(Terminated(true)),
                    Inst::Float(f) => {
                        self.stack.push(Data::Float(*f));
                        Ok(Terminated(false))
                    }
                    Inst::Get(name) => {
                        match name.as_str() {
                            "Pos:X" => self.stack.push(Data::Float(state.pos_x())),
                            "Pos:Y" => self.stack.push(Data::Float(state.pos_y())),
                            "Input:Up" => self.stack.push(to_fbool(state.input_up())),
                            "Input:Down" => self.stack.push(to_fbool(state.input_down())),
                            "Input:Left" => self.stack.push(to_fbool(state.input_left())),
                            "Input:Right" => self.stack.push(to_fbool(state.input_right())),
                            "Input:Shot" => self.stack.push(to_fbool(state.input_shot())),
                            "Input:Slow" => self.stack.push(to_fbool(state.input_slow())),
                            _ => return Err(RuntimeError::UnknownStateName(name.to_owned())),
                        };
                        Ok(Terminated(false))
                    }
                    Inst::Set(name) => {
                        if let Some(d) = self.stack.pop() {
                            match name.as_str() {
                                "Pos:X" => {
                                    if let Data::Float(f) = d {
                                        //println!("SetPosX: pos.x <- {}", x);
                                        state.set_pos_x(f);
                                        Ok(Terminated(false))
                                    } else {
                                        Err(RuntimeError::TypeMismatched(d, "float".to_owned()))
                                    }
                                }
                                "Pos:Y" => {
                                    if let Data::Float(f) = d {
                                        println!("SetPosX: pos.x <- {}", f);
                                        state.set_pos_y(f);
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
                    Inst::Fire(bullet_name) => {
                        if let Some(y) = self.stack.pop() {
                            if let Some(x) = self.stack.pop() {
                                let x = match x {
                                    Data::Float(f) => f,
                                };
                                let y = match y {
                                    Data::Float(f) => f,
                                };
                                ops_queue.push_back(Operation::PutBullet(
                                    x,
                                    y,
                                    "".to_string(),
                                    BulletType::Bullet1,
                                    BulletColor::White,
                                ));
                                Ok(Terminated(false))
                            } else {
                                Err(RuntimeError::StackUnderflow)
                            }
                        } else {
                            Err(RuntimeError::StackUnderflow)
                        }
                    }
                    Inst::Add | Inst::Sub | Inst::Mul => {
                        if let Some(b) = self.stack.pop() {
                            if let Some(a) = self.stack.pop() {
                                let a = match a {
                                    Data::Float(f) => f,
                                };
                                let b = match b {
                                    Data::Float(f) => f,
                                };
                                self.stack.push(Data::Float(match inst {
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
                        if let Some(a) = self.stack.pop() {
                            if let Some(b) = self.stack.pop() {
                                let a = match a {
                                    Data::Float(f) => f as i32,
                                };
                                let b = match b {
                                    Data::Float(f) => f as i32,
                                };

                                if a == b {
                                    self.stack.push(Data::Float(1.0));
                                } else {
                                    self.stack.push(Data::Float(0.0));
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
                        if let Some(a) = self.stack.pop() {
                            if let Some(b) = self.stack.pop() {
                                let a = match a {
                                    Data::Float(f) => f,
                                };
                                let b = match b {
                                    Data::Float(f) => f,
                                };

                                if a == b {
                                    self.stack.push(Data::Float(1.0));
                                } else {
                                    self.stack.push(Data::Float(0.0));
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
                        if let Some(a) = self.stack.pop() {
                            if let Some(b) = self.stack.pop() {
                                let a = match a {
                                    Data::Float(f) => f,
                                };
                                let b = match b {
                                    Data::Float(f) => f,
                                };

                                if a > b {
                                    self.stack.push(Data::Float(1.0));
                                } else {
                                    self.stack.push(Data::Float(0.0));
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
                        if let Some(x) = self.stack.pop() {
                            if let Data::Float(x) = x {
                                if x == 0.0 {
                                    self.stack.push(Data::Float(1.0));
                                } else {
                                    self.stack.push(Data::Float(0.0));
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
                        if let Some(x) = self.stack.pop() {
                            self.stack.push(x.clone());
                            self.stack.push(x);
                            Ok(Terminated(false))
                        } else {
                            Err(RuntimeError::StackUnderflow)
                        }
                    }
                    Inst::Jump(offset) => {
                        self.pc += offset - 1;
                        Ok(Terminated(false))
                    }
                    Inst::JumpIfZero(offset) => {
                        if let Some(y) = self.stack.pop() {
                            if let Data::Float(y) = y {
                                if y == 0.0 {
                                    self.pc += offset - 1;
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
                self.pc,
                Vec::from(self.code.clone()),
            )),
        }
    }
}
