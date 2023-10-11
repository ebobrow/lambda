use std::collections::HashSet;

use crate::parser::{Constant, Expr};

fn is_value(expr: &Expr) -> bool {
    matches!(expr, Expr::Constant(_) | Expr::Var(_) | Expr::Abs { .. })
}

pub fn interpret(expr: &Expr, by_value: bool) -> Expr {
    match expr {
        Expr::Var(_) | Expr::Constant(_) | Expr::Abs { .. } => expr.clone(),
        Expr::App { e1, e2 } => app(e1, e2, by_value),
        Expr::If { e1, e2, e3 } => r#if(e1, e2, e3, by_value),
    }
}

fn app(e1: &Expr, e2: &Expr, by_value: bool) -> Expr {
    if !is_value(e1) {
        let e1_new = interpret(e1, by_value);
        app(&e1_new, e2, by_value)
    } else if by_value && !is_value(e2) {
        let e2_new = interpret(e2, by_value);
        app(e1, &e2_new, by_value)
    } else if let Expr::Abs { x, t: _, e } = e1 {
        interpret(&substitute(e, x, e2), by_value)
    } else {
        unreachable!("failed typechecking")
    }
}

fn r#if(e1: &Expr, e2: &Expr, e3: &Expr, by_value: bool) -> Expr {
    if !is_value(e1) {
        let e1_new = interpret(e1, by_value);
        r#if(&e1_new, e2, e3, by_value)
    } else if let Expr::Constant(Constant::True) = e1 {
        interpret(e2, by_value)
    } else {
        interpret(e3, by_value)
    }
}

fn substitute(expr: &Expr, old: &String, new: &Expr) -> Expr {
    match expr {
        Expr::Var(x) if x == old => new.clone(),
        Expr::Var(_) | Expr::Constant(_) => expr.clone(),
        Expr::App { e1, e2 } => Expr::App {
            e1: Box::new(substitute(e1, old, new)),
            e2: Box::new(substitute(e2, old, new)),
        },
        Expr::Abs { x, t: _, e: _ } if x == old => expr.clone(),
        Expr::Abs { x, t, e } => {
            let fv_body = fv(new);
            if fv_body.contains(x) {
                // Just add 1s until we have a new variable name!
                let mut new_name = format!("{x}1");
                while fv_body.contains(&new_name) {
                    new_name = format!("{new_name}1");
                }
                // TODO: what if substitution fails here too
                let e = substitute(e, x, &Expr::Var(new_name.clone()));
                Expr::Abs {
                    x: new_name,
                    t: t.clone(),
                    e: Box::new(substitute(&e, old, new)),
                }
            } else {
                Expr::Abs {
                    x: x.to_string(),
                    t: t.clone(),
                    e: Box::new(substitute(e, old, new)),
                }
            }
        }
        Expr::If { e1, e2, e3 } => Expr::If {
            e1: Box::new(substitute(e1, old, new)),
            e2: Box::new(substitute(e2, old, new)),
            e3: Box::new(substitute(e3, old, new)),
        },
    }
}

fn fv(e: &Expr) -> HashSet<&String> {
    match e {
        Expr::Var(x) => HashSet::from([x]),
        Expr::Constant(_) => HashSet::new(),
        Expr::App { e1, e2 } => {
            let fv1 = fv(e1);
            let fv2 = fv(e2);
            fv1.union(&fv2).cloned().collect()
        }
        Expr::Abs { x, t: _, e } => {
            let mut set = fv(e);
            set.remove(x);
            set
        }
        Expr::If { e1, e2, e3 } => {
            let fv1 = fv(e1);
            let fv2 = fv(e2);
            let fv3 = fv(e3);
            fv1.union(&fv2)
                .cloned()
                .collect::<HashSet<_>>()
                .union(&fv3)
                .cloned()
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitution() {
        // (\x.x y)[y := x] => (\x1.x1 x)
        let e = Expr::Abs {
            x: "x".into(),
            t: crate::parser::Type::Bool,
            e: Box::new(Expr::App {
                e1: Box::new(Expr::Var("x".into())),
                e2: Box::new(Expr::Var("y".into())),
            }),
        };
        let e2 = substitute(&e, &"y".into(), &Expr::Var("x".into()));
        assert_eq!(
            e2,
            Expr::Abs {
                x: "x1".into(),
                t: crate::parser::Type::Bool,
                e: Box::new(Expr::App {
                    e1: Box::new(Expr::Var("x1".into())),
                    e2: Box::new(Expr::Var("x".into()))
                })
            }
        );
    }
}
