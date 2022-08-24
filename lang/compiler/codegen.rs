use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ObjectStates;
use lang_component::{
    syntax::{Body, Expr, Name, Op2, Signature, Symbol, SyntaxTree, Type},
    vm::Inst,
};

type VarInfo = (Type, String);

#[derive(Debug, Clone)]
enum StackData {
    Var(VarInfo),
    State(VarInfo),
    Float,
    Bool,
}

impl From<Type> for StackData {
    fn from(t: Type) -> Self {
        match t {
            Type::Float => StackData::Float,
            Type::Bool => StackData::Bool,
        }
    }
}

impl From<Symbol> for StackData {
    fn from(s: Symbol) -> Self {
        match s {
            Symbol::State(Name(name)) => StackData::State((Type::Float, name.clone())),
            Symbol::Var(Name(name)) => StackData::Var((Type::Float, name.clone())),
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
            StackData::State((_, n)) => n == &name[..],
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
pub struct CodegenState {
    current_proc: Option<String>,
    pub code: Vec<Inst>,
    proc_map: Rc<RefCell<HashMap<String, Proc>>>,
    proc_order: Rc<RefCell<Vec<String>>>,
    stack: StackInfo,
    pub memory: Vec<u8>,
    memory_info: Rc<RefCell<Vec<MemoryInfo>>>,
    current_unresolved: Rc<RefCell<Vec<ResolveInfo>>>,
    object_state_ids: Rc<ObjectStates>,
}

impl CodegenState {
    fn new(
        proc_map: Rc<RefCell<HashMap<String, Proc>>>,
        memory_info: Rc<RefCell<Vec<MemoryInfo>>>,
        object_state_ids: Rc<ObjectStates>,
    ) -> Self {
        Self {
            current_proc: None,
            code: Vec::new(),
            proc_map,
            proc_order: Rc::new(RefCell::new(Vec::new())),
            stack: StackInfo::new(),
            memory: Vec::from([0; 128]),
            memory_info: memory_info,
            current_unresolved: Rc::new(RefCell::new(Vec::new())),
            object_state_ids,
        }
    }

    fn clone_without_code(&self) -> Self {
        let mut state = self.clone();
        state.code = vec![];
        state
    }
}

macro_rules! iter_names_except_main {
    ($state: expr) => {
        $state
            .proc_order
            .borrow()
            .iter()
            .filter(|n| &n[..] != "main")
    };
}

macro_rules! emit {
    ($state: expr, $inst:expr) => {
        $state.code.push($inst);
    };
}

#[derive(Debug, Clone)]
pub enum CodegenError {
    UnknownVMState(String),
    UnknownVariable(String),
    ProcAlreadyDefined(String),
    MainProcIsNotDefined,
    UndefinedProc(String),
    GlobalDefineOnlyAllowsLiteral(Expr),
    GlobalDefineOnlyAllowsToVar(Symbol),
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
            Symbol::State(Name(name)) => {
                if let Some(id) = state.object_state_ids.get_state_id(name) {
                    emit!(state, Inst::Get(id));
                    state.stack.push(StackData::Float);
                } else {
                    return Err(CodegenError::UnknownVMState(name.to_string()));
                }
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
                    _ => todo!("implemented yet!"),
                }
            );
            let _ = state.stack.pop();
            let _ = state.stack.pop();

            state.stack.push(StackData::Float);
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
                Symbol::State(Name(name)) => {
                    if let Some(id) = state.object_state_ids.get_state_id(name) {
                        codegen_expr(expr, state)?;
                        let _ = state.stack.pop();

                        emit!(state, Inst::Set(id));
                    } else {
                        return Err(CodegenError::UnknownVMState(name.to_string()));
                    }
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
        }
    }

    Ok(())
}

fn insert_return_to_body(name: &str, body: &[Body]) -> Vec<Body> {
    let mut body = body.to_vec();

    if let Some(statement) = body.last() {
        match statement {
            Body::Return(_) => body,
            Body::Expr(expr) => {
                if name != "main" {
                    body.push(Body::Return(Some(*expr.clone())));
                }
                body
            }
            _ => {
                body.push(Body::Return(None));
                body
            }
        }
    } else {
        body.push(Body::Return(None));
        body
    }
}

fn codegen_proc(
    name: &str,
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
        };
        proc_stack.push(sd);
    }

    let mut proc_state = state.clone_without_code();
    proc_state.current_proc = Some(name.clone());
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
    stvec: Vec<SyntaxTree>,
    state: &mut CodegenState,
) -> Result<(), CodegenError> {
    for st in stvec.iter() {
        match st {
            SyntaxTree::DefProc(Name(name), signature, body) => {
                codegen_proc(name, signature, body, state)?;
            }
            SyntaxTree::GlobalDefine(Symbol::Var(Name(name)), expr) => match expr {
                Expr::Float(f) => {
                    let mi = MemoryInfo::new(name.to_string(), Type::Float);
                    let offset = mi.calculate_offset(state.memory_info.clone());

                    state.memory_info.borrow_mut().push(mi);

                    let le_4bytes = f.to_le_bytes();
                    for (idx, byte) in state.memory.as_mut_slice()[offset..offset + 4]
                        .iter_mut()
                        .enumerate()
                    {
                        *byte = le_4bytes[idx];
                    }
                }
                Expr::Bool(b) => {
                    let mi = MemoryInfo::new(name.to_string(), Type::Bool);
                    let offset = mi.calculate_offset(state.memory_info.clone());

                    state.memory_info.borrow_mut().push(mi);

                    state.memory[offset] = if *b { 1 } else { 0 };
                }
                expr => {
                    return Err(CodegenError::GlobalDefineOnlyAllowsLiteral(expr.clone()));
                }
            },
            SyntaxTree::GlobalDefine(sym, _) => {
                return Err(CodegenError::GlobalDefineOnlyAllowsToVar(sym.clone()));
            }
        };
    }

    Ok(())
}

