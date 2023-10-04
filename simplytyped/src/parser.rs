use anyhow::{anyhow, bail};

use crate::scanner::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    /// x
    Var(String),
    /// c
    Constant(Constant),
    /// e1 e2
    App { e1: Box<Expr>, e2: Box<Expr> },
    /// \x:t.e
    Abs { x: String, t: Type, e: Box<Expr> },
    /// if e1 then e2 else e3
    If {
        e1: Box<Expr>,
        e2: Box<Expr>,
        e3: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    True,
    False,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Bool,
    Fn(Box<Type>, Box<Type>),
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Bool => "Bool".to_string(),
            Type::Fn(a, b) => {
                if let Type::Fn(_, _) = **a {
                    format!("({}) -> {}", a.to_string(), b.to_string())
                } else {
                    format!("{} -> {}", a.to_string(), b.to_string())
                }
            }
        }
    }
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        let maybe_parenthesize = |e: &Expr| {
            if matches!(*e, Expr::Var(_) | Expr::Constant(_)) {
                e.to_string()
            } else {
                format!("({})", e.to_string())
            }
        };

        match self {
            Expr::Var(x) => x.to_string(),
            Expr::Constant(c) => c.to_string(),
            Expr::App { e1, e2 } => {
                format!("{} {}", maybe_parenthesize(e1), maybe_parenthesize(e2))
            }
            Expr::Abs { x, t, e } => format!("\\{}: {}.{}", x, t.to_string(), e.to_string()),
            Expr::If { e1, e2, e3 } => format!(
                "if {} then {} else {}",
                e1.to_string(),
                e2.to_string(),
                e3.to_string()
            ),
        }
    }
}

impl ToString for Constant {
    fn to_string(&self) -> String {
        match self {
            Constant::True => "true".to_string(),
            Constant::False => "false".to_string(),
        }
    }
}

pub struct Parser {
    stream: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn parse(stream: Vec<Token>) -> anyhow::Result<Expr> {
        let mut parser = Self { stream, pos: 0 };
        parser.expr()
    }

    fn expr(&mut self) -> anyhow::Result<Expr> {
        self.do_expr(true)
    }

    fn do_expr(&mut self, recurse_app: bool) -> anyhow::Result<Expr> {
        match self.peek().ok_or_else(|| anyhow!("empty stream"))? {
            Token::Identifier(_) => {
                let e1 = self.var()?;
                if recurse_app {
                    self.maybe_app(e1)
                } else {
                    Ok(e1)
                }
            }
            Token::Lambda => self.abstraction(),
            Token::If => self.if_then_else(),
            Token::LeftParen => {
                self.consume(&Token::LeftParen)?;
                let e = self.expr()?;
                self.consume(&Token::RightParen)?;
                if recurse_app {
                    self.maybe_app(e)
                } else {
                    Ok(e)
                }
            }
            Token::True => {
                self.consume(&Token::True)?;
                Ok(Expr::Constant(Constant::True))
            }
            Token::False => {
                self.consume(&Token::False)?;
                Ok(Expr::Constant(Constant::False))
            }

            t @ (Token::Then
            | Token::Else
            | Token::Colon
            | Token::Equal
            | Token::Dot
            | Token::Arrow
            | Token::Bool
            | Token::RightParen) => {
                bail!("unexpected token {:?}", t)
            }
        }
    }

    fn if_then_else(&mut self) -> anyhow::Result<Expr> {
        self.consume(&Token::If)?;
        let e1 = self.expr()?;
        self.consume(&Token::Then)?;
        let e2 = self.expr()?;
        self.consume(&Token::Else)?;
        let e3 = self.expr()?;
        Ok(Expr::If {
            e1: Box::new(e1),
            e2: Box::new(e2),
            e3: Box::new(e3),
        })
    }

    fn var(&mut self) -> anyhow::Result<Expr> {
        let ident = self.consume_ident()?;
        Ok(Expr::Var(ident))
    }

    fn maybe_app(&mut self, e1: Expr) -> anyhow::Result<Expr> {
        if let Ok(e2) = self.do_expr(false) {
            self.maybe_app(Expr::App {
                e1: Box::new(e1),
                e2: Box::new(e2),
            })
        } else {
            Ok(e1)
        }
    }

    fn abstraction(&mut self) -> anyhow::Result<Expr> {
        self.consume(&Token::Lambda)?;
        let x = self.var()?;
        self.consume(&Token::Colon)?;
        let t = self.ty()?;
        self.consume(&Token::Dot)?;
        let e = self.expr()?;
        if let Expr::Var(x) = x {
            Ok(Expr::Abs {
                x,
                t,
                e: Box::new(e),
            })
        } else {
            unreachable!()
        }
    }

    fn ty(&mut self) -> anyhow::Result<Type> {
        let t1 = match self.peek() {
            Some(Token::LeftParen) => {
                self.consume(&Token::LeftParen)?;
                let t = self.ty()?;
                self.consume(&Token::RightParen)?;
                t
            }
            Some(Token::Bool) => {
                self.consume(&Token::Bool)?;
                Type::Bool
            }
            peek => bail!("expected type, got `{:?}`", peek),
        };
        match self.peek() {
            Some(Token::Arrow) => {
                self.consume(&Token::Arrow)?;
                let t2 = self.ty()?;
                Ok(Type::Fn(Box::new(t1), Box::new(t2)))
            }
            _ => Ok(t1),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let stream = vec![
            Token::LeftParen,
            Token::Lambda,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Bool,
            Token::Dot,
            Token::Identifier("x".into()),
            Token::RightParen,
            Token::True,
        ];
        let e = Parser::parse(stream).unwrap();
        assert_eq!(
            e,
            Expr::App {
                e1: Box::new(Expr::Abs {
                    x: "x".into(),
                    t: Type::Bool,
                    e: Box::new(Expr::Var("x".into()))
                }),
                e2: Box::new(Expr::Constant(Constant::True))
            }
        );
    }
}
