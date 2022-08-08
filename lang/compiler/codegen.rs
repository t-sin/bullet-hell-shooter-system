use lang_component::{
    syntax::{Body, Expr, Name, Op2, Symbol, SyntaxTree, Type},
    vm::{get_vm_name, Inst},
};

type VarInfo = (Type, String);

#[derive(Debug, Clone)]
enum StackData {
    Var(VarInfo),
    State(VarInfo),
    Float,
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

    fn index(&self, n: u32) -> Option<&StackData> {
        // TODO: consider its parent
        let idx = self.info.len() - 1 - n as usize;
        self.info.iter().nth(idx)
    }

    fn get(&self, name: &str) -> Option<(usize, StackData)> {
        println!("name = {}, stack = {:?}", name, self);
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

#[derive(Debug)]
struct CodegenState {
    code: Vec<Inst>,
    stack: StackInfo,
}

impl CodegenState {
    fn new() -> Self {
        Self {
            code: Vec::new(),
            stack: StackInfo::new(),
        }
    }

    pub fn set_parent_stack(mut self, parent: Box<StackInfo>) -> Self {
        self.stack.parent = Some(parent.clone());
        self
    }

    fn append_code(&mut self, other: &mut Self) {
        self.code.append(&mut other.code)
    }
}

fn codegen_expr(expr: &Expr, state: &mut CodegenState) {
    println!("expr = {:?}, state = {:?}", expr, state);

    match expr {
        Expr::Float(f) => {
            state.code.push(Inst::Float(*f));
            state.stack.push(StackData::Float);
        }
        Expr::String(_) => todo!("treat strings"),
        Expr::Symbol(sym) => match sym {
            Symbol::State(Name(name)) => {
                if let Some(name) = get_vm_name(name) {
                    state.code.push(Inst::Get(name));
                    state.stack.push(StackData::Float);
                } else {
                    panic!("unknown VM state name: {}", name);
                }
            }
            Symbol::Var(Name(name)) => {
                if let Some((idx, StackData::Var((_, _)))) = state.stack.get(&name[..]) {
                    state.code.push(Inst::Float((idx + 1) as f32));
                    state.stack.push(StackData::Float);

                    state.code.push(Inst::Index);
                } else {
                    panic!("undefined var: {}", name);
                }
            }
        },
        Expr::Op2(op, expr1, expr2) => {
            codegen_expr(expr1, state);
            let _ = state.stack.pop();
            codegen_expr(expr2, state);
            let _ = state.stack.pop();

            state.code.push(match op {
                Op2::Add => Inst::Add,
                Op2::Sub => Inst::Sub,
                Op2::Mul => Inst::Mul,
                _ => todo!("implemented yet!"),
            });
            state.stack.push(StackData::Float);
        }
        Expr::If(cond, tru, fls) => {
            let mut trustate = CodegenState::new().set_parent_stack(Box::new(state.stack.clone()));
            codegen_expr(tru, &mut trustate);
            let mut flsstate = CodegenState::new().set_parent_stack(Box::new(state.stack.clone()));
            codegen_expr(fls, &mut flsstate);
            let true_len = trustate.code.len();
            let false_len = flsstate.code.len();

            // conditional parts
            codegen_expr(cond, state);
            state.code.push(Inst::Float(1.0));

            state.code.push(Inst::EqFloat);
            state.code.push(Inst::JumpIfZero(true_len + 2)); //  true clause + Jump + 1

            // true clause
            state.append_code(&mut trustate);
            state.code.push(Inst::Jump(false_len + 1)); //  false clause + 1

            // false clause
            state.append_code(&mut flsstate);
        }
    }
}

fn codegen_main(body: &[Body], state: &mut CodegenState) {
    for b in body.iter() {
        match b {
            Body::Assignment(sym, expr) => match sym {
                Symbol::State(Name(name)) => {
                    if let Some(name) = get_vm_name(name) {
                        println!("state = {:?}", state);
                        codegen_expr(expr, state);
                        let _ = state.stack.pop();

                        state.code.push(Inst::Set(name));
                    } else {
                        panic!("unknown VM state name: {}", name);
                    }
                }
                Symbol::Var(_) => todo! {"treat lexical variables"},
            },
            Body::LexicalDefine(sym, expr) => {
                let sd = match sym {
                    Symbol::State(Name(name)) => StackData::State((Type::Float, name.clone())),
                    Symbol::Var(Name(name)) => StackData::Var((Type::Float, name.clone())),
                };
                codegen_expr(expr, state);
                // remove StackData::Value of expr to replace Var or State
                let _ = state.stack.pop();
                state.stack.push(sd);
            }
            _ => todo!(""),
        }
    }

    for _ in 0..state.stack.info.len() {
        state.code.push(Inst::Drop)
    }
}

fn codegen_syntax_trees(stvec: Vec<SyntaxTree>, state: &mut CodegenState) {
    for st in stvec.iter() {
        match st {
            SyntaxTree::DefProc(Name(name), _, body) => {
                if name == "main" {
                    codegen_main(body, state)
                } else {
                    todo!("treat not-an-entrypoint functions")
                }
            }
            _ => todo!("treat global variables maybe with memory?"),
        };
    }
}

pub fn codegen(source: Vec<SyntaxTree>) -> Vec<Inst> {
    let mut state = CodegenState::new();

    codegen_syntax_trees(source, &mut state);
    state.code.push(Inst::Term);

    state.code.clone()
}

#[cfg(test)]
mod codegen_test {
    use super::super::parse::parse;
    use super::super::tokenize::tokenize;
    use super::*;

    fn test_codegen(expected: Vec<Inst>, string: &str) {
        println!("text: {:?}", string);
        if let Ok(("", tokens)) = tokenize(string) {
            println!("tokens: {:?}", tokens);
            if let Ok((&[], stvec)) = parse(&tokens) {
                let actual = codegen(stvec);
                println!("actual = {:?}\nexpected = {:?}", actual, expected);

                assert_eq!(actual.len(), expected.len());
                let eq = actual.iter().zip(expected.clone()).all(|(a, b)| *a == b);
                assert!(eq);
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
            vec![Inst::Float(1.0), Inst::Set("Pos:X".to_string()), Inst::Term],
            r##"
            proc main() {
              $px = 1.0
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_assign_binop_value_to_py() {
        test_codegen(
            vec![
                Inst::Float(1.0),
                Inst::Float(2.0),
                Inst::Add,
                Inst::Set("Pos:Y".to_string()),
                Inst::Term,
            ],
            r##"
            proc main() {
              $py = 1.0 + 2.0
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
                Inst::Set("Pos:Y".to_string()),
                Inst::Term,
            ],
            r##"
            proc main() {
              $py = 1.0 + 2.0 * 3.0
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_assign_value_with_if_expr() {
        test_codegen(
            vec![
                Inst::Get("Pos:X".to_string()),
                Inst::Get("Input:Slow".to_string()),
                Inst::Float(1.0),
                Inst::EqFloat,
                Inst::JumpIfZero(3),
                Inst::Float(4.0),
                Inst::Jump(2),
                Inst::Float(7.0),
                Inst::Add,
                Inst::Set("Pos:X".to_string()),
                Inst::Term,
            ],
            r##"
            proc main() {
              $px = $px + if $input_slow { 4.0 } else { 7.0 }
            }
            "##,
        );
    }

    #[test]
    fn test_codegen_local_variables() {
        test_codegen(
            vec![
                Inst::Float(42.0),
                Inst::Get("Pos:X".to_string()),
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::Set("Pos:X".to_string()),
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              let x = 42.0
              $px = $px + x
            }
            "##,
        );
        test_codegen(
            vec![
                Inst::Float(42.0),
                Inst::Float(420.0),
                Inst::Get("Pos:X".to_string()),
                Inst::Float(2.0),
                Inst::Index,
                Inst::Add,
                Inst::Float(1.0),
                Inst::Index,
                Inst::Add,
                Inst::Set("Pos:X".to_string()),
                Inst::Drop,
                Inst::Drop,
                Inst::Term,
            ],
            r##"
            proc main() {
              let x = 42.0
              let y = 420.0
              $px = $px + x + y
            }
            "##,
        );
    }
}
