use std::{cell::RefCell, collections::HashMap, rc::Rc};

use lang_component::{
    bullet::StateId,
    syntax::{Body, Expr, Name, Op2, Signature, Symbol, SyntaxTree, Type},
    vm::{ExternalOperation, Inst},
};

type VarInfo = (Type, String);

#[derive(Debug, Clone)]
enum StackData {
    Var(VarInfo),
    Float,
    Bool,
    String,
}

impl From<Type> for StackData {
    fn from(t: Type) -> Self {
        match t {
            Type::Float => StackData::Float,
            Type::Bool => StackData::Bool,
            Type::String => StackData::String,
        }
    }
}

impl From<Symbol> for StackData {
    fn from(s: Symbol) -> Self {
        match s {
            Symbol::Var(Name(name)) => StackData::Var((Type::Float, name.clone())),
            Symbol::Ref(bid, state) => {
                let bid: String = bid.into();
                match state {
                    StateId::PosX => StackData::Var((Type::Float, format!("{}.x", bid))),
                    StateId::PosY => StackData::Var((Type::Float, format!("{}.y", bid))),
                    StateId::InputUp => StackData::Var((Type::Bool, format!("{}.input_up", bid))),
                    StateId::InputDown => {
                        StackData::Var((Type::Bool, format!("{}.input_down", bid)))
                    }
                    StateId::InputLeft => {
                        StackData::Var((Type::Bool, format!("{}.input_left", bid)))
                    }
                    StateId::InputRight => {
                        StackData::Var((Type::Bool, format!("{}.input_right", bid)))
                    }
                    StateId::InputShot => {
                        StackData::Var((Type::Bool, format!("{}.input_shot", bid)))
                    }
                    StateId::InputSlow => {
                        StackData::Var((Type::Bool, format!("{}.input_slow", bid)))
                    }
                    StateId::Enabled => StackData::Var((Type::Bool, format!("{}.enabled", bid))),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct StackInfo {
    info: Vec<StackData>,
    parent: Option<Box<StackInfo>>,
}

impl StackInfo {
    fn new() -> Self {
        Self {
            info: Vec::new(),
            parent: None,
        }
    }

    fn push(&mut self, data: StackData) {
        self.info.push(data);
    }

    fn pop(&mut self) -> Option<StackData> {
        self.info.pop()
    }

    fn get(&self, name: &str) -> Option<(usize, StackData)> {
        //let  len = self.info.len();
        if let Some((idx, sd)) = self.info.iter().enumerate().find(|(_, sd)| match sd {
            StackData::Var((_, n)) => n == &name[..],
            _ => false,
        }) {
            Some((self.info.len() - 1 - idx, sd.clone()))
        } else {
            if let Some(parent) = &self.parent {
                if let Some((idx, sd)) = parent.get(name) {
                    let idx = self.info.len() + idx;
                    Some((idx, sd))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    fn peek(&self, n: usize) -> Option<StackData> {
        self.info.get(self.info.len() - 1 - n).cloned()
    }
}

#[derive(Debug, Clone)]
struct MemoryInfo {
    name: String,
    r#type: Type,
}

impl MemoryInfo {
    pub fn new(name: String, r#type: Type) -> Self {
        Self { name, r#type }
    }

    pub fn calculate_offset(&self, info_list: Rc<RefCell<Vec<MemoryInfo>>>) -> usize {
        let mut offset = 0;

        for mi in info_list.borrow().iter() {
            let size = match mi.r#type {
                Type::Float => 4,
                Type::Bool => 1,
                Type::String => 0, // string stored as [size, ch0, ch1, ...]
            };

            if self.name == mi.name && self.r#type == mi.r#type {
                break;
            } else {
                offset += size;
            }
        }

        offset
    }
}

type ResolveInfo = (usize, String);

#[derive(Debug, Clone)]
struct Proc {
    // コード全体のなかでのオフセット
    offset: usize,
    signature: Signature,
    code: Vec<Inst>,
    // 関数ジャンプ先未解決リスト
    // (命令の位置、呼び出したい関数名)
    unresolved_list: Vec<ResolveInfo>,
}

impl Proc {
    fn new() -> Self {
        Self {
            offset: 0,
            signature: Signature::default(),
            code: Vec::new(),
            unresolved_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProcType {
    Proc,
    Bullet,
    Stage,
}

#[derive(Debug, Clone)]
pub enum ResolveType {
    Proc(String),
    GlobalDef(String),
    LocalDef(String),
}

#[derive(Debug, Clone)]
pub struct UnresolvedProc {
    pub name: String,
    pub r#type: ProcType,
    pub filename: String,
    pub code: Vec<Inst>,
    pub offset_address: usize,
    pub local_memory: Vec<(String, Type)>,
    pub global_memory: Vec<(String, Type)>,
    pub unresolved_list: Vec<(ResolveType, usize)>,
}

#[derive(Debug, Clone)]
pub struct CodegenState {
    procs: Vec<UnresolvedProc>,
    stack: StackInfo,
}

impl CodegenState {
    fn new() -> Self {
        Self {
            procs: Vec::new(),
            stack: StackInfo::new(),
        }
    }
}

macro_rules! emit {
    ($state: expr, $inst:expr) => {
        $state.procs.last().unwrap().code.push($inst)
    };
}

#[derive(Debug, Clone)]
pub enum CodegenError {
    UnknownVMState(String),
    UnknownVariable(String),
    UnknownName(String),
    ProcAlreadyDefined(String),
    MainProcIsNotDefined,
    UndefinedProc(String),
    GlobalDefineOnlyAllowsLiteral(Expr),
    GlobalDefineOnlyAllowsToVar(Symbol),
    WrongParamNumberWhileInvokingExternalOp,
    WrongTypeWhileInvokingExternalOp,
    StringIsNotSupportedHere,
    NotAString,
    BulletRefNotAllowedHere,
}

fn codegen_external_op_fire(args: Vec<Expr>, state: &mut CodegenState) -> Result<(), CodegenError> {
    if args.len() != 3 {
        return Err(CodegenError::WrongParamNumberWhileInvokingExternalOp);
    }

    let bullet_id = if let Expr::String(bullet_name) = &args[0] {
        if let Some((idx, _)) = state
            .compiled_code_vec
            .iter()
            .enumerate()
            .find(|(_, bc)| &bc.name == bullet_name)
        {
            idx
        } else {
            return Err(CodegenError::UnknownName(bullet_name.to_string()));
        }
    } else {
        return Err(CodegenError::NotAString);
    };

    codegen_expr(&args[1], state)?;
    if !matches!(state.stack.peek(0), Some(StackData::Float)) {
        return Err(CodegenError::WrongTypeWhileInvokingExternalOp);
    }

    codegen_expr(&args[2], state)?;
    if !matches!(state.stack.peek(0), Some(StackData::Float)) {
        return Err(CodegenError::WrongTypeWhileInvokingExternalOp);
    }

    let _ = state.stack.pop();
    let _ = state.stack.pop();
    let _ = state.stack.pop();

    emit!(state, Inst::Operate(ExternalOperation::Fire(bullet_id)));
    state.stack.push(StackData::Bool);

    Ok(())
}

fn codegen_external_op_die(args: Vec<Expr>, state: &mut CodegenState) -> Result<(), CodegenError> {
    if args.len() != 0 {
        return Err(CodegenError::WrongParamNumberWhileInvokingExternalOp);
    }

    emit!(state, Inst::Operate(ExternalOperation::Die));

    Ok(())
}

const EXTERNAL_OPS: [(
    &str,
    &dyn Fn(Vec<Expr>, &mut CodegenState) -> Result<(), CodegenError>,
); 2] = [
    ("fire", &codegen_external_op_fire),
    ("die", &codegen_external_op_die),
];

fn codegen_external_op(
    name: &str,
    args: Vec<Expr>,
    state: &mut CodegenState,
) -> Result<(), CodegenError> {
    let op = EXTERNAL_OPS.iter().find(|(n, _)| n == &name);

    if op.is_none() {
        return Err(CodegenError::UndefinedProc(name.to_string()));
    }

    let (_, codegen_fn) = op.unwrap();
    (codegen_fn)(args, state)
}

fn codegen_expr(expr: &Expr, state: &mut CodegenState) -> Result<(), CodegenError> {
    match expr {
        Expr::Float(f) => {
            emit!(state, Inst::Float(*f));
            state.stack.push(StackData::Float);
        }
        Expr::Bool(b) => {
            emit!(state, Inst::Bool(*b));
            state.stack.push(StackData::Bool);
        }
        Expr::String(_) => todo!("treat strings"),
        Expr::Symbol(sym) => match sym {
            Symbol::Ref(bid, sid) => {
                let sd = match *sid {
                    StateId::PosX => StackData::Float,
                    StateId::PosY => StackData::Float,
                    StateId::InputUp => StackData::Bool,
                    StateId::InputDown => StackData::Bool,
                    StateId::InputLeft => StackData::Bool,
                    StateId::InputRight => StackData::Bool,
                    StateId::InputShot => StackData::Bool,
                    StateId::InputSlow => StackData::Bool,
                    StateId::Enabled => StackData::Bool,
                };
                state.stack.push(sd);
                emit!(state, Inst::RefRead(*bid, *sid));
            }
            Symbol::Var(Name(name)) => {
                if let Some((idx, StackData::Var((_, _)))) = state.stack.get(&name[..]) {
                    emit!(state, Inst::Float(idx as f32));
                    state.stack.push(StackData::Float);

                    emit!(state, Inst::Index);

                    return Ok(());
                }

                if let Some(mi) = state
                    .memory_info
                    .borrow()
                    .iter()
                    .find(|mi| mi.name == *name)
                {
                    let offset = mi.calculate_offset(state.memory_info.clone());

                    emit!(state, Inst::Read(offset, mi.r#type));
                    state.stack.push(mi.r#type.into());

                    return Ok(());
                }

                return Err(CodegenError::UnknownVariable(name.to_string()));
            }
        },
        Expr::Op2(op, expr1, expr2) => {
            codegen_expr(expr1, state)?;
            codegen_expr(expr2, state)?;

            emit!(
                state,
                match op {
                    Op2::Add => Inst::Add,
                    Op2::Sub => Inst::Sub,
                    Op2::Mul => Inst::Mul,
                    Op2::Div => Inst::Div,
                    Op2::Mod => Inst::Mod,
                    Op2::Gt => Inst::Gt,
                    Op2::Lt => Inst::Lt,
                    Op2::Gte => Inst::Gte,
                    Op2::Lte => Inst::Lte,
                    Op2::Eq => Inst::EqFloat,
                    Op2::LogOr => Inst::LogOr,
                    Op2::LogAnd => Inst::LogAnd,
                }
            );
            let _ = state.stack.pop();
            let _ = state.stack.pop();

            match op {
                Op2::Add | Op2::Sub | Op2::Mul | Op2::Div | Op2::Mod => {
                    state.stack.push(StackData::Float)
                }
                Op2::Gt | Op2::Lt | Op2::Gte | Op2::Lte | Op2::Eq | Op2::LogOr | Op2::LogAnd => {
                    state.stack.push(StackData::Bool)
                }
            };
        }
        Expr::If(cond, tru, fls) => {
            codegen_expr(cond, state)?;
            let _ = state.stack.pop();
            emit!(state, Inst::JumpIfFalse(-10000));

            let else_jump_from = state.code.len() - 1;

            codegen_expr(tru, state)?;
            emit!(state, Inst::Jump(-20000));
            let true_jump_from = state.code.len() - 1;

            let else_jump_to = state.code.len();
            let else_jump_offset = (else_jump_to - else_jump_from) as i32;
            *state.code.get_mut(else_jump_from).unwrap() = Inst::JumpIfFalse(else_jump_offset);

            codegen_expr(fls, state)?;
            let true_jump_to = state.code.len();
            let true_jump_offset = (true_jump_to - true_jump_from) as i32;
            *state.code.get_mut(true_jump_from).unwrap() = Inst::Jump(true_jump_offset);

            let _ = state.stack.pop();
        }
        Expr::ProcCall(Name(name), args) => {
            match codegen_external_op(name, args.to_vec(), state) {
                Ok(()) => return Ok(()),
                Err(CodegenError::UndefinedProc(_)) => (),
                Err(err) => return Err(err),
            }

            if let None = state.proc_map.borrow().get(name) {
                return Err(CodegenError::UndefinedProc(name.to_string()));
            }

            for arg in args.iter() {
                codegen_expr(arg, state)?;
            }

            let proc_address_offset = state.code.len();
            state
                .current_unresolved
                .borrow_mut()
                .push((proc_address_offset, name.to_string()));
            emit!(state, Inst::Float(-2000.0)); // dummy proc's address

            emit!(state, Inst::Call);
        }
    };

    Ok(())
}

fn codegen_proc_body(
    name: &str,
    arg_num: usize,
    ret: Option<Type>,
    body: &[Body],
    state: &mut CodegenState,
) -> Result<(), CodegenError> {
    for b in body.iter() {
        match b {
            Body::Assignment(sym, expr) => match sym {
                Symbol::Ref(bid, sid) => {
                    codegen_expr(expr, state)?;
                    let _ = state.stack.pop();

                    emit!(state, Inst::RefWrite(*bid, *sid));
                }
                Symbol::Var(Name(name)) => {
                    codegen_expr(expr, state)?;
                    let _ = state.stack.pop();

                    if let Some(mi) = state
                        .memory_info
                        .borrow()
                        .iter()
                        .find(|mi| mi.name == *name)
                    {
                        let offset = mi.calculate_offset(state.memory_info.clone());

                        emit!(state, Inst::Write(offset));
                    } else {
                        return Err(CodegenError::UnknownVariable(name.to_string()));
                    }
                }
            },
            Body::LexicalDefine(sym, expr) => {
                let sd: StackData = sym.clone().into();
                codegen_expr(expr, state)?;
                // remove StackData::Value of expr to replace Var or State
                let _ = state.stack.pop();
                state.stack.push(sd);
            }
            Body::Return(val) => {
                if let Some(expr) = val {
                    codegen_expr(expr, state)?;
                    let _ = state.stack.pop();
                }

                if name == "main" {
                    for _ in 0..state.stack.info.len() {
                        emit!(state, Inst::Drop);
                    }
                    emit!(state, Inst::Term);
                } else {
                    if ret.is_none() {
                        // push dummy value that will be dropped by caller
                        emit!(state, Inst::Float(-4200000.0));
                    }
                    emit!(state, Inst::Ret(arg_num));
                }

                return Ok(());
            }
            Body::Expr(expr) => {
                codegen_expr(expr, state)?;
                emit!(state, Inst::Drop);
                let _ = state.stack.pop();
            }
            _ => todo!(),
        }
    }

    Ok(())
}

fn insert_return_to_body(name: &str, body: &[Body]) -> Vec<Body> {
    let mut body = body.to_vec();
    let mut should_be_replaced = false;

    if let Some(statement) = body.last() {
        match statement {
            Body::Return(_) => (),
            Body::Expr(_) => {
                if name == "main" {
                    body.push(Body::Return(None));
                } else {
                    should_be_replaced = true;
                }
            }
            _ => {
                body.push(Body::Return(None));
            }
        }
    } else {
        body.push(Body::Return(None));
    }

    if should_be_replaced {
        if let Body::Expr(expr) = body[body.len() - 1].clone() {
            let len = body.len();
            body[len - 1] = Body::Return(Some(*expr.clone()));
        }
    }

    body
}

fn codegen_proc(
    name: &str,
    r#type: ProcType,
    sig: &Signature,
    body: &[Body],
    state: &mut CodegenState,
) -> Result<(), CodegenError> {
    let name = name.to_string();

    if let Some(_) = state.proc_map.borrow().get(&name[..]) {
        return Err(CodegenError::ProcAlreadyDefined(name.to_string()));
    }

    let mut proc_stack = StackInfo::new();
    for arg in sig.args.iter() {
        let sd = match arg.r#type {
            Type::Float => StackData::Var((Type::Float, arg.name.0.clone())),
            Type::Bool => StackData::Var((Type::Bool, arg.name.0.clone())),
            Type::String => return Err(CodegenError::StringIsNotSupportedHere),
        };
        proc_stack.push(sd);
    }

    let mut proc_state = state.clone_without_code();
    proc_state.stack = proc_stack;
    proc_state.code = vec![];

    let body = insert_return_to_body(&name[..], body);
    codegen_proc_body(
        &name[..],
        sig.args.len(),
        sig.ret,
        body.as_slice(),
        &mut proc_state,
    )?;

    let mut proc = Proc::new();
    proc.signature = Signature::new(sig.args.to_vec(), sig.ret);
    proc.code = proc_state.code;
    proc.unresolved_list = proc_state.current_unresolved.take();

    state.proc_map.borrow_mut().insert(name.clone(), proc);
    state.proc_order.borrow_mut().push(name.clone());

    Ok(())
}

fn codegen_syntax_trees(
    stvec: &[SyntaxTree],
    state: &mut CodegenState,
) -> Result<(), CodegenError> {
    for st in stvec.iter() {
        match st {
            SyntaxTree::DefProc(Name(name), signature, body) => {
                codegen_proc(name, ProcType::Proc, signature, body, state)?;
            }
            SyntaxTree::DefBullet(Name(name), signature, body) => {
                codegen_proc(name, ProcType::Bullet, signature, body, state)?;
            }
            SyntaxTree::DefStage(Name(name), signature, body) => {
                codegen_proc(name, ProcType::Stage, signature, body, state)?;
            }
            _ => todo!(),
            // SyntaxTree::GlobalDefine(Symbol::Var(Name(name)), expr) => match expr {
            //     Expr::Float(f) => {
            //         let mi = MemoryInfo::new(name.to_string(), Type::Float);
            //         let offset = mi.calculate_offset(state.memory_info.clone());

            //         state.memory_info.borrow_mut().push(mi);

            //         let le_4bytes = f.to_le_bytes();
            //         for (idx, byte) in state.memory.as_mut_slice()[offset..offset + 4]
            //             .iter_mut()
            //             .enumerate()
            //         {
            //             *byte = le_4bytes[idx];
            //         }
            //     }
            //     Expr::Bool(b) => {
            //         let mi = MemoryInfo::new(name.to_string(), Type::Bool);
            //         let offset = mi.calculate_offset(state.memory_info.clone());

            //         state.memory_info.borrow_mut().push(mi);

            //         state.memory[offset] = if *b { 1 } else { 0 };
            //     }
            //     expr => {
            //         return Err(CodegenError::GlobalDefineOnlyAllowsLiteral(expr.clone()));
            //     }
            // },
            // SyntaxTree::GlobalDefine(sym, _) => {
            //     return Err(CodegenError::GlobalDefineOnlyAllowsToVar(sym.clone()));
            // }
        };
    }

    Ok(())
}

pub fn codegen(source: &[SyntaxTree]) -> Result<Vec<UnresolvedProc>, CodegenError> {
    let proc_map = Rc::new(RefCell::new(HashMap::new()));
    let memory_info = Rc::new(RefCell::new(Vec::new()));
    let mut state = CodegenState::new(proc_map, memory_info, compiled_bullet_vec);

    codegen_syntax_trees(source, &mut state)
}

// fn place_proc_into_code(
//     name: &str,
//     offset: &mut usize,
//     code: &mut Vec<Inst>,
//     proc_map: Rc<RefCell<HashMap<String, Proc>>>,
// ) -> Result<(), CodegenError> {
//     if let Some(mut proc) = proc_map.borrow_mut().get_mut(name) {
//         proc.offset = *offset;
//         for inst in proc.code.iter() {
//             code.push(inst.clone());
//             *offset += 1;
//         }
//     } else {
//         return Err(CodegenError::MainProcIsNotDefined);
//     }

//     Ok(())
// }

// fn codegen_pass2_place_proc_code(state: &mut CodegenState) -> Result<(), CodegenError> {
//     let mut offset = 0;

//     place_proc_into_code("main", &mut offset, &mut state.code, state.proc_map.clone())?;

//     for name in iter_names_except_main!(state) {
//         place_proc_into_code(name, &mut offset, &mut state.code, state.proc_map.clone())?;
//     }

//     Ok(())
// }

// fn resolve_jumps_in_proc(
//     name: &str,
//     code: &mut Vec<Inst>,
//     proc_map: Rc<RefCell<HashMap<String, Proc>>>,
// ) -> Result<(), CodegenError> {
//     let proc_map = proc_map.borrow();
//     let proc = proc_map.get(name);
//     if let Some(proc) = proc {
//         let proc_head = proc.offset;
//         for (offset, name) in proc.unresolved_list.iter() {
//             let inst_offset = proc_head + offset;
//             let inst = code.get_mut(inst_offset).unwrap();

//             if let Some(callee) = proc_map.get(name) {
//                 *inst = Inst::Float(callee.offset as f32);
//             }
//         }
//     } else {
//         return Err(CodegenError::MainProcIsNotDefined);
//     }

//     Ok(())
// }

// fn codegen_pass3_resolve_jumps(state: &mut CodegenState) -> Result<(), CodegenError> {
//     resolve_jumps_in_proc("main", &mut state.code, state.proc_map.clone())?;

//     for name in iter_names_except_main!(state) {
//         resolve_jumps_in_proc(name, &mut state.code, state.proc_map.clone())?;
//     }

//     Ok(())
// }

// pub struct CodegenResult {
//     pub code: Vec<Inst>,
//     pub memory: Vec<u8>,
//     pub signature: Signature,
// }

// pub fn codegen(
//     source: Vec<SyntaxTree>,
//     compiled_bullet_vec: &Vec<Rc<BulletCode>>,
// ) -> Result<CodegenResult, CodegenError> {
//     let proc_map = Rc::new(RefCell::new(HashMap::new()));
//     let memory_info = Rc::new(RefCell::new(Vec::new()));
//     let mut state = CodegenState::new(proc_map, memory_info, compiled_bullet_vec);

//     codegen_pass1_generate_proc_code(source, &mut state)?;
//     codegen_pass2_place_proc_code(&mut state)?;
//     codegen_pass3_resolve_jumps(&mut state)?;

//     let signature = state
//         .proc_map
//         .borrow()
//         .get("main")
//         .unwrap()
//         .signature
//         .clone();

//     let result = CodegenResult {
//         code: state.code,
//         memory: state.memory,
//         signature,
//     };

//     Ok(result)
// }

#[cfg(test)]
mod codegen_test {
    use lang_component::bullet::BulletId;

    use super::super::parse::parse;
    use super::super::tokenize::tokenize;
    use super::*;

    fn test_codegen(expected: Vec<Inst>, string: &str) {
        let mut compiled_bullet_vec = Vec::new();
        compiled_bullet_vec.push(Rc::new(BulletCode::new("bullet_0")));
        compiled_bullet_vec.push(Rc::new(BulletCode::new("bullet_1")));
        compiled_bullet_vec.push(Rc::new(BulletCode::new("bullet_2")));

        println!("text: {:?}", string);
        if let Ok(("", tokens)) = tokenize(string) {
            println!("tokens: {:?}", tokens);
            if let Ok((&[], stvec)) = parse(&tokens) {
                match codegen(stvec, &compiled_bullet_vec) {
                    Ok(CodegenResult { code: actual, .. }) => {
                        println!("actual = {:?}\nexpected = {:?}", actual, expected);

                        assert_eq!(actual.len(), expected.len());
                        let eq = actual.iter().zip(expected.clone()).all(|(a, b)| *a == b);
                        assert!(eq);
                    }
                    Err(err) => {
                        println!("Cannot codegen: {} because {:?}", string, err);
                        assert!(false);
                    }
                }
            } else {
                println!("Cannot parse source: {}", string);
                assert!(false);
            }
        } else {
            println!("Cannot tokenize source: {}", string);
            assert!(false);
        }
    }

    #[test]
    fn test_codegen_assign_value_to_px() {
        test_codegen(
            vec![
                Inst::Float(1.0),
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Term,
            ],
            r##"
            proc main() {
              self.x = 1.0
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_assign_binop_value_to_y() {
        test_codegen(
            vec![
                Inst::Float(1.0),
                Inst::Float(2.0),
                Inst::Add,
                Inst::RefWrite(BulletId::Itself, StateId::PosY),
                Inst::Term,
            ],
            r##"
            proc main() {
              self.y = 1.0 + 2.0
            }
            "##,
        );
        test_codegen(
            vec![
                Inst::Float(1.0),
                Inst::Float(2.0),
                Inst::Float(3.0),
                Inst::Mul,
                Inst::Add,
                Inst::RefWrite(BulletId::Itself, StateId::PosY),
                Inst::Term,
            ],
            r##"
            proc main() {
              self.y = 1.0 + 2.0 * 3.0
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_assign_value_with_if_expr() {
        test_codegen(
            vec![
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::RefRead(BulletId::Itself, StateId::InputSlow),
                Inst::JumpIfFalse(3),
                Inst::Float(4.0),
                Inst::Jump(2),
                Inst::Float(7.0),
                Inst::Add,
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Term,
            ],
            r##"
            proc main() {
              self.x = self.x + if self.input_slow { 4.0 } else { 7.0 }
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_local_variables() {
        test_codegen(
            vec![
                Inst::Float(42.0),
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              let x = 42.0
              self.x = self.x + x
            }
            "##,
        );
        test_codegen(
            vec![
                Inst::Float(42.0),
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              let x = 42.0
              self.x = self.x + x + x
            }
            "##,
        );
        test_codegen(
            vec![
                Inst::Float(42.0),
                Inst::Float(420.0),
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::Float(2.0),
                Inst::Index,
                Inst::Add,
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Drop,
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              let x = 42.0
              let y = 420.0
              self.x = self.x + x + y
            }
            "##,
        );
    }

    // #[test]
    // fn test_codegen_global_variable_referencing() {
    //     test_codegen(
    //         vec![
    //             Inst::RefRead(BulletId::Itself, StateId::PosX),
    //             Inst::Read(0, Type::Float),
    //             Inst::Add,
    //             Inst::RefWrite(BulletId::Itself, StateId::PosX),
    //             Inst::Term,
    //         ],
    //         r##"
    //         global v = 42.0

    //         proc main() {
    //           self.x = self.x + v
    //         }
    //         "##,
    //     );
    //     test_codegen(
    //         vec![
    //             Inst::Read(0, Type::Float),
    //             Inst::Float(1.0),
    //             Inst::Add,
    //             Inst::Write(0),
    //             Inst::Term,
    //         ],
    //         r##"
    //         global v = 42.0

    //         proc main() {
    //           v = v + 1.0
    //         }
    //         "##,
    //     );
    //     test_codegen(
    //         vec![
    //             Inst::Float(100.0),
    //             Inst::RefRead(BulletId::Itself, StateId::PosX),
    //             Inst::Float(1.0),
    //             Inst::Index,
    //             Inst::Add,
    //             Inst::Read(0, Type::Float),
    //             Inst::Add,
    //             Inst::RefWrite(BulletId::Itself, StateId::PosX),
    //             Inst::Drop,
    //             Inst::Term,
    //         ],
    //         r##"
    //         global g = 42.0

    //         proc main() {
    //           let l = 100.0
    //           self.x = self.x + l + g
    //         }
    //         "##,
    //     );
    // }

    #[test]
    fn test_codegen_proc_call() {
        test_codegen(
            vec![
                // proc main()
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::Float(6.0), // address of proc
                Inst::Call,
                Inst::Add,
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Term,
                // proc const42() -> float
                Inst::Float(42.0),
                Inst::Ret(0),
            ],
            r##"
            proc const_42() -> float { return 42 }

            proc main() {
              self.x = self.x + const_42()
            }
            "##,
        );
        test_codegen(
            vec![
                // proc main()
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::Float(5.0), // address of proc
                Inst::Call,
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Term,
                // proc add_42() -> float
                Inst::Float(0.0),
                Inst::Index,
                Inst::Float(42.0),
                Inst::Add,
                Inst::Ret(1),
            ],
            r##"
            proc add_42(n: float) -> float { return n + 42 }

            proc main() {
              self.x = add_42(self.x)
            }
            "##,
        );
        test_codegen(
            vec![
                // proc main()
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::Float(10.0), // address of add_42()
                Inst::Call,
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Term,
                // proc add_10() -> float
                Inst::Float(0.0),
                Inst::Index,
                Inst::Float(10.0),
                Inst::Add,
                Inst::Ret(1),
                // proc add_42() -> float
                Inst::Float(0.0),
                Inst::Index,
                Inst::Float(5.0), // address of add_10()
                Inst::Call,
                Inst::Float(32.0),
                Inst::Add,
                Inst::Ret(1),
            ],
            r##"
            proc add_10(n: float) -> float { return n + 10 }
            proc add_42(n: float) -> float { return add_10(n) + 32 }

            proc main() {
              self.x = add_42(self.x)
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_proc_call_without_return() {
        test_codegen(
            vec![
                // proc main()
                Inst::Float(8.0), // address of do_nothing()
                Inst::Call,
                Inst::Drop,
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::Float(42.0),
                Inst::Add,
                Inst::RefWrite(BulletId::Itself, StateId::PosX),
                Inst::Term,
                // proc do_nothing()
                Inst::Float(-4200000.0),
                Inst::Ret(0),
            ],
            r##"
            proc do_nothing() {}

            proc main() {
              do_nothing()
              self.x = self.x + 42
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_external_operation() {
        test_codegen(
            vec![
                Inst::RefRead(BulletId::Itself, StateId::PosX),
                Inst::RefRead(BulletId::Itself, StateId::PosY),
                Inst::Operate(ExternalOperation::Fire(1)),
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              fire("bullet_1", self.x, self.y)
            }
            "##,
        );
    }
}
