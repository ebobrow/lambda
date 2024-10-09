use std::collections::HashSet;

use crate::parser::{Computation, Constant, Value};

pub fn interpret(comp: &Computation) -> Value {
    match comp {
        Computation::Return(_) => todo!(),
        Computation::OpCall { op, param } => todo!(),
        Computation::Seq { x, c1, c2 } => seq(x, c1, c2),
        Computation::If { v, c1, c2 } => r#if(v, c1, c2),
        Computation::App { v1, v2 } => app(v1, v2),
        Computation::Handling { with, handle } => todo!(),
    }
}

fn app(v1: &Value, v2: &Value) -> Value {
    if let Value::Fun { x, c } = v1 {
        interpret(&substitute_comp(c, x, v2))
    } else {
        unreachable!()
    }
}

fn seq(x: &String, c1: &Computation, c2: &Computation) -> Value {
    match c1 {
        Computation::Return(v) => interpret(&substitute_comp(c2, x, v)),
        Computation::OpCall { op, param } => {
            // op(x;y.c) === do y <- op v in c
            // Maybe leave it in here??
            todo!()
        }
        _ => unreachable!("uh"),
    }
}

fn r#if(v: &Value, c1: &Computation, c2: &Computation) -> Value {
    match v {
        Value::Constant(Constant::True) => interpret(&c1),
        Value::Constant(Constant::False) => interpret(&c2),
        _ => unreachable!("uh... :|"),
    }
}

fn substitute_comp(comp: &Computation, old: &String, new: &Value) -> Computation {
    match comp {
        Computation::Return(v) => Computation::Return(substitute_val(v, old, new)),
        Computation::OpCall { op, param } => todo!(),
        Computation::Seq { x, c1, c2 } => todo!(),
        Computation::If { v, c1, c2 } => todo!(),
        Computation::App { v1, v2 } => todo!(),
        Computation::Handling { with, handle } => todo!(),
        // Expr::Var(x) if x == old => new.clone(),
        // Expr::Var(_) | Expr::Constant(_) => comp.clone(),
        // Expr::App { e1, e2 } => Expr::App {
        //     e1: Box::new(substitute(e1, old, new)),
        //     e2: Box::new(substitute(e2, old, new)),
        // },
        // Expr::Abs { x, t: _, e: _ } if x == old => comp.clone(),
        // Expr::Abs { x, t, e } => {
        //     let fv_body = fv(new);
        //     if fv_body.contains(x) {
        //         // Just add 1s until we have a new variable name!
        //         let mut new_name = format!("{x}1");
        //         while fv_body.contains(&new_name) {
        //             new_name = format!("{new_name}1");
        //         }
        //         // TODO: what if substitution fails here too
        //         let e = substitute(e, x, &Expr::Var(new_name.clone()));
        //         Expr::Abs {
        //             x: new_name,
        //             t: t.clone(),
        //             e: Box::new(substitute(&e, old, new)),
        //         }
        //     } else {
        //         Expr::Abs {
        //             x: x.to_string(),
        //             t: t.clone(),
        //             e: Box::new(substitute(e, old, new)),
        //         }
        //     }
        // }
        // Expr::If { e1, e2, e3 } => Expr::If {
        //     e1: Box::new(substitute(e1, old, new)),
        //     e2: Box::new(substitute(e2, old, new)),
        //     e3: Box::new(substitute(e3, old, new)),
        // },
    }
}

fn substitute_val(v: &Value, old: &String, new: &Value) -> Value {
    match v {
        Value::Var(_) => todo!(),
        Value::Constant(_) => todo!(),
        Value::Fun { x, c } => todo!(),
        Value::Handler(_) => todo!(),
    }
}

// fn fv(e: &Expr) -> HashSet<&String> {
//     match e {
//         Expr::Var(x) => HashSet::from([x]),
//         Expr::Constant(_) => HashSet::new(),
//         Expr::App { e1, e2 } => {
//             let fv1 = fv(e1);
//             let fv2 = fv(e2);
//             fv1.union(&fv2).cloned().collect()
//         }
//         Expr::Abs { x, t: _, e } => {
//             let mut set = fv(e);
//             set.remove(x);
//             set
//         }
//         Expr::If { e1, e2, e3 } => {
//             let fv1 = fv(e1);
//             let fv2 = fv(e2);
//             let fv3 = fv(e3);
//             fv1.union(&fv2)
//                 .cloned()
//                 .collect::<HashSet<_>>()
//                 .union(&fv3)
//                 .cloned()
//                 .collect()
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn substitution() {
//         // (\x.x y)[y := x] => (\x1.x1 x)
//         let e = Expr::Abs {
//             x: "x".into(),
//             t: crate::parser::Type::Bool,
//             e: Box::new(Expr::App {
//                 e1: Box::new(Expr::Var("x".into())),
//                 e2: Box::new(Expr::Var("y".into())),
//             }),
//         };
//         let e2 = substitute(&e, &"y".into(), &Expr::Var("x".into()));
//         assert_eq!(
//             e2,
//             Expr::Abs {
//                 x: "x1".into(),
//                 t: crate::parser::Type::Bool,
//                 e: Box::new(Expr::App {
//                     e1: Box::new(Expr::Var("x1".into())),
//                     e2: Box::new(Expr::Var("x".into()))
//                 })
//             }
//         );
//     }
// }
