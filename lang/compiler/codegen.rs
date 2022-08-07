use lang_component::{
    syntax::{Body, Expr, Name, Op2, Symbol, SyntaxTree},
    vm::{get_vm_name, Inst},
};

struct CodegenState {
    code: Vec<Inst>,
}

impl CodegenState {
    fn new() -> Self {
        Self { code: Vec::new() }
    }

    fn append(&mut self, other: &mut Self) {
        self.code.append(&mut other.code)
    }
}

fn codegen_expr(expr: &Expr, state: &mut CodegenState) {
    match expr {
        Expr::Float(f) => state.code.push(Inst::Float(*f)),
        Expr::String(_) => todo!("treat strings"),
        Expr::Symbol(sym) => match sym {
            Symbol::State(Name(name)) => {
                if let Some(name) = get_vm_name(name) {
                    state.code.push(Inst::Get(name))
                } else {
                    panic!("unknown VM state name: {}", name);
                }
            }
            Symbol::Var(_) => todo! {"treat lexical variables"},
        },
        Expr::Op2(op, expr1, expr2) => {
            codegen_expr(expr1, state);
            codegen_expr(expr2, state);
            state.code.push(match op {
                Op2::Add => Inst::Add,
                Op2::Sub => Inst::Sub,
                Op2::Mul => Inst::Mul,
                _ => todo!("implemented yet!"),
            });
        }
        Expr::If(cond, tru, fls) => {
            let mut trustate = CodegenState::new();
            codegen_expr(tru, &mut trustate);
            let mut flsstate = CodegenState::new();
            codegen_expr(fls, &mut flsstate);
            let true_len = trustate.code.len();
            let false_len = flsstate.code.len();

            // conditional parts
            codegen_expr(cond, state);
            state.code.push(Inst::Float(1.0));
            state.code.push(Inst::EqFloat);
            state.code.push(Inst::JumpIfZero(true_len + 2)); //  true clause + Jump + 1

            // true clause
            state.append(&mut trustate);
            state.code.push(Inst::Jump(false_len + 1)); //  false clause + 1

            // false clause
            state.append(&mut flsstate);
        }
    }
}

fn codegen_main(body: &[Body], state: &mut CodegenState) {
    for b in body.iter() {
        match b {
            Body::Assignment(sym, expr) => match sym {
                Symbol::State(Name(name)) => {
                    if let Some(name) = get_vm_name(name) {
                        codegen_expr(expr, state);
                        state.code.push(Inst::Set(name))
                    } else {
                        panic!("unknown VM state name: {}", name);
                    }
                }
                Symbol::Var(_) => todo! {"treat lexical variables"},
            },
            _ => todo!("not supported yet!"),
        }
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

    state.code
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
                assert_eq!(actual.len(), expected.len());
                let eq = actual.iter().zip(expected.clone()).all(|(a, b)| *a == b);
                println!("actual = {:?}\nexpected = {:?}", actual, expected);
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
            vec![Inst::Float(1.0), Inst::Set("PosX".to_string()), Inst::Term],
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
                Inst::Set("PosY".to_string()),
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
                Inst::Set("PosY".to_string()),
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
                Inst::Get("PosX".to_string()),
                Inst::Get("InputSlow".to_string()),
                Inst::JumpIfZero(2),
                Inst::Float(4.0),
                Inst::Jump(2),
                Inst::Float(7.0),
                Inst::Add,
                Inst::Set("PosX".to_string()),
                Inst::Term,
            ],
            r##"
            proc main() {
              $px = $px + if $input_slow { 4.0 } else { 7.0 }
            }
            "##,
        );
    }
}
