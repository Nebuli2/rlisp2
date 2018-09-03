use context::Context;
use exception::Exception::*;
use expression::Expression;
use expression::Expression::*;
use util::{wrap_begin, Str};

use im::ConsList;

fn create_lambda(
    params: ConsList<Expression>,
    body: ConsList<Expression>,
    ctx: &Context,
) -> Expression {
    let params: Result<ConsList<Str>, ()> = params
        .iter()
        .map(|param| match *param {
            Symbol(ref name) => Ok(name.clone()),
            _ => Err(()),
        })
        .collect();
    params
        .map(|params| {
            let body = if body.len() == 1 {
                body.head().map(|expr| expr.as_ref().clone())
            } else {
                Some(wrap_begin(body))
            }.unwrap_or_default();
            let capture = body.extract_symbols(ctx);
            let capture = if capture.is_empty() {
                None
            } else {
                Some(capture)
            };
            Lambda(params, Box::new(body.clone()), capture)
        })
        .unwrap_or_else(|_| Exception(Syntax("(lambda [args...] body)".into())))
}

pub fn _lambda(list: &ConsList<Expression>, ctx: &mut Context) -> Expression {
    let params = list.tail().and_then(|list| list.head());
    let body = list.tail().and_then(|list| list.tail());
    // let vec: Vec<_> = list.iter().map(|expr| (*expr).clone()).collect();
    // let _lambda = Symbol(LAMBDA.into());
    match (params, body) {
        (Some(params), Some(body)) => match (*params).clone() {
            Cons(list) => create_lambda(list.clone(), body, ctx),
            _ => Exception(Syntax("(lambda [args...] body)".into())),
        },
        // create_lambda(params.clone(), body.clone()),
        _ => Exception(Syntax("(lambda [args...] body)".into())),
    }
}

pub fn _quote(list: &ConsList<Expression>, _: &mut Context) -> Expression {
    match list.len() - 1 {
        n if n != 1 => Exception(Arity(1, n)),
        _ => {
            let expr = list.tail().and_then(|list| list.head());
            match expr {
                Some(expr) => {
                    let bx = Box::new((*expr).clone());
                    Quote(bx)
                }
                _ => Exception(Arity(1, 0)),
            }
        }
    }
}

pub fn _define(list: &ConsList<Expression>, ctx: &mut Context) -> Expression {
    list.tail()
        .and_then(|list| list.head())
        .map(|head| (*head).clone())
        .map(|head| match head {
            Symbol(ident) => {
                // Simple binding
                // Check arity
                match list.len() {
                    len if len == 3 => {
                        let value = list.iter().nth(2).map(|expr| expr.eval(ctx)).unwrap();
                        ctx.insert(ident, value);
                    }
                    len => {
                        // Arity mismatch
                    }
                }
            }
            Cons(func) => {
                // Function binding
                let ident = func.head().map(|ident| (*ident).clone());
                ident.map(|ident| match ident {
                    Symbol(ident) => {
                        // Continue
                        let params = func.tail().unwrap_or_default();
                        let params: Option<ConsList<_>> = params
                            .iter()
                            .map(|param| match param.as_ref() {
                                ident @ Symbol(..) => Some(ident.clone()),
                                _ => None,
                            })
                            .collect();
                        params.map(|params| {
                            let body = list.tail().and_then(|list| list.tail());
                            body.map(|body| {
                                let lambda = create_lambda(params, body, ctx);
                                ctx.insert(ident, lambda);
                            });
                        });
                    }
                    other => {
                        // Error, must have symbol as function identifier
                    }
                });
            }
            _ => {}
        });
    Expression::default()
}

pub fn _env(list: &ConsList<Expression>, ctx: &mut Context) -> Expression {
    let arg = list
        .tail()
        .and_then(|tail| tail.head())
        .map(|arg| arg.eval(ctx));
    arg.map(|arg| match arg {
        Symbol(ident) => ctx
            .get(ident)
            .map(|expr| expr.clone())
            .unwrap_or_else(|| Quote(Box::new(Cons(ConsList::new())))),
        _ => Exception(Signature("symbol".into(), arg.to_string().into())),
    }).unwrap_or_else(|| Exception(Arity(1, 99)))
}

