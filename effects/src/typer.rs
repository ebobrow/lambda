use std::collections::HashMap;

use anyhow::{anyhow, bail, Ok};

use crate::parser::{Constant, Expr, Type};

#[derive(Default)]
pub struct Typer {
    context: HashMap<String, Type>,
}

impl Typer {
    pub fn typecheck(&mut self, e: &Expr) -> anyhow::Result<Type> {
        match e {
            Expr::Var(x) => self.var(x),
            Expr::Constant(c) => self.constant(c),
            Expr::App { e1, e2 } => self.app(e1, e2),
            Expr::Abs { x, t, e } => self.abs(x, t, e),
            Expr::If { e1, e2, e3 } => self.r#if(e1, e2, e3),
        }
    }

    fn var(&self, var: &String) -> anyhow::Result<Type> {
        Ok(self
            .context
            .get(var)
            .ok_or_else(|| anyhow!("undeclared variable `{}`", var))?
            .clone())
    }

    fn constant(&self, constant: &Constant) -> anyhow::Result<Type> {
        match constant {
            Constant::True | Constant::False => Ok(Type::Bool),
        }
    }

    fn app(&mut self, e1: &Expr, e2: &Expr) -> anyhow::Result<Type> {
        if let Type::Fn(a, b) = self.typecheck(e1)? {
            let t2 = self.typecheck(e2)?;
            if a.as_ref() == &t2 {
                Ok(*b)
            } else {
                bail!("invalid argument; expected type `{:?}`", a);
            }
        } else {
            bail!("expected function type, got `{:?}`", e1);
        }
    }

    fn abs(&mut self, x: &String, t: &Type, e: &Expr) -> anyhow::Result<Type> {
        self.context.insert(x.to_string(), t.clone());
        let t2 = self.typecheck(e)?;
        self.context.remove(x);
        Ok(Type::Fn(Box::new(t.clone()), Box::new(t2.clone())))
    }

    fn r#if(&mut self, e1: &Expr, e2: &Expr, e3: &Expr) -> anyhow::Result<Type> {
        if let Type::Bool = self.typecheck(e1)? {
            let t2 = self.typecheck(e2)?;
            let t3 = self.typecheck(e3)?;
            if t2 == t3 {
                Ok(t2)
            } else {
                bail!("mismatched `if` branches `{:?}` and `{:?}`", t2, t3);
            }
        } else {
            bail!("expected boolean");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let e = Expr::Abs {
            x: "x".into(),
            t: Type::Bool,
            e: Box::new(Expr::Var("x".into())),
        };
        let ty = Typer::default().typecheck(&e).unwrap();
        assert_eq!(ty, Type::Fn(Box::new(Type::Bool), Box::new(Type::Bool)));
    }
}
