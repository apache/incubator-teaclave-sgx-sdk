#![allow(unused_variables)]

use std::prelude::v1::*;
use ::{Error, Result};
use ast::*;
use scanner::{Scanner, Token};
use std::io::{BufReader, Read};
use std::mem;

static BINARY_PRIORITY: &'static [(u8, u8)/*(left, right)*/; 15] = &[
    (6, 6), (6, 6), (7, 7), (7, 7), (7, 7), // `+' `-' `*' `/' `%'
    (10, 9), (5, 4), // ^, .. (right associative)
    (3, 3), (3, 3), (3, 3), // ==, <, <=
    (3, 3), (3, 3), (3, 3), // ~=, >, >=
    (2, 2), (1, 1), // and, or
];

const UNARY_PRIORITY: u8 = 8;


pub fn unary_op(t: &Token) -> UnaryOpr {
    match *t {
        Token::Not => UnaryOpr::Not,
        Token::Char(c) if c == '-' => UnaryOpr::Minus,
        Token::Char(c) if c == '#' => UnaryOpr::Length,
        _ => UnaryOpr::NoUnary
    }
}

pub fn binary_op(t: &Token) -> BinaryOpr {
    match *t {
        Token::Char(c) if c == '+' => BinaryOpr::Add,
        Token::Char(c) if c == '-' => BinaryOpr::Sub,
        Token::Char(c) if c == '*' => BinaryOpr::Mul,
        Token::Char(c) if c == '/' => BinaryOpr::Div,
        Token::Char(c) if c == '%' => BinaryOpr::Mod,
        Token::Char(c) if c == '^' => BinaryOpr::Pow,
        Token::Concat => BinaryOpr::Concat,
        Token::NE => BinaryOpr::NE,
        Token::Eq => BinaryOpr::Eq,
        Token::Char(c) if c == '<' => BinaryOpr::LT,
        Token::LE => BinaryOpr::LE,
        Token::Char(c) if c == '>' => BinaryOpr::GT,
        Token::GE => BinaryOpr::GE,
        Token::And => BinaryOpr::And,
        Token::Or => BinaryOpr::Or,
        _ => BinaryOpr::NoBinary,
    }
}

#[derive(Debug)]
pub struct Parser<R> {
    source: String,
    scanner: Scanner<R>,
    prev_number: u32,
    line_number: u32,
    token: Token,
    ahead_token: Token,
}

impl<R: Read> Parser<R> {
    pub fn new(reader: BufReader<R>, name: String) -> Parser<R> {
        Parser {
            source: name,
            scanner: Scanner::new(reader),
            prev_number: 1,
            line_number: 1,
            token: Token::EOF,
            ahead_token: Token::EOF,
        }
    }


    /// Here is the complete syntax of Lua in extended BNF. (It does not describe operator precedences.)
    ///
    /// ```BNF
    ///	chunk ::= {stat [`;´]} [laststat [`;´]]
    ///
    ///	block ::= chunk
    ///
    ///	stat ::=  varlist `=´ explist |
    ///		 functioncall |
    ///		 do block end |
    ///		 while exp do block end |
    ///		 repeat block until exp |
    ///		 if exp then block {elseif exp then block} [else block] end |
    ///		 for Name `=´ exp `,´ exp [`,´ exp] do block end |
    ///		 for namelist in explist do block end |
    ///		 function funcname funcbody |
    ///		 local function Name funcbody |
    ///		 local namelist [`=´ explist]
    ///
    ///	laststat ::= return [explist] | break
    ///
    ///	funcname ::= Name {`.´ Name} [`:´ Name]
    ///
    ///	varlist ::= var {`,´ var}
    ///
    ///	var ::=  Name | prefixexp `[´ exp `]´ | prefixexp `.´ Name
    ///
    ///	namelist ::= Name {`,´ Name}
    ///
    ///	explist ::= {exp `,´} exp
    ///
    ///	exp ::=  nil | false | true | Number | String | `...´ | function |
    ///		 prefixexp | tableconstructor | exp binop exp | unop exp
    ///
    ///	prefixexp ::= var | functioncall | `(´ exp `)´
    ///
    ///	functioncall ::=  prefixexp args | prefixexp `:´ Name args
    ///
    ///	args ::=  `(´ [explist] `)´ | tableconstructor | String
    ///
    ///	function ::= function funcbody
    ///
    ///	funcbody ::= `(´ [parlist] `)´ block end
    ///
    ///	parlist ::= namelist [`,´ `...´] | `...´
    ///
    ///	tableconstructor ::= `{´ [fieldlist] `}´
    ///
    ///	fieldlist ::= field {fieldsep field} [fieldsep]
    ///
    ///	field ::= `[´ exp `]´ `=´ exp | Name `=´ exp | exp
    ///
    ///	fieldsep ::= `,´ | `;´
    ///
    ///	binop ::= `+´ | `-´ | `*´ | `/´ | `^´ | `%´ | `..´ |
    ///		 `<´ | `<=´ | `>´ | `>=´ | `==´ | `~=´ |
    ///		 and | or
    ///
    ///	unop ::= `-´ | not | `#´
    /// ```
    pub fn parse(&mut self) -> Result<Vec<StmtNode>> {
        self.next()?;
        let stmts = self.block()?;
        self.check(Token::EOF)?;
        Ok(stmts)
    }

