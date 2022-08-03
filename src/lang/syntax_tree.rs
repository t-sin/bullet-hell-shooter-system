pub enum Type {
    Float,
    String,
}

pub struct Name(pub String);

pub struct Arg {
    pub name: Name,
    pub r#type: Type,
}

pub enum Op2 {
    Add,
    Sub,
    Mul,
    Div,
}

pub enum Expr {
    Float(f32),
    Op2(Op2, Box<Expr>, Box<Expr>),
}

pub enum AssignTarget {
    VarName(Name),   // foo
    StateName(Name), // $pos_x
}

pub enum Body {
    LexicalAssign(AssignTarget, Expr),
    FnCall(Name, Vec<Expr>),
    If(Expr, Vec<Body>, Vec<Body>),
    Return(Expr),
}

pub enum SyntaxTree {
    GlobalAssign(AssignTarget, Expr),
    Defun(Name, Vec<Arg>, Vec<Body>),
}
