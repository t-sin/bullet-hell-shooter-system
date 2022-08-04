#[derive(Debug, PartialEq)]
pub enum Type {
    Float,
    String,
}

#[derive(Debug, PartialEq)]
pub struct Name(pub String);

#[derive(Debug, PartialEq)]
pub struct Arg {
    pub name: Name,
    pub r#type: Type,
}

#[derive(Debug, PartialEq)]
pub enum Op2 {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Var(Name),   // foo
    State(Name), // $pos_x
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Float(f32),
    String(String),
    Symbol(Symbol),
    Op2(Op2, Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Body {
    LexicalAssign(Symbol, Expr),
    FnCall(Name, Vec<Expr>),
    If(Expr, Vec<Body>, Vec<Body>),
    Return(Expr),
}

#[derive(Debug, PartialEq)]
pub enum SyntaxTree {
    GlobalAssign(Symbol, Expr),
    Defun(Name, Vec<Arg>, Vec<Body>),
}