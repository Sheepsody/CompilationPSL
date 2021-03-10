#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Expr,
    NumberExpr {
        value: f64,
    },
    UnaryExpr {
        op: Op,
        child: Box<Node>,
    },
    BinaryExpr {
        op: Op,
        lhs: Box<Node>,
        rhs: Box<Node>,
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
