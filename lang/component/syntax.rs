use crate::bullet::{BulletId, StateId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Float,
    String,
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
pub struct Signature {
    pub args: Vec<Arg>,
    pub ret: Option<Type>,
}

impl Signature {
    pub fn new(args: Vec<Arg>, ret: Option<Type>) -> Self {
        Self { args, ret }
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self {
            args: vec![],
            ret: None,
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
    // precedence level 4
    LogOr,
    LogAnd,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Var(Name),              // foo
    Ref(BulletId, StateId), // player.x
}

#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    // `let x = ...`
    //   is available in `proc`, `bullet` and `stage`
    //   will be removed at the end of `proc`, `bullet` and `stage`
    LexicalDefine(Symbol, Expr),
    // `local x = ...`
    //   is available in `bullet`
    //   is shared until the bullet died
    LocalDefine(Symbol, Expr),
    // `global x = ...`
    //   is available in a `proc`, `bullet`
    //   is shared in the stage, over any bullets
    GlobalDefine(Symbol, Expr),
    Assignment(Symbol, Expr),
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
    ProcCall(Name, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxTree {
    DefProc(Name, Signature, Vec<Body>),
    DefBullet(Name, Signature, Vec<Body>),
    DefStage(Name, Signature, Vec<Body>),
}
