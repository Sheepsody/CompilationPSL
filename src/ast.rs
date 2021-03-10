use std::string::String;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    NumberExpr(f64),
    IdentExpr(String),
    UnaryExpr {
        op: Op,
        child: Box<Node>,
    },
    BinaryExpr {
        op: Op,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    InitExpr {
        ident: String,
        expr: Box<Node>,
    },
    AssignExpr {
        ident: String,
        expr: Box<Node>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Op {
    Sub,
    Add,
    Mul,
    Div,
    Pow,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}
