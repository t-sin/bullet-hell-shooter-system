use std::collections::VecDeque;

use lang_component::{
    bullet::{BulletColor, BulletId, BulletType},
    syntax::Type,
    vm::{Data, ExternalOperation, Inst, OperationQuery},
};

use crate::{error::RuntimeError, r#macro::*, Continuation, SuspendingReason};

impl Continuation {
    pub fn start(
        &mut self,
        code: &Vec<Inst>,
        op_queue: &mut VecDeque<OperationQuery>,
    ) -> Result<SuspendingReason, RuntimeError> {
        //self.stack.clear();
        self.pc = 0;

        self.resume(code, op_queue)
    }

    pub fn resume(
        &mut self,
        code: &Vec<Inst>,
        op_queue: &mut VecDeque<OperationQuery>,
    ) -> Result<SuspendingReason, RuntimeError> {
        loop {
            match self.interpret1(code, op_queue) {
                Ok(reason) => match reason {
                    SuspendingReason::Running => continue,
                    _ => return Ok(reason),
                },
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }

    fn interpret1(
        &mut self,
        code: &Vec<Inst>,
        op_queue: &mut VecDeque<OperationQuery>,
    ) -> Result<SuspendingReason, RuntimeError> {
        let pc = self.pc;
        let inst = code.get(pc);
        self.pc += 1;

        match inst {
            Some(inst) => match inst {
                Inst::Term => Ok(SuspendingReason::Terminated),
                Inst::Operate(op) => match op {
                    ExternalOperation::FireDummy(_) => unreachable!(),
                    ExternalOperation::Fire(id) => {
                        let y = stack_pop!(self.stack);
                        let x = stack_pop!(self.stack);
                        #[allow(irrefutable_let_patterns)]
                        let x = float_data!(x);
                        #[allow(irrefutable_let_patterns)]
                        let y = float_data!(y);

                        let query = OperationQuery::Fire(
                            *id,
                            (x, y),
                            BulletType::Bullet1,
                            BulletColor::White,
                            vec![],
                        );
                        op_queue.push_front(query);

                        Ok(SuspendingReason::Running)
                    }
                    ExternalOperation::Die => {
                        let query = OperationQuery::Die(BulletId::Itself);
                        op_queue.push_front(query);

                        Ok(SuspendingReason::Running)
                    }
                },
                Inst::Read(offset, r#type) => {
                    let offset = *offset;
                    check_memory_bound!(self.memory, offset, *r#type);

                    match r#type {
                        Type::Float => {
                            let le_4bytes: Result<[u8; 4], _> =
                                self.memory.as_slice()[offset..offset + 4].try_into();
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
                        Type::String => todo!("storing strings in memory is not implemented"),
                    }

                    Ok(SuspendingReason::Running)
                }
                Inst::Write(offset) => {
                    let data = stack_pop!(self.stack);
                    let offset = *offset;
                    check_memory_bound!(self.memory, offset, data.r#type());

                    match data {
                        Data::Float(f) => {
                            let le_4bytes = f.to_le_bytes();
                            for (idx, byte) in self.memory.as_mut_slice()[offset..offset + 4]
                                .iter_mut()
                                .enumerate()
                            {
                                *byte = le_4bytes[idx];
                            }
                        }
                        Data::Bool(b) => {
                            self.memory[offset] = if b { 1 } else { 0 };
                        }
                        Data::String(_) => todo!("storing strings in memory is not implemented"),
                    };

                    Ok(SuspendingReason::Running)
                }
                Inst::ReadGlobal(_offset, r#_type) => todo!(),
                Inst::WriteGlobal(_offset) => todo!(),
                Inst::Float(f) => {
                    self.stack.push(Data::Float(*f));
                    Ok(SuspendingReason::Running)
                }
                Inst::Bool(b) => {
                    self.stack.push(Data::Bool(*b));
                    Ok(SuspendingReason::Running)
                }
                Inst::RefRead(bid, sid) => Ok(SuspendingReason::ToReadState(*bid, *sid)),
                Inst::RefWrite(bid, sid) => {
                    let d = stack_pop!(self.stack);
                    Ok(SuspendingReason::ToWriteState(*bid, *sid, d))
                }
                Inst::Add | Inst::Sub | Inst::Mul | Inst::Div | Inst::Mod => {
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
                        Inst::Div => a / b,
                        Inst::Mod => a % b,
                        _ => unreachable!(),
                    }));

                    Ok(SuspendingReason::Running)
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

                    Ok(SuspendingReason::Running)
                }
                Inst::Gt | Inst::Lt | Inst::Gte | Inst::Lte | Inst::EqFloat => {
                    let b = stack_pop!(self.stack);
                    let a = stack_pop!(self.stack);
                    #[allow(irrefutable_let_patterns)]
                    let a = float_data!(a);
                    #[allow(irrefutable_let_patterns)]
                    let b = float_data!(b);

                    let res = match inst {
                        Inst::Gt => a > b,
                        Inst::Lt => a < b,
                        Inst::Gte => a >= b,
                        Inst::Lte => a <= b,
                        Inst::EqFloat => a == b,
                        _ => unreachable!(),
                    };

                    if res {
                        self.stack.push(Data::Bool(true));
                    } else {
                        self.stack.push(Data::Bool(false));
                    }

                    Ok(SuspendingReason::Running)
                }
                Inst::LogOr | Inst::LogAnd => {
                    let b = stack_pop!(self.stack);
                    let a = stack_pop!(self.stack);
                    #[allow(irrefutable_let_patterns)]
                    let a = bool_data!(a);
                    #[allow(irrefutable_let_patterns)]
                    let b = bool_data!(b);

                    let res = match inst {
                        Inst::LogOr => a || b,
                        Inst::LogAnd => a && b,
                        _ => unreachable!(),
                    };

                    if res {
                        self.stack.push(Data::Bool(true));
                    } else {
                        self.stack.push(Data::Bool(false));
                    }

                    Ok(SuspendingReason::Running)
                }
                Inst::Not => {
                    let b = stack_pop!(self.stack);
                    #[allow(irrefutable_let_patterns)]
                    let b = bool_data!(b);

                    self.stack.push(Data::Bool(!b));
                    Ok(SuspendingReason::Running)
                }
                Inst::Dup => {
                    let x = stack_pop!(self.stack);
                    self.stack.push(x.clone());
                    self.stack.push(x);
                    Ok(SuspendingReason::Running)
                }
                Inst::Drop => {
                    let _ = self.stack.pop();
                    Ok(SuspendingReason::Running)
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

                    Ok(SuspendingReason::Running)
                }
                Inst::Jump(offset) => {
                    let next_pc = offset - 1;
                    if next_pc < 0 || code.len() as i32 <= next_pc {
                        return Err(RuntimeError::OutOfCode(next_pc, code.to_vec()));
                    }

                    self.pc += next_pc as usize;
                    Ok(SuspendingReason::Running)
                }
                Inst::JumpIfFalse(offset) => {
                    let b = stack_pop!(self.stack);
                    #[allow(irrefutable_let_patterns)]
                    let b = bool_data!(b);

                    if !b {
                        let next_pc = offset - 1;
                        if next_pc < 0 || code.len() as i32 <= next_pc {
                            return Err(RuntimeError::OutOfCode(next_pc, code.to_vec()));
                        }

                        self.pc += next_pc as usize;
                    }
                    Ok(SuspendingReason::Running)
                }
                Inst::Call => {
                    let f = stack_pop!(self.stack);
                    #[allow(irrefutable_let_patterns)]
                    let offset = float_data!(f);
                    let offset = offset as i32;

                    if offset < 0 || code.len() as i32 <= offset {
                        return Err(RuntimeError::OutOfCode(offset, code.to_vec()));
                    }

                    self.rstack.push(self.pc);

                    self.pc = offset as usize;
                    Ok(SuspendingReason::Running)
                }
                Inst::Ret(num) => {
                    if let Some(ret) = self.rstack.pop() {
                        for _ in 0..*num {
                            let _ = self.stack.pop();
                        }
                        self.pc = ret;

                        Ok(SuspendingReason::Running)
                    } else {
                        Err(RuntimeError::ReturnStackUnderflow)
                    }
                }
            },
            None => Err(RuntimeError::OutOfCode(
                self.pc as i32,
                Vec::from(code.clone()),
            )),
        }
    }
}
