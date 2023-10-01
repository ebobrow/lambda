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
    Abs { x: Box<Expr>, t: Type, e: Box<Expr> },
    /// if e1 then e2 else e3
    Let {
        e1: Box<Expr>,
        e2: Box<Expr>,
        e3: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Bool,
    Fn(Box<Type>, Box<Type>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    True,
    False,
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
                self.maybe_app(e)
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
        Ok(Expr::Let {
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
        let mut ee = Vec::new();
        while let Ok(e) = self.do_expr(false) {
            ee.push(e);
        }
        Ok(ee.iter().fold(e1, |a, b| Expr::App {
            e1: Box::new(a),
            e2: Box::new(b.clone()),
        }))
    }

    fn abstraction(&mut self) -> anyhow::Result<Expr> {
        self.consume(&Token::Lambda)?;
        let x = self.var()?;
        self.consume(&Token::Colon)?;
        let t = self.ty()?;
        self.consume(&Token::Dot)?;
        let e = self.expr()?;
        Ok(Expr::Abs {
            x: Box::new(x),
            t,
            e: Box::new(e),
        })
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
                    x: Box::new(Expr::Var("x".into())),
                    t: Type::Bool,
                    e: Box::new(Expr::Var("x".into()))
                }),
                e2: Box::new(Expr::Constant(Constant::True))
            }
        );
    }
}