fn codegen_pass1_generate_proc_code(
    source: Vec<SyntaxTree>,
    state: &mut CodegenState,
) -> Result<(), CodegenError> {
    codegen_syntax_trees(source, state)
}

fn place_proc_into_code(
    name: &str,
    offset: &mut usize,
    code: &mut Vec<Inst>,
    proc_map: Rc<RefCell<HashMap<String, Proc>>>,
) -> Result<(), CodegenError> {
    if let Some(mut proc) = proc_map.borrow_mut().get_mut(name) {
        proc.offset = *offset;
        for inst in proc.code.iter() {
            code.push(inst.clone());
            *offset += 1;
        }
    } else {
        return Err(CodegenError::MainProcIsNotDefined);
    }

    Ok(())
}

fn codegen_pass2_place_proc_code(state: &mut CodegenState) -> Result<(), CodegenError> {
    let mut offset = 0;

    place_proc_into_code("main", &mut offset, &mut state.code, state.proc_map.clone())?;

    for name in iter_names_except_main!(state) {
        place_proc_into_code(name, &mut offset, &mut state.code, state.proc_map.clone())?;
    }

    Ok(())
}

fn resolve_jumps_in_proc(
    name: &str,
    code: &mut Vec<Inst>,
    proc_map: Rc<RefCell<HashMap<String, Proc>>>,
) -> Result<(), CodegenError> {
    let proc_map = proc_map.borrow();
    let proc = proc_map.get(name);
    if let Some(proc) = proc {
        let proc_head = proc.offset;
        for (offset, name) in proc.unresolved_list.iter() {
            let inst_offset = proc_head + offset;
            let inst = code.get_mut(inst_offset).unwrap();

            if let Some(callee) = proc_map.get(name) {
                *inst = Inst::Float(callee.offset as f32);
            }
        }
    } else {
        return Err(CodegenError::MainProcIsNotDefined);
    }

    Ok(())
}

fn codegen_pass3_resolve_jumps(state: &mut CodegenState) -> Result<(), CodegenError> {
    resolve_jumps_in_proc("main", &mut state.code, state.proc_map.clone())?;

    for name in iter_names_except_main!(state) {
        resolve_jumps_in_proc(name, &mut state.code, state.proc_map.clone())?;
    }

    Ok(())
}

pub struct CodegenResult {
    pub code: Vec<Inst>,
    pub memory: Vec<u8>,
    pub signature: Signature,
}

pub fn codegen(
    source: Vec<SyntaxTree>,
    state_map: ObjectStates,
) -> Result<CodegenResult, CodegenError> {
    let proc_map = Rc::new(RefCell::new(HashMap::new()));
    let memory_info = Rc::new(RefCell::new(Vec::new()));
    let state_map = Rc::new(state_map);
    let mut state = CodegenState::new(proc_map, memory_info, state_map);

    codegen_pass1_generate_proc_code(source, &mut state)?;
    codegen_pass2_place_proc_code(&mut state)?;
    codegen_pass3_resolve_jumps(&mut state)?;

    let signature = state
        .proc_map
        .borrow()
        .get("main")
        .unwrap()
        .signature
        .clone();

    let result = CodegenResult {
        code: state.code,
        memory: state.memory,
        signature,
    };

    Ok(result)
}

#[cfg(test)]
mod codegen_test {
    use super::super::parse::parse;
    use super::super::tokenize::tokenize;
    use super::*;

