use std::string::String;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    NumberExpr(f64),
    IdentExpr(String),
    BoolExpr(bool),
    BlockExpr(Vec<Node>),
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
        ident: Box<Node>,
        expr: Box<Node>,
    },
    AssignExpr {
        ident: Box<Node>,
        expr: Box<Node>,
    },
    FuncExpr {
        ident: Box<Node>,
        args: Vec<String>,
        body: Box<Node>,
    },
    CallExpr {
        ident: Box<Node>,
        args: Vec<Node>,
    },
    CondExpr {
        cond: Box<Node>,
        cons: Box<Node>,
        alter: Option<Box<Node>>,
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
