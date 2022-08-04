use super::{
    syntax_tree::{Body, Expr, Name, Symbol, SyntaxTree},
    vm::Inst,
};

struct CodegenState {
    code: Vec<Inst>,
}

fn codegen_expr(expr: &Expr, state: &mut CodegenState) {
    match expr {
        Expr::Float(f) => state.code.push(Inst::Float(*f)),
        Expr::String(s) => todo!("treat strings"),
        Expr::Symbol(sym) => match sym {
            Symbol::State(Name(name)) => state.code.push(Inst::Get(name.to_string())),
            Symbol::Var(_) => todo! {"treat lexical variables"},
        },
        Expr::Op2(op, expr1, expr2) => {
            todo!("binary op!!!");
        }
        Expr::If(cond, tru, fls) => {
            todo!("binary if!!!");
        }
    }
}

fn codegen_main(body: &[Body], state: &mut CodegenState) {
    for b in body.iter() {
        match b {
            Body::Assignment(sym, expr) => match sym {
                Symbol::State(Name(name)) => {
                    codegen_expr(expr, state);
                    state.code.push(Inst::Set(name.to_string()));
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
    let mut state = CodegenState { code: Vec::new() };

    codegen_syntax_trees(source, &mut state);

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
            vec![Inst::Float(1.0), Inst::Set("PosX".to_string())],
            r##"
            proc main() {
              $px = 1.0
            }
            "##,
        )
    }
}
