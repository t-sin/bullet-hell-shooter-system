#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Float,
    // String,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub name: Name,
    pub r#type: Type,
}

impl Arg {
    pub fn new(name: String, r#type: Type) -> Self {
        Self {
            name: Name(name),
            r#type,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op2 {
    // precedence level 1
    Mul,
    Div,
    Mod,
    // precedence level 2
    Add,
    Sub,
    // precedence level 3
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Var(Name),   // foo
    State(Name), // $pos_x
}

#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    LexicalDefine(Symbol, Expr),
    Assignment(Symbol, Expr),
    ProcCall(Name, Vec<Expr>),
    Return(Option<Expr>),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Float(f32),
    Bool(bool),
    String(String),
    Symbol(Symbol),
    Op2(Op2, Box<Expr>, Box<Expr>),
    // If(Box<Expr>, Vec<Body>, Vec<Body>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxTree {
    GlobalDefine(Symbol, Expr),
    DefProc(Name, Vec<Arg>, Option<Type>, Vec<Body>),
}