    fn next(&mut self) -> Result<()> {
        self.prev_number = self.line_number;
        self.token = match self.ahead_token {
            Token::EOF => self.scanner.scan()?,
            _ => {
                let mut ahead = Token::EOF;
                mem::swap(&mut ahead, &mut self.ahead_token);
                ahead
            }
        };
        self.line_number = self.scanner.line_number();
        //println!("<{}>: => {:?}, {:?}", self.line_number, self.token, self.ahead_token);
        Ok(())
    }

    fn look_ahead(&mut self) -> Result<()> {
        debug_assert!(self.ahead_token == Token::EOF);
        self.ahead_token = self.scanner.scan()?;
        Ok(())
    }

    fn block_follow(&self) -> bool {
        match self.token {
            Token::Else | Token::Elseif | Token::End | Token::Until | Token::EOF => true,
            _ => false,
        }
    }

    fn block(&mut self) -> Result<Vec<StmtNode>> {
        let mut stmts: Vec<StmtNode> = vec![];
        loop {
            if self.block_follow() {
                break;
            }
            stmts.push(self.statement()?);
            self.testnext(&Token::Char(';'))?;
        }
        return Ok(stmts);
    }

    fn testnext(&mut self, t: &Token) -> Result<bool> {
        if self.token == *t {
            self.next()?;
            return Ok(true);
        }
        Ok(false)
    }

    fn statement(&mut self) -> Result<StmtNode> {
        loop {
            match self.token {
                Token::Char(char) if char == ';' => self.next()?,
                _ => break
            }
        }
        let line = self.line_number;
        // is not last statement
        let stmt = match self.token {
            Token::If => self.ifstat()?,
            Token::While => self.whilestat()?,
            Token::Do => {
                self.next()?;
                let stmts = self.block()?;
                self.check_match(Token::End, Token::Do, line)?;
                Stmt::DoBlock(stmts)
            }
            Token::For => self.forstat()?,
            Token::Repeat => self.repeatstat()?,
            Token::Function => self.funcstat()?,
            Token::Local => {
                self.next()?;
                if self.testnext(&Token::Function)? {
                    self.localfunc()?
                } else {
                    self.localstat()?
                }
            }
            Token::Return => self.retstat()?,
            Token::Break => self.breakstat()?,
            _ => self.exprstat()?
        };
        Ok(StmtNode::new(stmt, (line, self.prev_number)))
    }

    fn field(&mut self) -> Result<Field> {
        let field = match self.token {
            Token::Char('[') => {
                self.next()?;
                let key = self.expression()?;
                self.check_next(Token::Char(']'))?;
                self.check_next(Token::Char('='))?;
                let val = self.expression()?;
                Field::new(Some(key), val)
            }
            Token::Ident(_) => {
                self.look_ahead()?;
                let key = if self.ahead_token == Token::Char('=') {
                    let s = if let Token::Ident(ref s) = self.token { s.clone() } else { unreachable!() };
                    self.next()?;
                    ExprNode::new(Expr::String(s.to_string()), (self.prev_number, self.prev_number))
                } else {
                    self.expression()?
                };
                if self.testnext(&Token::Char('='))? {
                    let val = self.expression()?;
                    Field::new(Some(key), val)
                } else {
                    Field::new(None, key)
                }
            }
            _ => {
                let val = self.expression()?;
                Field::new(None, val)
            }
        };
        Ok(field)
    }