    fn test_codegen(expected: Vec<Inst>, string: &str) {
        let mut object_states = HashMap::new();
        object_states.insert("x".to_string(), 0);
        object_states.insert("y".to_string(), 1);
        object_states.insert("input_slow".to_string(), 10);
        let object_states = ObjectStates(object_states);

        println!("text: {:?}", string);
        if let Ok(("", tokens)) = tokenize(string) {
            println!("tokens: {:?}", tokens);
            if let Ok((&[], stvec)) = parse(&tokens) {
                if let Ok(CodegenResult { code: actual, .. }) = codegen(stvec, object_states) {
                    println!("actual = {:?}\nexpected = {:?}", actual, expected);

                    assert_eq!(actual.len(), expected.len());
                    let eq = actual.iter().zip(expected.clone()).all(|(a, b)| *a == b);
                    assert!(eq);
                } else {
                    println!("Cannot codegen: {}", string);
                    assert!(false);
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
            vec![Inst::Float(1.0), Inst::Set(0), Inst::Term],
            r##"
            proc main() {
              $x = 1.0
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
                Inst::Set(1),
                Inst::Term,
            ],
            r##"
            proc main() {
              $y = 1.0 + 2.0
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
                Inst::Set(1),
                Inst::Term,
            ],
            r##"
            proc main() {
              $y = 1.0 + 2.0 * 3.0
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_assign_value_with_if_expr() {
        test_codegen(
            vec![
                Inst::Get(0),
                Inst::Get(10),
                Inst::JumpIfFalse(3),
                Inst::Float(4.0),
                Inst::Jump(2),
                Inst::Float(7.0),
                Inst::Add,
                Inst::Set(0),
                Inst::Term,
            ],
            r##"
            proc main() {
              $x = $x + if $input_slow { 4.0 } else { 7.0 }
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_local_variables() {
        test_codegen(
            vec![
                Inst::Float(42.0),
                Inst::Get(0),
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::Set(0),
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              let x = 42.0
              $x = $x + x
            }
            "##,
        );
        test_codegen(
            vec![
                Inst::Float(42.0),
                Inst::Get(0),
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::Set(0),
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              let x = 42.0
              $x = $x + x + x
            }
            "##,
        );
        test_codegen(
            vec![
                Inst::Float(42.0),
                Inst::Float(420.0),
                Inst::Get(0),
                Inst::Float(2.0),
                Inst::Index,
                Inst::Add,
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::Set(0),
                Inst::Drop,
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              let x = 42.0
              let y = 420.0
              $x = $x + x + y
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_global_variable_referencing() {
        test_codegen(
            vec![
                Inst::Get(0),
                Inst::Read(0, Type::Float),
                Inst::Add,
                Inst::Set(0),
                Inst::Term,
            ],
            r##"
            global v = 42.0

            proc main() {
              $x = $x + v
            }
            "##,
        );
        test_codegen(
            vec![
                Inst::Read(0, Type::Float),
                Inst::Float(1.0),
                Inst::Add,
                Inst::Write(0),
                Inst::Term,
            ],
            r##"
            global v = 42.0

            proc main() {
              v = v + 1.0
            }
            "##,
        );
        test_codegen(
            vec![
                Inst::Float(100.0),
                Inst::Get(0),
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::Read(0, Type::Float),
                Inst::Add,
                Inst::Set(0),
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            global g = 42.0

            proc main() {
              let l = 100.0
              $x = $x + l + g
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_proc_call() {
        test_codegen(
            vec![
                // proc main()
                Inst::Get(0),
                Inst::Float(6.0), // address of proc
                Inst::Call,
                Inst::Add,
                Inst::Set(0),
                Inst::Term,
                // proc const42() -> float
                Inst::Float(42.0),
                Inst::Ret(0),
            ],
            r##"
            proc const_42() -> float { return 42 }

            proc main() {
              $x = $x + const_42()
            }
            "##,
        );
        test_codegen(
            vec![
                // proc main()
                Inst::Get(0),
                Inst::Float(5.0), // address of proc
                Inst::Call,
                Inst::Set(0),
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
              $x = add_42($x)
            }
            "##,
        );
        test_codegen(
            vec![
                // proc main()
                Inst::Get(0),
                Inst::Float(10.0), // address of add_42()
                Inst::Call,
                Inst::Set(0),
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
              $x = add_42($x)
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
                Inst::Get(0),
                Inst::Float(42.0),
                Inst::Add,
                Inst::Set(0),
                Inst::Term,
                // proc do_nothing()
                Inst::Float(-4200000.0),
                Inst::Ret(0),
            ],
            r##"
            proc do_nothing() {}

            proc main() {
              do_nothing()
              $x = $x + 42
            }
            "##,
        );
    }
}
