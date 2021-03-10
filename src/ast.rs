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
    ProtoExpr(Vec<String>),
    FuncExpr {
        ident: Box<Node>,
        proto: Box<Node>,
        body: Box<Node>,
    },
    CallExpr {
        ident: Box<Node>,
        args: Vec<Node>,
    },
    IfExpr {
        cond: Box<Node>,
        then: Box<Node>,
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
