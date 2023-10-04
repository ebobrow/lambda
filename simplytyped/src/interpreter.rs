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
        // TODO: free variable stuff
        Expr::Abs { x, t, e } => Expr::Abs {
            x: x.to_string(),
            t: t.clone(),
            e: Box::new(substitute(e, old, new)),
        },
        Expr::If { e1, e2, e3 } => Expr::If {
            e1: Box::new(substitute(e1, old, new)),
            e2: Box::new(substitute(e2, old, new)),
            e3: Box::new(substitute(e3, old, new)),
        },
    }
}