    fn constructor(&mut self) -> Result<ExprNode> {
        debug_assert!(self.token == Token::Char('{'));
        let line = self.line_number;
        let mut fields: Vec<Field> = vec![];
        self.next()?;
        while self.token != Token::Char('}') {
            let field = self.field()?;
            fields.push(field);
            match self.token {
                Token::Char(',') | Token::Char(';') => self.next()?,
                _ => {}
            }
        }
        self.check_next(Token::Char('}'))?;
        let exprnode = ExprNode::new(Expr::Table(fields), (line, self.prev_number));
        Ok(exprnode)
    }

    /// ```BNF
    /// namelist :: Ident | namelist ',' Ident
    /// ```
    fn namelist(&mut self) -> Result<Vec<String>> {
        let mut names: Vec<String> = vec![];
        loop {
            match self.token {
                Token::Ident(ref name) => names.push(name.clone()),
                Token::Char(',') => return Err(self.unexpected(&self.token)),
                _ => break
            }
            self.next()?;
            if !self.testnext(&Token::Char(','))? {
                break;
            }
        }
        Ok(names)
    }

    fn exprlist(&mut self) -> Result<Vec<ExprNode>> {
        let mut exprs = vec![self.expression()?];
        while self.testnext(&Token::Char(','))? {
            let expr = self.expression()?;
            exprs.push(expr);
        }
        Ok(exprs)
    }

    fn funcbody(&mut self) -> Result<ExprNode> {
        let line = self.line_number;
        // paramater list
        self.check_next(Token::Char('('))?;
        let mut parlist = ParList::new();
        if let Token::Ident(_) = self.token {
            let names = self.namelist()?;
            parlist.set_names(names);
        }
        if Token::Dots == self.token {
            parlist.set_vargs(true);
            self.next()?;
        }
        self.check_next(Token::Char(')'))?;

        let block = self.block()?;
        self.check_next(Token::End)?;

        let exprnode = ExprNode::new(Expr::Function(parlist, block), (line, self.prev_number));
        Ok(exprnode)
    }

    fn function(&mut self) -> Result<ExprNode> {
        debug_assert!(self.token == Token::Function);
        self.next()?;
        self.funcbody()
    }

    /// Prefix expression
    ///
    /// ```BNF
    /// prefixexp -> NAME | '(' expr ')'
    /// ```
    fn prefixexp(&mut self) -> Result<ExprNode> {
        let line = self.line_number;
        let expr = match self.token {
            Token::Ident(ref s) => {
                let expr = ExprNode::new(Expr::Ident(s.clone()), (line, line));
                expr
            }
            Token::Char('(') => {
                self.next()?;
                let line = self.line_number;
                let mut expr = self.expression()?;
                match expr.inner_mut() {
                    Expr::FuncCall(ref mut call) => call.adj = true,
                    _ => {}
                }
                self.check(Token::Char(')'))?;
                expr
            }
            _ => return Err(self.unexpected(&self.token))
        };
        self.next()?;
        Ok(expr)
    }

    fn fnargs(&mut self) -> Result<Vec<ExprNode>> {
        let mut next = false;
        let expr = match self.token {
            Token::Char('(') => {
                let line = self.line_number;
                self.next()?;
                let exprs = if self.token != Token::Char(')') {
                    self.exprlist()?
                } else {
                    // no arguments
                    vec![]
                };
                self.check_match(Token::Char(')'), Token::Char('('), line)?;
                exprs
            }
            Token::Char('{') => vec![self.constructor()?],
            Token::String(ref s) => {
                let line = self.prev_number;
                next = true;
                let mut expr = ExprNode::new(Expr::String(s.clone()), (line, line));
                vec![expr]
            }
            _ => return Err(Error::SyntaxError("function arguments expected".to_string()))
        };
        if next {
            self.next()?;
        }
        Ok(expr)
    }