pub fn _if(list: &ConsList<Expression>, ctx: &mut Context) -> Expression {
    let cond = list
        .tail()
        .and_then(|tail| tail.head())
        .map(|expr| expr.eval(ctx));
    let then_branch = list
        .tail()
        .and_then(|tail| tail.tail())
        .and_then(|tail| tail.head());
    let else_branch = list
        .tail()
        .and_then(|tail| tail.tail())
        .and_then(|tail| tail.tail())
        .and_then(|tail| tail.head());
    match (cond, then_branch, else_branch) {
        (Some(Bool(cond)), Some(then_branch), Some(else_branch)) => {
            if cond {
                then_branch.eval(ctx)
            } else {
                else_branch.eval(ctx)
            }
        }
        (Some(a), Some(b), Some(c)) => Exception(Signature(
            "bool, any, any".into(),
            format!("{}, {}, {}", a, b, c).into(),
        )),
        _ => Exception(Arity(3, list.len())),
    }
}

pub fn _cond(list: &ConsList<Expression>, ctx: &mut Context) -> Expression {
    ctx.ascend_scope();

    // Ensure that "else" branch works
    ctx.insert("else", Bool(true));

    let branches = list.tail().unwrap_or_else(|| ConsList::new());
    for branch in branches.iter() {
        match branch.as_ref() {
            Cons(list) if list.len() == 2 => {
                let cond = list.head();
                let value = list.tail().and_then(|tail| tail.head());

                match (cond, value) {
                    (Some(cond), Some(value)) => match cond.eval(ctx) {
                        Bool(false) => (),
                        Bool(true) => {
                            ctx.descend_scope();
                            return value.eval(ctx);
                        }
                        _ => {
                            ctx.descend_scope();
                            return Exception(Syntax("condition must be a boolean value".into()));
                        }
                    },
                    _ => {
                        ctx.descend_scope();
                        return Exception(Syntax("condition block must contain 2 elements".into()));
                    }
                }
            }
            _ => {
                ctx.descend_scope();
                return Exception(Syntax("condition block must be a list".into()));
            }
        }
    }

    ctx.descend_scope();
    Expression::default()
}

pub fn _let(list: &ConsList<Expression>, ctx: &mut Context) -> Expression {
    let bindings = list.tail().and_then(|tail| tail.head());
    let body = list.tail().and_then(|list| list.tail());

    ctx.ascend_scope();
    let bindings = bindings
        .ok_or_else(|| Arity(2, 0))
        .and_then(|bindings| match bindings.as_ref().clone() {
            Cons(bindings_list) => Ok(bindings_list),
            _ => Err(Syntax("binding list must be a list of bindings".into())), // Better error handling than none
        })
        .and_then(|bindings| {
            for binding in bindings.iter() {
                match binding.as_ref() {
                    Cons(binding) if binding.len() == 2 => {
                        // Proper binding here
                        // Unwrap is safe here as we have already checked the length
                        let ident = binding.head().unwrap();
                        let value = binding.tail().and_then(|x| x.head()).unwrap();

                        match ident.as_ref() {
                            Symbol(ident) => {
                                let value = value.eval(ctx);
                                if !value.is_exception() {
                                    ctx.insert(ident, value);
                                }
                            }
                            other => {
                                return Err(Syntax(
                                    format!(
                                        "identifier in binding must be a symbol, found {}",
                                        other
                                    ).into(),
                                ))
                            }
                        }
                    }
                    Cons(list) => return Err(Arity(2, list.len())),
                    other => {
                        return Err(Syntax(
                            format!(
                                "binding must be a list containing a symbol and a value, found {}",
                                other
                            ).into(),
                        ))
                    }
                }
            }
            Ok(())
        });

    let body = bindings
        .and(body.ok_or_else(|| Syntax("body not found".into())))
        .map(|body| match body.len() {
            1 => body.head().unwrap().as_ref().clone(),
            _ => wrap_begin(body),
        })
        .map(|body| body.eval(ctx));
    ctx.descend_scope();
    body.unwrap_or_else(|ex| Exception(ex))
}

// pub fn _eval(expr: &Expression, ctx: &mut Context) -> Expression {
//     // (eval expr env)
//     match expr {
//         Cons(list) => {
//             let expr = list.tail()
//                 .and_then(|tail| tail.head())
//                 .map(|expr| expr.eval(ctx)); // expr
//             let env = list.tail()
//                 .and_then(|tail| tail.tail())
//                 .and_then(|tail| tail.head())
//                 .map(|expr| expr.eval(ctx));
//             match (expr, env) {
//                 (Some(expr), Some(env)) => match env {

//                 }
//             }
//         },
//         _ => {}
//     }
//     Cons(ConsList::new())
// }
