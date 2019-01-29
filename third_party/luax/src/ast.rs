use std::prelude::v1::*;
use std::collections::HashSet;
use serde_json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Stmt {
    Do(Vec<Stmt>),
    Set(Vec<Lhs>, Vec<Expr>),
    While(Expr, Block),
    Repeat(Block, Expr),
    If(Vec<(Expr, Block)>, Option<Block>),
    Fornum(Lhs, Expr, Expr, Option<Expr>, Block),
    Forin(Vec<Lhs>, Vec<Expr>, Block),
    Local(Vec<Lhs>, Vec<Expr>),
    Localrec(Lhs, Expr),
    Goto(String),
    Label(String),
    Return(Vec<Expr>),
    Break,
    Call(Expr, Vec<Expr>)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Block {
    Block(Vec<Stmt>)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expr {
    Nil,
    Dots,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Vec<Lhs>, Block),
    Table(Vec<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Idiv(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Concat(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Unm(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Pair(Box<Expr>, Box<Expr>),
    Id(String),
    Index(Box<Expr>, Box<Expr>)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Lhs {
    Id(String),
    Index(Expr, Expr)
}

macro_rules! concat_vec {
    ($left:expr, $right:expr) => ({
        let mut ret = $left;
        ret.extend($right);
        ret
    })
}

macro_rules! pair_get_used_vars {
    ($left:expr, $right:expr) => ({
        concat_vec!($left.get_used_vars(), $right.get_used_vars())
    })
}

macro_rules! pair_get_closure_escaped_vars {
    ($left:expr, $right:expr) => ({
        concat_vec!($left.get_closure_escaped_vars(), $right.get_closure_escaped_vars())
    })
}

impl Block {
    pub fn from_json<T: AsRef<str>>(json: T) -> Result<Block, serde_json::Error> {
        serde_json::from_str(json.as_ref())
    }
}

impl Lhs {
    pub fn id(&self) -> Option<&str> {
        match *self {
            Lhs::Id(ref s) => Some(s.as_str()),
            _ => None
        }
    }
}

pub trait GetEscapeInfo {
    fn get_used_vars(&self) -> Vec<String>;
    fn get_closure_escaped_vars(&self) -> Vec<String>;
}

impl Block {
    pub fn statements(&self) -> &Vec<Stmt> {
        match *self {
            Block::Block(ref v) => v
        }
    }
}

impl GetEscapeInfo for Block {
    fn get_used_vars(&self) -> Vec<String> {
        let mut locals: HashSet<String> = HashSet::new();
        let mut result: Vec<String> = Vec::new();

        for stmt in self.statements() {
            let new_results: Vec<String> = stmt.get_used_vars().into_iter()
                .filter(|x| !locals.contains(x))
                .collect();
            result.extend(
                new_results
            );
            if let Stmt::Local(ref lhs, _) = *stmt {
                for v in lhs {
                    if let Some(k) = v.id() {
                        locals.insert(k.to_string());
                    }
                }
            }
        }

        result
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        self.statements().get_closure_escaped_vars()
    }
}

impl GetEscapeInfo for Lhs {
    fn get_used_vars(&self) -> Vec<String> {
        match *self {
            Lhs::Id(ref v) => vec! [ v.clone() ],
            Lhs::Index(ref left, ref right) => {
                let mut ret = left.get_used_vars();
                ret.extend(right.get_used_vars());
                ret
            }
        }
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        Vec::new()
    }
}

impl GetEscapeInfo for Vec<Lhs> {
    fn get_used_vars(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        for v in self.iter() {
            ret.extend(v.get_used_vars());
        }
        ret
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        Vec::new()
    }
}

impl GetEscapeInfo for Stmt {
    fn get_used_vars(&self) -> Vec<String> {
        match *self {
            Stmt::Do(ref v) => v.get_used_vars(),
            Stmt::Set(ref l, ref r) => pair_get_used_vars!(l, r),
            Stmt::While(ref l, ref r) => pair_get_used_vars!(l, r),
            Stmt::Repeat(ref l, ref r) => pair_get_used_vars!(l, r),
            Stmt::If(ref l, ref r) => pair_get_used_vars!(l, r),
            Stmt::Fornum(ref a, ref b, ref c, ref d, ref e) => {
                concat_vec!(
                    pair_get_used_vars!(a, b),
                    concat_vec!(
                        pair_get_used_vars!(c, d),
                        e.get_used_vars()
                    )
                )
            },
            Stmt::Forin(ref a, ref b, ref c) => {
                concat_vec!(
                    pair_get_used_vars!(a, b),
                    c.get_used_vars()
                )
            },
            Stmt::Local(ref l, ref r) => r.get_used_vars(),
            Stmt::Localrec(ref l, ref r) => pair_get_used_vars!(l, r),
            Stmt::Goto(_) | Stmt::Label(_) | Stmt::Break => Vec::new(),
            Stmt::Return(ref v) => v.get_used_vars(),
            Stmt::Call(ref l, ref r) => pair_get_used_vars!(l, r)
        }
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        match *self {
            Stmt::Do(ref v) => v.get_closure_escaped_vars(),
            Stmt::Set(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Stmt::While(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Stmt::Repeat(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Stmt::If(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Stmt::Fornum(ref a, ref b, ref c, ref d, ref e) => {
                concat_vec!(
                    pair_get_closure_escaped_vars!(a, b),
                    concat_vec!(
                        pair_get_closure_escaped_vars!(c, d),
                        e.get_closure_escaped_vars()
                    )
                )
            },
            Stmt::Forin(ref a, ref b, ref c) => {
                concat_vec!(
                    pair_get_closure_escaped_vars!(a, b),
                    c.get_closure_escaped_vars()
                )
            },
            Stmt::Local(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Stmt::Localrec(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Stmt::Goto(_) | Stmt::Label(_) | Stmt::Break => Vec::new(),
            Stmt::Return(ref v) => v.get_closure_escaped_vars(),
            Stmt::Call(ref l, ref r) => pair_get_closure_escaped_vars!(l, r)
        }
    }
}

impl GetEscapeInfo for Vec<Stmt> {
    fn get_used_vars(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        for v in self.iter() {
            ret.extend(v.get_used_vars());
        }
        ret
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        for v in self.iter() {
            ret.extend(v.get_closure_escaped_vars());
        }
        ret
    }
}

impl GetEscapeInfo for Expr {
    fn get_used_vars(&self) -> Vec<String> {
        match *self {
            Expr::Nil | Expr::Dots | Expr::Boolean(_) | Expr::Number(_) | Expr::String(_) => Vec::new(),
            Expr::Function(ref l, ref r) => {
                let mut args: HashSet<String> = l.iter()
                    .map(|v| v.id().unwrap().to_string())
                    .collect();
                let mut result: Vec<String> = r.get_used_vars()
                    .into_iter()
                    .filter(|v| !args.contains(v)).collect();
                result
            },
            Expr::Table(ref t) => t.get_used_vars(),
            Expr::Add(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Sub(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Mul(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Div(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Idiv(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Mod(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Pow(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Concat(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Eq(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Ne(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Lt(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Gt(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Le(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Ge(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Not(ref v) => v.get_used_vars(),
            Expr::Unm(ref v) => v.get_used_vars(),
            Expr::And(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Or(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Call(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Pair(ref l, ref r) => pair_get_used_vars!(l, r),
            Expr::Id(ref v) => vec! [ v.clone() ],
            Expr::Index(ref l, ref r) => pair_get_used_vars!(l, r),
        }
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        match *self {
            Expr::Nil | Expr::Dots | Expr::Boolean(_) | Expr::Number(_) | Expr::String(_) => Vec::new(),
            Expr::Function(_, _) => self.get_used_vars(),
            Expr::Table(ref t) => t.get_closure_escaped_vars(),
            Expr::Add(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Sub(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Mul(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Div(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Idiv(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Mod(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Pow(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Concat(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Eq(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Ne(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Lt(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Gt(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Le(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Ge(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Not(ref v) => v.get_closure_escaped_vars(),
            Expr::Unm(ref v) => v.get_closure_escaped_vars(),
            Expr::And(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Or(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Call(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Pair(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
            Expr::Id(ref v) => vec! [  ],
            Expr::Index(ref l, ref r) => pair_get_closure_escaped_vars!(l, r),
        }
    }
}

impl GetEscapeInfo for Vec<Expr> {
    fn get_used_vars(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        for v in self.iter() {
            ret.extend(v.get_used_vars());
        }
        ret
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        for v in self.iter() {
            ret.extend(v.get_closure_escaped_vars());
        }
        ret
    }
}

impl<L, R> GetEscapeInfo for Vec<(L, R)> where L: GetEscapeInfo, R: GetEscapeInfo {
    fn get_used_vars(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        for &(ref l, ref r) in self.iter() {
            ret.extend(l.get_used_vars());
            ret.extend(r.get_used_vars());
        }
        ret
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        for &(ref l, ref r) in self.iter() {
            ret.extend(l.get_closure_escaped_vars());
            ret.extend(r.get_closure_escaped_vars());
        }
        ret
    }
}

impl<T> GetEscapeInfo for Option<T> where T: GetEscapeInfo {
    fn get_used_vars(&self) -> Vec<String> {
        match *self {
            Some(ref v) => v.get_used_vars(),
            None => Vec::new()
        }
    }

    fn get_closure_escaped_vars(&self) -> Vec<String> {
        match *self {
            Some(ref v) => v.get_closure_escaped_vars(),
            None => Vec::new()
        }
    }
}
