use std::prelude::v1::*;
use std::fmt::{Debug, Error, Formatter};

/// Node represents a node in abstract syntax tree
pub struct Node<T> {
    /// line info: (begin_line, end_line)
    lineinfo: (u32, u32),
    inner: T,
}

impl<T> Node<T> {
    pub fn new(inner: T, lineinfo: (u32, u32)) -> Node<T> {
        Node {
            lineinfo,
            inner,
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn lineinfo(&self) -> (u32, u32) {
        self.lineinfo
    }

    pub fn line(&self) -> u32 {
        self.lineinfo.0
    }

    pub fn last_line(&self) -> u32 {
        self.lineinfo.1
    }
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "<{}:{}> {:#?}", self.lineinfo.0, self.lineinfo.1, self.inner)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOpr {
    Add = 0,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Concat,
    Eq,
    LT,
    LE,
    NE,
    GT,
    GE,
    And,
    Or,
    NoBinary,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnaryOpr {
    Minus,
    Not,
    Length,
    NoUnary,
}

#[derive(Debug)]
pub enum Expr {
    True,
    False,
    Nil,
    Number(f64),
    String(String),
    Dots,
    Ident(String),

    /// AttrGet(Object, Key)
    AttrGet(Box<ExprNode>, Box<ExprNode>),
    Table(Vec<Field>),
    FuncCall(Box<FuncCall>),
    MethodCall(Box<MethodCall>),

    /// BinaryOp(Operator, Lhs, Rhs)
    BinaryOp(BinaryOpr, Box<ExprNode>, Box<ExprNode>),

    /// UnaryOp(Operator, expr)
    UnaryOp(UnaryOpr, Box<ExprNode>),

    /// Function(ParList, Stmts)
    Function(ParList, Vec<StmtNode>),
}

impl Expr {
    pub fn is_vararg(&self) -> bool {
        match self {
            &Expr::Dots => true,
            &Expr::FuncCall(ref call) => !call.adj,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub key: Option<ExprNode>,
    pub val: ExprNode,
}

impl Field {
    pub fn new(key: Option<ExprNode>, val: ExprNode) -> Field { Field { key, val } }
}

#[derive(Debug)]
pub struct ParList {
    pub vargs: bool,
    pub names: Vec<String>,
}

impl ParList {
    pub fn new() -> ParList {
        ParList {
            vargs: false,
            names: Vec::new(),
        }
    }

    pub fn set_vargs(&mut self, vargs: bool) {
        self.vargs = vargs;
    }

    pub fn set_names(&mut self, names: Vec<String>) {
        self.names = names;
    }

    pub fn push_param(&mut self, param: String) {
        self.names.push(param)
    }
}

#[derive(Debug)]
pub struct FuncDef {
    pub name: Vec<ExprNode>,
    pub body: Vec<ExprNode>,
}

impl FuncDef {
    pub fn new(name: ExprNode, body: ExprNode) -> Box<FuncDef> {
        // TODO: refactor this
        Box::new(FuncDef {
            name: vec![name],
            body: vec![body],
        })
    }
}

#[derive(Debug)]
pub struct MethodDef {
    pub receiver: ExprNode,
    pub method: String,
    pub body: ExprNode,
}

impl MethodDef {
    pub fn new(receiver: ExprNode, method: String, body: ExprNode) -> Box<MethodDef> {
        Box::new(MethodDef {
            receiver,
            method,
            body,
        })
    }
}

#[derive(Debug)]
pub struct MethodCall {
    pub receiver: ExprNode,
    pub method: String,
    pub args: Vec<ExprNode>,
}

impl MethodCall {
    pub fn new(receiver: ExprNode, method: String, args: Vec<ExprNode>) -> MethodCall {
        MethodCall {
            receiver,
            method,
            args,
        }
    }
}

#[derive(Debug)]
pub struct FuncCall {
    pub func: ExprNode,
    pub args: Vec<ExprNode>,
    pub adj: bool,
}

impl FuncCall {
    pub fn new(func: ExprNode, args: Vec<ExprNode>) -> FuncCall {
        FuncCall {
            func,
            args,
            adj: false,
        }
    }
}

#[derive(Debug)]
pub struct IfThenElse {
    pub condition: ExprNode,
    pub then: Vec<StmtNode>,
    pub els: Vec<StmtNode>,
}

impl IfThenElse {
    pub fn new(condition: ExprNode, then: Vec<StmtNode>, els: Vec<StmtNode>) -> IfThenElse {
        IfThenElse {
            condition,
            then,
            els,
        }
    }

    pub fn set_els(&mut self, els: Vec<StmtNode>) {
        self.els = els;
    }
}

#[derive(Debug)]
pub struct NumberFor {
    pub name: String,
    pub init: ExprNode,
    pub limit: ExprNode,
    pub step: ExprNode,
    pub stmts: Vec<StmtNode>,
}

impl NumberFor {
    pub fn new(name: String, init: ExprNode, limit: ExprNode, step: ExprNode, stmts: Vec<StmtNode>) -> Box<NumberFor> {
        Box::new(NumberFor {
            name,
            init,
            limit,
            step,
            stmts,
        })
    }
}

#[derive(Debug)]
pub struct GenericFor {
    pub names: Vec<String>,
    pub exprs: Vec<ExprNode>,
    pub stmts: Vec<StmtNode>,
}

impl GenericFor {
    pub fn new(names: Vec<String>, exprs: Vec<ExprNode>, stmts: Vec<StmtNode>) -> Box<GenericFor> {
        Box::new(GenericFor {
            names,
            exprs,
            stmts,
        })
    }
}

#[derive(Debug)]
pub enum Stmt {
    /// Assign(Lhs, Rhs)
    Assign(Vec<ExprNode>, Vec<ExprNode>),

    /// LocalAssign(Names, Exprs)
    LocalAssign(Vec<String>, Vec<ExprNode>),
    FuncCall(ExprNode),
    MethodCall(ExprNode),
    DoBlock(Vec<StmtNode>),

    /// While(Condition, Stmts)
    While(ExprNode, Vec<StmtNode>),

    /// Repeat(Condition, Stmts)
    Repeat(ExprNode, Vec<StmtNode>),
    If(IfThenElse),
    NumberFor(Box<NumberFor>),
    GenericFor(Box<GenericFor>),
    FuncDef(Box<FuncDef>),
    MethodDef(Box<MethodDef>),
    Return(Vec<ExprNode>),
    Break,
}

pub type StmtNode = Node<Stmt>;
pub type ExprNode = Node<Expr>;
