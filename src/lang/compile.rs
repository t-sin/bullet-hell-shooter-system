use super::{syntax_tree::SyntaxTree, vm::Inst};

pub fn parse(source: &str) -> Vec<SyntaxTree> {
    let mut ast = Vec::new();

    ast
}

pub fn compile(source: &str) -> Vec<Inst> {
    let mut code = Vec::new();
    let _ast = parse(source);

    code
}
