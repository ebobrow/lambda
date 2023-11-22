use std::collections::HashMap;

use anyhow::{anyhow, bail, Ok};

use crate::scanner::Token;

#[derive(Debug)]
pub enum Value {
    Var(String),
    Constant(Constant),
    Fun { x: String, c: Box<Computation> },
    Handler(Box<Handler>),
}

#[derive(Debug)]
pub struct Handler {
    /// optional tuple (var, body)
    ret: Option<(String, Computation)>,

    /// map from op name to (var, Continuation, body)
    ops: HashMap<String, (String, String, Computation)>,
}

#[derive(Debug)]
pub enum Computation {
    /// return v
    Return(Value),

    /// op(v;y.c)
    // OpCall {
    //     name: String,
    //     param: Value,
    //     ret_name: String,
    //     continuation: Box<Computation>,
    // },

    /// op ()
    OpCall { op: String, param: Option<Value> },

    /// let x = c1 in c2
    Seq {
        x: String,
        c1: Box<Computation>,
        c2: Box<Computation>,
    },

    /// if v then c1 else c2
    If {
        v: Value,
        c1: Box<Computation>,
        c2: Box<Computation>,
    },

    /// v1 v2
    App { v1: Value, v2: Value },

    /// with v handle c
    Handling {
        with: Value,
        handle: Box<Computation>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    True,
    False,
}

// impl ToString for Expr {
//     fn to_string(&self) -> String {
//         let maybe_parenthesize = |e: &Expr| {
//             if matches!(*e, Expr::Var(_) | Expr::Constant(_)) {
//                 e.to_string()
//             } else {
//                 format!("({})", e.to_string())
//             }
//         };

//         match self {
//             Expr::Var(x) => x.to_string(),
//             Expr::Constant(c) => c.to_string(),
//             Expr::App { e1, e2 } => {
//                 format!("{} {}", maybe_parenthesize(e1), maybe_parenthesize(e2))
//             }
//             Expr::Abs { x, t, e } => format!("\\{}: {}.{}", x, t.to_string(), e.to_string()),
//             Expr::If { e1, e2, e3 } => format!(
//                 "if {} then {} else {}",
//                 e1.to_string(),
//                 e2.to_string(),
//                 e3.to_string()
//             ),
//         }
//     }
// }

// impl ToString for Constant {
//     fn to_string(&self) -> String {
//         match self {
//             Constant::True => "true".to_string(),
//             Constant::False => "false".to_string(),
//         }
//     }
// }

pub struct Parser {
    stream: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn parse(stream: Vec<Token>) -> anyhow::Result<Computation> {
        let mut parser = Self { stream, pos: 0 };
        parser.computation()
    }

    fn computation(&mut self) -> anyhow::Result<Computation> {
        match self.peek().ok_or_else(|| anyhow!("empty stream"))? {
            Token::Return => self.ret(),
            Token::Identifier(_) => {
                // let e1 = self.var()?;
                // if recurse_app {
                //     self.maybe_app(e1)
                // } else {
                //     Ok(e1)
                // }
                self.op()
            }
            Token::Let => self.r#let(),
            Token::If => self.if_then_else(),
            Token::With => self.with(),
            Token::LeftParen => {
                self.consume(&Token::LeftParen)?;
                let e = self.computation()?;
                self.consume(&Token::RightParen)?;
                Ok(e)
                // if recurse_app {
                //     self.maybe_app(e)
                // } else {
                //     Ok(e)
                // }
            }

            Token::Then
            | Token::Else
            | Token::Equal
            | Token::Dot
            | Token::Arrow
            | Token::Lambda
            | Token::LeftCurly
            | Token::RightCurly
            | Token::RightParen
            | Token::Comma
            | Token::Semicolon
            | Token::True
            | Token::False
            | Token::Handler
            | Token::In
            | Token::Handle => {
                let v1 = self.value().unwrap()?;
                let v2 = self.value().unwrap()?;
                Ok(Computation::App { v1, v2 })
                // if let Some(v1) = self.value() {
                //     if let Some(v2) = self.value() {
                //         return Ok(Computation::App { v1: v1?, v2: v2? });
                //     }
                // }
                // bail!("unexpected token {:?}", t)
            }
        }
    }

    fn ret(&mut self) -> anyhow::Result<Computation> {
        self.consume(&Token::Return)?;
        let v = self.value().unwrap_or_else(|| bail!("expected value"))?;
        Ok(Computation::Return(v))
    }

    fn r#let(&mut self) -> anyhow::Result<Computation> {
        self.consume(&Token::Let)?;
        let x = self.consume_ident()?;
        self.consume(&Token::Equal)?;
        let c1 = self.computation()?;
        self.consume(&Token::In)?;
        let c2 = self.computation()?;
        Ok(Computation::Seq {
            x,
            c1: Box::new(c1),
            c2: Box::new(c2),
        })
    }

    fn with(&mut self) -> anyhow::Result<Computation> {
        self.consume(&Token::With)?;
        let v = self.value().unwrap_or_else(|| bail!("expected value"))?;
        self.consume(&Token::Handle)?;
        let c = self.computation()?;
        Ok(Computation::Handling {
            with: v,
            handle: Box::new(c),
        })
    }