    /// Primary expression
    ///
    /// ```BNF
    /// primaryexp ->
    /// prefixexp { '.' NAME | '[' exp `]' | ':' NAME funcargs | funcargs }
    /// ```
    fn primaryexp(&mut self) -> Result<ExprNode> {
        let line = self.line_number;
        let mut exprnode = self.prefixexp()?;
        loop {
            let expr = match self.token {
                Token::Char('.') => {
                    self.next()?;
                    let obj = if let Token::Ident(ref s) = self.token {
                        let line = self.line_number;
                        let mut key = ExprNode::new(Expr::String(s.clone()), (line, line));
                        let mut obj = Expr::AttrGet(Box::new(exprnode), Box::new(key));
                        obj
                    } else {
                        return Err(self.unexpected(&self.token));
                    };
                    self.next()?;
                    obj
                }
                Token::Char('[') => {
                    self.next()?;
                    let mut key = self.expression()?;
                    let mut obj = Expr::AttrGet(Box::new(exprnode), Box::new(key));
                    self.check_next(Token::Char(']'))?;
                    obj
                }
                Token::Char(':') => {
                    self.next()?;
                    let method = if let Token::Ident(ref s) = self.token {
                        s.clone()
                    } else {
                        return Err(self.unexpected(&self.token));
                    };
                    self.next()?;
                    let args = self.fnargs()?;
                    let call = MethodCall::new(exprnode, method, args);
                    Expr::MethodCall(Box::new(call))
                }
                Token::Char('(') | Token::String(_) | Token::Char('{') => {
                    let args = self.fnargs()?;
                    let call = FuncCall::new(exprnode, args);
                    Expr::FuncCall(Box::new(call))
                }
                _ => {
                    return Ok(exprnode);
                }
            };
            exprnode = ExprNode::new(expr, (line, self.prev_number))
        }
    }

    /// Simple expression
    ///
    /// ```BNF
    /// simpleexp -> NUMBER | STRING | NIL | true | false | ... |
    /// constructor | FUNCTION body | primaryexp
    /// ```
    fn simple_expr(&mut self) -> Result<ExprNode> {
        let lineinfo = (self.line_number, self.line_number);
        let node = match self.token {
            Token::True => ExprNode::new(Expr::True, lineinfo),
            Token::False => ExprNode::new(Expr::False, lineinfo),
            Token::Nil => ExprNode::new(Expr::Nil, lineinfo),
            Token::Number(n) => ExprNode::new(Expr::Number(n), lineinfo),
            Token::Dots => ExprNode::new(Expr::Dots, lineinfo),
            Token::String(ref s) => ExprNode::new(Expr::String(s.clone()), (self.prev_number, self.line_number)),
            Token::Char(c) if c == '{' => return Ok(self.constructor()?),
            Token::Function => return Ok(self.function()?),
            _ => return Ok(self.primaryexp()?)
        };
        self.next()?;
        Ok(node)
    }

    fn sub_expression(&mut self, limit: u8) -> Result<(ExprNode, BinaryOpr)> {
        let op = unary_op(&self.token);
        let line = self.line_number;
        let mut expr = if op != UnaryOpr::NoUnary {
            self.next()?;
            let sub = self.sub_expression(UNARY_PRIORITY)?.0;
            let lineinfo = (line, self.prev_number);
            ExprNode::new(Expr::UnaryOp(op, Box::new(sub)), lineinfo)
        } else {
            self.simple_expr()?
        };

        let mut op = binary_op(&self.token);
        loop {
            if op == BinaryOpr::NoBinary || BINARY_PRIORITY[op as usize].1 <= limit {
                break;
            }
            self.next()?;
            let line = self.line_number;
            let sub = self.sub_expression(BINARY_PRIORITY[op as usize].0)?;
            let lineinfo = (line, self.prev_number);
            expr = ExprNode::new(Expr::BinaryOp(op, Box::new(expr), Box::new(sub.0)), lineinfo);
            op = sub.1;
        }
        Ok((expr, op))
    }

    fn expression(&mut self) -> Result<ExprNode> { Ok(self.sub_expression(0)?.0) }

    fn test_then_block(&mut self) -> Result<IfThenElse> {
        let jump_false = 0;
        self.next()?;
        let condition = self.expression()?;
        self.check_next(Token::Then)?;
        let then = self.block()?;
        Ok(IfThenElse::new(condition, then, vec![]))
    }

