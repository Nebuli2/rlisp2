use context::Context;
use environment::Environment;
use exception::Exception::*;
use expression::Expression;
use expression::Expression::*;
use util::{nil, wrap_begin, Str};

use im::ConsList;

const DEFINE: &str = "define";

fn create_lambda(params: ConsList<Expression>, body: Expression, ctx: &Context) -> Expression {
    let params: Result<ConsList<Str>, ()> = params
        .iter()
        .map(|param| match *param {
            Symbol(ref name) => Ok(name.clone()),
            _ => Err(()),
        })
        .collect();
    params
        .map(|params| {
            // Attempt to create capture
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

pub fn _lambda(expr: &Expression, ctx: &mut Context) -> Expression {
    match expr {
        Cons(list) => {
            let params = list.tail().and_then(|list| list.head());
            let body = list.tail().and_then(|list| list.tail()).map(wrap_begin);
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
        _ => Exception(Syntax("(lambda [args...] body)".into())),
    }
    // match &vec[..] {
    //     [_lambda, Cons(params), body] => {
    //         create_lambda(params.clone(), body.clone())
    //                 // let params: Result<ConsList<String>, ()> = params
    //                 //     .iter()
    //                 //     .map(|param| match *param {
    //                 //         Symbol(ref name) => Ok(name.clone()),
    //                 //         _ => Err(()),
    //                 //     })
    //                 //     .collect();
    //                 // params
    //                 //     .map(|params| Lambda(params, Box::new(body.clone())))
    //                 //     .unwrap_or_else(|_| {
    //                 //         Exception(Syntax("(lambda [args...] body)".to_string()))
    //                 //     })
    //             }
    //             _ => Exception(Syntax("(lambda [args...] body)".into())),
    //         }
    //     }
    //     _ => Exception(Syntax("(lambda [args...] body)".into())),
    // }
}

pub fn _define(expr: &Expression, ctx: &mut Context) -> Expression {
    match expr {
        Cons(list) => {
            let ident = list.tail().and_then(|list| list.head());
            let body = list.tail().and_then(|list| list.tail()).map(wrap_begin);
            match (ident, body) {
                (Some(ident), Some(body)) => match (*ident).clone() {
                    Cons(params) => {
                        if let Some(ident) = params.head() {
                            match (*ident).clone() {
                                Str(ident) => {
                                    let params = list.tail().unwrap_or_else(|| ConsList::new());
                                    let lambda = create_lambda(params, body, ctx);
                                    ctx.insert(ident, lambda);
                                    nil()
                                }
                                _ => Exception(Syntax("(define ident body...)".into())),
                            }
                        } else {
                            Exception(Syntax("(define ident body...)".into()))
                        }
                    }
                    _ => Exception(Syntax("(define ident body...)".into())),
                },
                _ => Exception(Syntax("(define ident body...)".into())),
            };

            let vec: Vec<_> = list.iter().map(|expr| (*expr).clone()).collect();
            let _define = Symbol(DEFINE.into());
            match &vec[..] {
                [_define, Symbol(name), value] => {
                    // Stuff
                    let value = value.eval(ctx);
                    ctx.insert(name, value);
                    nil()
                }
                [_define, Cons(func), body] => {
                    // Stuff
                    let func_args = func.tail().unwrap_or_default();
                    let func_name = func.head().map(|expr| (*expr).clone());
                    if let Some(name) = func_name {
                        match name {
                            Symbol(name) => {
                                let lambda = create_lambda(func_args, body.clone(), ctx);
                                if let Exception(e) = lambda {
                                    Exception(e)
                                } else {
                                    ctx.insert(name.clone(), lambda);
                                    nil()
                                }
                            }
                            _ => Exception(Signature("".into(), "not that".into())),
                        }
                    } else {
                        Exception(Signature("".into(), "not that".into()))
                    }
                }
                _ => Exception(Signature("".into(), "not that".into())),
            }
        }
        _ => Exception(Signature("".into(), "not that".into())),
    }
}

pub fn _env(expr: &Expression, ctx: &mut Context) -> Expression {
    match expr {
        Cons(list) => {
            let arg = list.tail()
                .and_then(|tail| tail.head())
                .map(|arg| arg.eval(ctx));
            arg.map(|arg| match arg {
                Symbol(ident) => ctx.get(ident)
                    .map(|expr| expr.clone())
                    .unwrap_or_else(|| Quote(Box::new(Cons(ConsList::new())))),
                _ => Exception(Signature("symbol".into(), arg.to_string().into())),
            }).unwrap_or_else(|| Exception(Arity(1, 99)))
        }
        _ => Exception(Signature("".into(), "not that".into())),
    }
}

pub fn _if(expr: &Expression, ctx: &mut Context) -> Expression {
    match expr {
        Cons(list) => {
            let cond = list.tail()
                .and_then(|tail| tail.head())
                .map(|expr| expr.eval(ctx));
            let then_branch = list.tail()
                .and_then(|tail| tail.tail())
                .and_then(|tail| tail.head());
            let else_branch = list.tail()
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
        _ => Exception(Custom("".into())),
    }
}

pub fn _cond(expr: &Expression, ctx: &mut Context) -> Expression {
    match expr {
        Cons(list) => {
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
                                    return Exception(Syntax(
                                        "condition must be a boolean value".into(),
                                    ));
                                }
                            },
                            _ => {
                                ctx.descend_scope();
                                return Exception(Syntax(
                                    "condition block must contain 2 elements".into(),
                                ));
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
            nil()
        }
        _ => Exception(Syntax("".into())),
    }
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