    fn op(&mut self) -> anyhow::Result<Computation> {
        let op = self.consume_ident()?;
        if let Some(Token::LeftParen) = self.peek() {
            self.consume(&Token::LeftParen)?;
            self.consume(&Token::RightParen)?;
            Ok(Computation::OpCall { op, param: None })
        } else if let Some(v) = self.value() {
            Ok(Computation::OpCall {
                op,
                param: Some(v?),
            })
        } else {
            bail!("invalid parameter to {op}")
        }
    }

    fn if_then_else(&mut self) -> anyhow::Result<Computation> {
        self.consume(&Token::If)?;
        let v = self.value().unwrap_or_else(|| bail!("expected value"))?;
        self.consume(&Token::Then)?;
        let c1 = self.computation()?;
        self.consume(&Token::Else)?;
        let c2 = self.computation()?;
        Ok(Computation::If {
            v,
            c1: Box::new(c1),
            c2: Box::new(c2),
        })
    }

    fn value(&mut self) -> Option<anyhow::Result<Value>> {
        match self.peek()? {
            Token::Identifier(_) => Some(self.var()),
            Token::True => {
                let _ = self.consume(&Token::True);
                Some(Ok(Value::Constant(Constant::True)))
            }
            Token::False => {
                let _ = self.consume(&Token::False);
                Some(Ok(Value::Constant(Constant::False)))
            }
            Token::Lambda => Some(self.fun()),
            Token::Handler => Some(self.handler()),

            Token::Equal
            | Token::Dot
            | Token::Comma
            | Token::LeftParen
            | Token::RightParen
            | Token::LeftCurly
            | Token::RightCurly
            | Token::Semicolon
            | Token::If
            | Token::Then
            | Token::Else
            | Token::Arrow
            | Token::Return
            | Token::Let
            | Token::In
            | Token::With
            | Token::Handle => None,
        }
    }

    fn var(&mut self) -> anyhow::Result<Value> {
        let ident = self.consume_ident()?;
        Ok(Value::Var(ident))
    }

    fn handler(&mut self) -> anyhow::Result<Value> {
        self.consume(&Token::Handler)?;
        self.consume(&Token::LeftCurly)?;

        let ret = if self.consume(&Token::Return).is_ok() {
            let x = self.consume_ident()?;
            self.consume(&Token::Arrow)?;
            let c = self.computation()?;
            Some((x, c))
        } else {
            None
        };

        let mut ops = HashMap::new();
        loop {
            if let Some(Token::RightCurly) = self.peek() {
                break;
            }

            let op = self.consume_ident()?;
            self.consume(&Token::LeftParen)?;
            let x = self.consume_ident()?;
            self.consume(&Token::Semicolon)?;
            let k = self.consume_ident()?;
            self.consume(&Token::RightParen)?;
            self.consume(&Token::Arrow)?;
            let c = self.computation()?;
            ops.insert(op, (x, k, c));

            if self.consume(&Token::Comma).is_ok() {
                continue;
            }
            break;
        }
        self.consume(&Token::RightCurly)?;

        Ok(Value::Handler(Box::new(Handler { ret, ops })))
    }

    fn fun(&mut self) -> anyhow::Result<Value> {
        self.consume(&Token::Lambda)?;
        let x = self.var()?;
        self.consume(&Token::Dot)?;
        let c = self.computation()?;
        if let Value::Var(x) = x {
            Ok(Value::Fun { x, c: Box::new(c) })
        } else {
            unreachable!()
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        if self.pos < self.stream.len() {
            Some(&self.stream[self.pos])
        } else {
            None
        }
    }

    fn consume(&mut self, tok: &Token) -> anyhow::Result<Option<&Token>> {
        let peek = self.peek();
        if peek == Some(tok) {
            self.pos += 1;
            Ok(self.peek())
        } else {
            bail!("expected {:?}, found {:?}", tok, peek)
        }
    }

    fn consume_ident(&mut self) -> anyhow::Result<String> {
        let peek = self.peek();
        if let Some(Token::Identifier(ident)) = peek {
            let ident = ident.clone();
            self.pos += 1;
            Ok(ident)
        } else {
            bail!("exptected identifier, found {:?}", peek)
        }
    }
}

// source: just trust me bro
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn works() {
//         let stream = vec![
//             Token::LeftParen,
//             Token::Lambda,
//             Token::Identifier("x".into()),
//             Token::Colon,
//             Token::Bool,
//             Token::Dot,
//             Token::Identifier("x".into()),
//             Token::RightParen,
//             Token::True,
//         ];
//         let e = Parser::parse(stream).unwrap();
//         assert_eq!(
//             e,
//             Expr::App {
//                 e1: Box::new(Expr::Abs {
//                     x: "x".into(),
//                     t: Type::Bool,
//                     e: Box::new(Expr::Var("x".into()))
//                 }),
//                 e2: Box::new(Expr::Constant(Constant::True))
//             }
//         );
//     }
// }