    /// ifstat -> IF cond THEN block {ELSEIF cond THEN block} [ELSE block] END
    fn ifstat(&mut self) -> Result<Stmt> {
        debug_assert!(self.token == Token::If);
        let line = self.line_number;
        let mut ifthen = self.test_then_block()?;
        let mut elseifs: Vec<(IfThenElse, (u32, u32))> = vec![];
        while self.token == Token::Elseif {
            let line = self.line_number;
            let elseif = self.test_then_block()?;
            elseifs.push((elseif, (line, self.prev_number)));
        }

        let mut els = if self.testnext(&Token::Else)? {
            self.block()?
        } else {
            vec![]
        };

        // link all elseif
        for mut elseif in elseifs.into_iter().rev().into_iter() {
            let mut e: Vec<Node<Stmt>> = vec![];
            mem::swap(&mut e, &mut els);
            let (mut elif, lineinfo) = elseif;
            elif.set_els(e);
            els.push(StmtNode::new(Stmt::If(elif), lineinfo));
        }
        self.check_match(Token::End, Token::If, line)?;
        ifthen.set_els(els);
        Ok(Stmt::If(ifthen))
    }

    /// ```BNF
    /// whilestat -> WHILE cond DO block END
    /// ```
    fn whilestat(&mut self) -> Result<Stmt> {
        debug_assert!(self.token == Token::While);
        let line = self.line_number;
        self.next()?; // skip WHILE
        let cond = self.expression()?;
        self.check_next(Token::Do)?;
        let stmts = self.block()?;
        self.check_match(Token::End, Token::While, line)?;
        Ok(Stmt::While(cond, stmts))
    }

    fn forbody(&mut self) -> Result<Vec<StmtNode>> {
        self.check_next(Token::Do)?;
        self.block()
    }

    fn fornum(&mut self, varname: String) -> Result<Stmt> {
        self.check_next(Token::Char('='))?;
        let init = self.expression()?;
        self.check_next(Token::Char(','))?;
        let limit = self.expression()?;
        // step optional
        let line = self.line_number;
        let step = if self.testnext(&Token::Char(','))? {
            self.expression()?
        } else {
            ExprNode::new(Expr::Number(1.0), (line, self.prev_number))
        };

        let body = self.forbody()?;
        let stmt = Stmt::NumberFor(NumberFor::new(varname, init, limit, step, body));
        Ok(stmt)
    }

    fn forlist(&mut self, indexname: String) -> Result<Stmt> {
        let mut names = vec![indexname];
        while self.testnext(&Token::Char(','))? {
            if let Token::Ident(ref s) = self.token {
                names.push(s.clone());
            } else {
                return Err(self.unexpected(&self.token));
            }
            self.next()?;
        }
        self.check_next(Token::In)?;
        let exprs = self.exprlist()?;
        let body = self.forbody()?;
        let stmt = Stmt::GenericFor(GenericFor::new(names, exprs, body));
        Ok(stmt)
    }

    /// ```BNF
    /// forstat -> FOR (fornum | forlist) END
    /// ```
    fn forstat(&mut self) -> Result<Stmt> {
        debug_assert!(self.token == Token::For);
        let line = self.line_number;
        self.next()?; // skip FOR
        let varname = if let Token::Ident(ref s) = self.token {
            s.clone()
        } else {
            return Err(Error::SyntaxError("unexpected symbol".to_string()));
        };
        self.next()?;
        let stmt = match self.token {
            Token::Char('=') => self.fornum(varname)?,
            Token::Char(',') | Token::In => self.forlist(varname)?,
            _ => return Err(Error::SyntaxError("= or in expected".to_string()))
        };
        self.check_match(Token::End, Token::For, line)?;
        Ok(stmt)
    }

    /// ```BNF
    /// repeatstat -> REPEAT block UNTIL cond
    /// ```
    fn repeatstat(&mut self) -> Result<Stmt> {
        debug_assert!(self.token == Token::Repeat);
        let line = self.line_number;
        self.next()?; // skip REPEAT
        let stmts = self.block()?;
        self.check_match(Token::Until, Token::Repeat, line)?;
        let cond = self.expression()?;
        Ok(Stmt::Repeat(cond, stmts))
    }

