use context::Context;
use environment::Environment;
use exception::{self, Exception::*};
use im::ConsList;
use std::fmt;
use util::Str;

#[derive(Clone)]
pub enum Expression {
    Bool(bool),
    Num(f64),
    Str(Str),
    Symbol(Str),

    Cons(ConsList<Expression>),

    Lambda(ConsList<Str>, Box<Expression>),

    // Represents an intrinsic function, taking a slice of expressions and
    // returning another expression.
    Intrinsic(fn(&[Expression]) -> Expression),

    // Represents a macro that transforms the expression into a new expression.
    Macro(fn(&Expression, &mut Context) -> Expression),

    // Represents an exception
    Exception(exception::Exception),

    Quote(Box<Expression>),
}

use self::Expression::*;

impl Expression {
    pub fn is_nil(&self) -> bool {
        match self {
            Cons(list) => list.is_empty(),
            _ => false,
        }
    }

    pub fn eval(&self, ctx: &mut Context) -> Expression {
        match self {
            Quote(expr) => (**expr).clone(),

            // Look up variable
            Symbol(ident) => ctx.get(ident)
                .map(|expr| expr.clone())
                .unwrap_or_else(|| Exception(Undefined(ident.clone()))),

            // Evaluate function
            Cons(list) => {
                if let Some(func) = list.head() {
                    let func = func.eval(ctx);
                    match func {
                        Macro(f) => f(self, ctx),
                        Intrinsic(f) => {
                            let args: Vec<_> = list.tail()
                                .unwrap_or_else(|| ConsList::new())
                                .iter()
                                .map(|expr| expr.eval(ctx))
                                .collect();
                            f(&args)
                        }
                        Lambda(params, body) => eval_lambda(
                            params,
                            &body,
                            list.tail()
                                .unwrap_or_else(|| ConsList::new())
                                .iter()
                                .map(|expr| expr.eval(ctx))
                                .collect(),
                            ctx,
                        ),
                        _ => Exception(Custom("not a callable value".into())),
                    }
                } else {
                    Exception(Custom("no function to call".into()))
                }
            }

            // Otherwise just clone the value
            expr => expr.clone(),
        }
    }
}

fn eval_lambda(
    params: ConsList<Str>,
    body: &Expression,
    args: ConsList<Expression>,
    ctx: &mut Context,
) -> Expression {
    // Check arity
    match (params.len(), args.len()) {
        (expected, found) if expected == found => {
            ctx.ascend_scope();
            for (param, arg) in params.iter().zip(args.iter()) {
                ctx.insert(param.to_string(), (*arg).clone());
            }
            let res = body.eval(ctx);
            ctx.descend_scope();
            res
        }
        (expected, found) => Exception(Arity(expected, found)),
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quote(expr) => write!(f, "'{}", expr)?,
            Bool(b) => write!(f, "{}", b)?,
            Num(n) => write!(f, "{}", n)?,
            Str(s) => write!(f, "\"{}\"", s)?,
            Symbol(s) => write!(f, "{}", s)?,
            Cons(list) => {
                let strs: Vec<_> = list.iter().map(|expr| expr.to_string()).collect();
                let inner = strs.join(" ");
                write!(f, "({})", inner)?;
            }
            Lambda(..) => write!(f, "<lambda>")?,
            Intrinsic(..) => write!(f, "<intrinsic>")?,
            Macro(..) => write!(f, "<macro>")?,
            Exception(ex) => write!(f, "[exception] {}", ex)?,
        }
        Ok(())
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quote(expr) => write!(f, "Quote({})", expr),
            Bool(b) => write!(f, "Bool({})", b),
            Num(n) => write!(f, "Num({})", n),
            Str(s) => write!(f, "Str(\"{}\")", s),
            Symbol(s) => write!(f, "Symbol({})", s),
            Cons(list) => {
                let strs: Vec<_> = list.iter().map(|expr| format!("{:?}", expr)).collect();
                let inner = strs.join(", ");
                write!(f, "Cons({})", inner)
            }
            other => write!(f, "{}", other),
        }
    }
}

pub fn nil() -> Expression {
    Cons(ConsList::new())
}

impl PartialEq for Expression {
    fn eq(&self, other: &Expression) -> bool {
        match (self, other) {
            (Num(a), Num(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (Lambda(args_a, body_a), Lambda(args_b, body_b)) => {
                args_a == args_b && body_a == body_b
            }
            (Quote(a), Quote(b)) => a == b,
            (Cons(a), Cons(b)) => a == b,
            _ => false,
        }
    }
}
