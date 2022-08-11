use std::collections::VecDeque;

use lang_component::{
    syntax::Type,
    vm::{Data, Inst},
};

use crate::{
    bullet::{BulletColor, BulletType, Operation, WriteState},
    error::RuntimeError,
    r#macro::*,
    VM,
};

pub struct Terminated(pub bool);

impl VM {
    pub fn new() -> Self {
        VM {
            pc: 0,
            code: Vec::new().into(),
            stack: Vec::new(),
            memory: Vec::from([0; 128]),
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
        //self.stack.clear();
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
                    Inst::Read(offset, r#type) => {
                        let offset = *offset;
                        check_memory_bound!(self.memory, offset, *r#type);

                        match r#type {
                            Type::Float => {
                                let le_4bytes: Result<[u8; 4], _> =
                                    self.memory.as_slice()[offset..].try_into();
                                if let Err(err) = le_4bytes {
                                    return Err(RuntimeError::CannotDecodeFloat(err));
                                }
                                let f = f32::from_le_bytes(le_4bytes.unwrap());
                                self.stack.push(Data::Float(f));
                            }
                            Type::Bool => {
                                let u8_bool = *self.memory.get(offset).unwrap();
                                self.stack.push(Data::Bool(u8_bool != 0));
                            }
                        }

                        Ok(Terminated(false))
                    }
                    Inst::Write(offset) => {
                        let data = stack_pop!(self.stack);
                        let offset = *offset;
                        check_memory_bound!(self.memory, offset, data.r#type());

                        match data {
                            Data::Float(f) => {
                                let le_4bytes = f.to_le_bytes();
                                for (idx, byte) in
                                    self.memory.as_mut_slice()[offset..4].iter_mut().enumerate()
                                {
                                    *byte = le_4bytes[idx];
                                }
                            }
                            Data::Bool(b) => {
                                self.memory[offset] = if b { 1 } else { 0 };
                            }
                        };

                        Ok(Terminated(false))
                    }
                    Inst::Float(f) => {
                        self.stack.push(Data::Float(*f));
                        Ok(Terminated(false))
                    }
                    Inst::Bool(b) => {
                        self.stack.push(Data::Bool(*b));
                        Ok(Terminated(false))
                    }
                    Inst::Get(name) => {
                        match name.as_str() {
                            "Pos:X" => self.stack.push(Data::Float(state.pos_x())),
                            "Pos:Y" => self.stack.push(Data::Float(state.pos_y())),
                            "Input:Up" => self.stack.push(Data::Bool(state.input_up())),
                            "Input:Down" => self.stack.push(Data::Bool(state.input_down())),
                            "Input:Left" => self.stack.push(Data::Bool(state.input_left())),
                            "Input:Right" => self.stack.push(Data::Bool(state.input_right())),
                            "Input:Shot" => self.stack.push(Data::Bool(state.input_shot())),
                            "Input:Slow" => self.stack.push(Data::Bool(state.input_slow())),
                            _ => return Err(RuntimeError::UnknownStateName(name.to_owned())),
                        };
                        Ok(Terminated(false))
                    }
                    Inst::Set(name) => {
                        let d = stack_pop!(self.stack);
                        match name.as_str() {
                            "Pos:X" => {
                                #[allow(irrefutable_let_patterns)]
                                let x = float_data!(d);
                                //println!("SetPosX: pos.x <- {}", x);
                                state.set_pos_x(x);
                                Ok(Terminated(false))
                            }
                            "Pos:Y" => {
                                #[allow(irrefutable_let_patterns)]
                                let y = float_data!(d);
                                println!("SetPosX: pos.x <- {}", y);
                                state.set_pos_y(y);
                                Ok(Terminated(false))
                            }
                            _ => return Err(RuntimeError::UnknownStateName(name.to_owned())),
                        }
                    }
                    Inst::Fire(blang_name) => {
                        let y = stack_pop!(self.stack);
                        let x = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let x = float_data!(x);
                        #[allow(irrefutable_let_patterns)]
                        let y = float_data!(y);
                        ops_queue.push_back(Operation::PutBullet(
                            x,
                            y,
                            blang_name.to_string(),
                            BulletType::Bullet1,
                            BulletColor::White,
                        ));
                        Ok(Terminated(false))
                    }
                    Inst::Add | Inst::Sub | Inst::Mul => {
                        let b = stack_pop!(self.stack);
                        let a = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let a = float_data!(a);
                        #[allow(irrefutable_let_patterns)]
                        let b = float_data!(b);
                        self.stack.push(Data::Float(match inst {
                            Inst::Add => a + b,
                            Inst::Sub => a - b,
                            Inst::Mul => a * b,
                            _ => unreachable!(),
                        }));

                        Ok(Terminated(false))
                    }
                    Inst::EqInt => {
                        let b = stack_pop!(self.stack);
                        let a = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let a = float_data!(a);
                        #[allow(irrefutable_let_patterns)]
                        let b = float_data!(b);

                        if a == b {
                            self.stack.push(Data::Bool(true));
                        } else {
                            self.stack.push(Data::Bool(false));
                        }

                        Ok(Terminated(false))
                    }
                    Inst::EqFloat => {
                        let b = stack_pop!(self.stack);
                        let a = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let a = float_data!(a);
                        #[allow(irrefutable_let_patterns)]
                        let b = float_data!(b);

                        if a == b {
                            self.stack.push(Data::Bool(true));
                        } else {
                            self.stack.push(Data::Bool(false));
                        }

                        Ok(Terminated(false))
                    }
                    Inst::Lt => {
                        let b = stack_pop!(self.stack);
                        let a = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let a = float_data!(a);
                        #[allow(irrefutable_let_patterns)]
                        let b = float_data!(b);

                        if a > b {
                            self.stack.push(Data::Float(1.0));
                        } else {
                            self.stack.push(Data::Float(0.0));
                        }

                        Ok(Terminated(false))
                    }
                    Inst::Not => {
                        let b = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let b = bool_data!(b);

                        self.stack.push(Data::Bool(!b));
                        Ok(Terminated(false))
                    }
                    Inst::Dup => {
                        let x = stack_pop!(self.stack);
                        self.stack.push(x.clone());
                        self.stack.push(x);
                        Ok(Terminated(false))
                    }
                    Inst::Drop => {
                        let _ = self.stack.pop();
                        Ok(Terminated(false))
                    }
                    Inst::Index => {
                        let n = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let n = float_data!(n) as usize;

                        if self.stack.len() <= n {
                            return Err(RuntimeError::StackUnderflow);
                        }

                        let idx = self.stack.len() - 1 - n;
                        let data = self.stack.iter().nth(idx).unwrap();
                        self.stack.push(data.clone());

                        Ok(Terminated(false))
                    }
                    Inst::Jump(offset) => {
                        self.pc += offset - 1;
                        Ok(Terminated(false))
                    }
                    Inst::JumpIfFalse(offset) => {
                        let b = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let b = bool_data!(b);
                        if !b {
                            self.pc += offset - 1;
                        }
                        Ok(Terminated(false))
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