    fn funcstat(&mut self) -> Result<Stmt> {
        debug_assert!(self.token == Token::Function);
        let line = self.line_number;
        self.next()?; // skip FUNCTION
        // funcname
        let name = if let Token::Ident(ref s) = self.token {
            Expr::Ident(s.clone())
        } else {
            return Err(self.unexpected(&self.token));
        };
        self.next()?;
        let mut nameexpr = ExprNode::new(name, (line, self.prev_number));
        while self.testnext(&Token::Char('.'))? {
            if let Token::Ident(ref s) = self.token {
                let key = ExprNode::new(Expr::String(s.clone()), (self.prev_number, self.prev_number));
                let mut obj = Expr::AttrGet(Box::new(nameexpr), Box::new(key));
                nameexpr = ExprNode::new(obj, (line, self.prev_number));
            } else {
                return Err(self.unexpected(&self.token));
            }
            self.next()?;
        }

        let method = if self.testnext(&Token::Char(':'))? {
            let m = if let Token::Ident(ref s) = self.token {
                Some(s.clone())
            } else {
                return Err(self.unexpected(&self.token));
            };
            self.next()?;
            m
        } else {
            None
        };

        let body = self.funcbody()?;
        let stmt = match method {
            Some(m) => {
                let def = MethodDef::new(nameexpr, m, body);
                Stmt::MethodDef(def)
            }
            None => {
                let def = FuncDef::new(nameexpr, body);
                Stmt::FuncDef(def)
            }
        };
        Ok(stmt)
    }

    fn localfunc(&mut self) -> Result<Stmt> {
        let line = self.line_number;
        let name = if let Token::Ident(ref s) = self.token {
            s.clone()
        } else {
            return Err(self.unexpected(&self.token));
        };
        self.next()?;
        let body = self.funcbody()?;
        let stmt = Stmt::LocalAssign(vec![name], vec![body]);
        Ok(stmt)
    }

    /// ```BNF
    /// stat -> LOCAL NAME {`,' NAME} [`=' explist1]
    /// ```
    fn localstat(&mut self) -> Result<Stmt> {
        let namelist = self.namelist()?;
        let exprlist = if self.testnext(&Token::Char('='))? {
            self.exprlist()?
        } else {
            vec![]
        };
        let stmt = Stmt::LocalAssign(namelist, exprlist);
        Ok(stmt)
    }

    /// ```BNF
    /// stat -> func | assignment
    /// ```
    fn exprstat(&mut self) -> Result<Stmt> {
        let expr = self.primaryexp()?;
        let stmt = match expr.inner() {
            &Expr::FuncCall(_) => Stmt::FuncCall(expr),
            &Expr::MethodCall(_) => Stmt::MethodCall(expr),
            _ => {
                let mut lhs = vec![expr];
                while self.testnext(&Token::Char(','))? {
                    lhs.push(self.primaryexp()?)
                }
                self.check_next(Token::Char('='))?;
                let rhs = self.exprlist()?;
                Stmt::Assign(lhs, rhs)
            }
        };
        Ok(stmt)
    }

    fn breakstat(&mut self) -> Result<Stmt> {
        debug_assert!(self.token == Token::Break);
        self.next()?; // skip BREAK
        let stmt = Stmt::Break;
        Ok(stmt)
    }

    fn retstat(&mut self) -> Result<Stmt> {
        debug_assert!(self.token == Token::Return);
        self.next()?; // skip RETURE
        let exprlist = if self.block_follow() || self.token == Token::Char(';') {
            vec![]
        } else {
            self.exprlist()?
        };
        let stmt = Stmt::Return(exprlist);
        Ok(stmt)
    }

    fn check_next(&mut self, expect: Token) -> Result<()> {
        self.check(expect)?;
        self.next()
    }

    fn check(&self, expect: Token) -> Result<()> {
        if self.token == expect {
            Ok(())
        } else {
            let s = format!("{}:{}: {} expected", self.source, self.line_number, expect.to_string());
            Err(Error::SyntaxError(s))
        }
    }

    fn check_match(&mut self, what: Token, who: Token, line: u32) -> Result<()> {
        if !self.testnext(&what)? {
            let err = if line == self.line_number {
                format!("{}:{}: {} expected", self.source, self.line_number, what.to_string())
            } else {
                format!("{}:{}: {} expected (to close {} at line {}", self.source, self.line_number,
                        what.to_string(), who.to_string(), line)
            };
            Err(Error::SyntaxError(err))
        } else {
            Ok(())
        }
    }

    fn unexpected(&self, t: &Token) -> Error {
        let err = format!("{}:{}: unexpected: {}", self.source, self.line_number, t.to_string());
        Error::SyntaxError(err)
    }
}
