use context::Context;
use exception::{self, Exception::*};
use im::ConsList;
use std::fmt;
use util::Str;

pub type Capture = HashMap<Str, Expression>;

#[derive(Clone)]
pub enum Expression {
    Bool(bool),
    Num(f64),
    Str(Str),
    Symbol(Str),

    Cons(ConsList<Expression>),

    Lambda(ConsList<Str>, Box<Expression>, Option<Capture>),

    // Represents an intrinsic function, taking a slice of expressions and
    // returning another expression.
    Intrinsic(fn(&[Expression], &mut Context) -> Expression),

    // Represents a macro that transforms the expression into a new expression.
    Macro(fn(&ConsList<Expression>, &mut Context) -> Expression),

    // Represents an exception
    Exception(exception::Exception),

    Quote(Box<Expression>),
}

use self::Expression::*;
use std::collections::HashMap;

impl Expression {
    /// Determines whether or not the expression is nil.
    pub fn is_nil(&self) -> bool {
        match self {
            Cons(list) => list.is_empty(),
            _ => false,
        }
    }

    pub fn is_exception(&self) -> bool {
        match self {
            Exception(..) => true,
            _ => false,
        }
    }

    /// Extracts the values of all symbols in the specified context into the
    /// specified capture.
    fn extract_symbols_to_capture(&self, capture: &mut Capture, ctx: &Context) {
        match self {
            Symbol(ident) => {
                if let Some(value) = ctx.get(ident) {
                    capture.insert(ident.clone(), value.clone());
                }
            }
            Cons(children) => {
                for child in children.iter() {
                    child.extract_symbols_to_capture(capture, ctx);
                }
            }
            _ => (),
        }
    }

    /// Extracts the values of all symbols in the specified context into a
    /// capture and returns that capture.
    pub fn extract_symbols(&self, ctx: &Context) -> Capture {
        let mut capture = HashMap::new();
        self.extract_symbols_to_capture(&mut capture, ctx);
        capture
    }

    /// Evaluates the specified expression within the specified context.
    pub fn eval(&self, ctx: &mut Context) -> Expression {
        match self {
            Quote(expr) => (**expr).clone(),

            // Look up variable
            Symbol(ident) => ctx
                .get(ident)
                .map(|expr| expr.clone())
                .unwrap_or_else(|| Exception(Undefined(ident.clone()))),

            // Evaluate function
            Cons(list) => {
                if let Some(func) = list.head() {
                    let func = func.eval(ctx);
                    match func {
                        Macro(f) => f(list, ctx),
                        Intrinsic(f) => {
                            let args: Result<Vec<_>, _> = list
                                .tail()
                                .unwrap_or_else(|| ConsList::new())
                                .iter()
                                .map(|expr| match expr.eval(ctx) {
                                    Exception(e) => Err(e),
                                    expr => Ok(expr),
                                })
                                .collect();
                            args.map(|args| f(&args, ctx))
                                .unwrap_or_else(|e| Exception(e))
                        }
                        Lambda(params, body, capture) => {
                            let args: Result<ConsList<_>, _> = list
                                .tail()
                                .unwrap_or_default()
                                .iter()
                                .map(|expr| match expr.eval(ctx) {
                                    e @ Exception(_) => Err(e),
                                    expr => Ok(expr),
                                })
                                .collect();
                            args.map(|args| eval_lambda(params, &body, args, ctx, capture))
                                .unwrap_or_else(|e| e)
                        }
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
    capture: Option<Capture>,
) -> Expression {
    // Check arity
    match (params.len(), args.len()) {
        (expected, found) if expected == found => {
            ctx.ascend_scope();

            // Apply values from capture
            if let Some(capture) = capture {
                for (key, value) in capture.into_iter() {
                    ctx.insert(key, value);
                }
            }

            // Apply arguments to parameters
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
            Lambda(params, body, ..) => {
                let params_vec: Vec<_> =
                    params.iter().map(|param| param.as_ref().clone()).collect();
                let inner = params_vec.join(" ");
                write!(f, "(lambda [{}] {})", inner, body)?;
            }
            Intrinsic(..) => write!(f, "<intrinsic>")?,
            Macro(..) => write!(f, "<macro>")?,
            Exception(ex) => write!(f, "[Exception]: {}", ex)?,
        }
        Ok(())
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quote(expr) => write!(f, "<Quote:{:?}>", expr),
            Bool(b) => write!(f, "<Bool:{}>", b),
            Num(n) => write!(f, "<Num:{}>", n),
            Str(s) => write!(f, "<Str:\"{}\">", s),
            Symbol(s) => write!(f, "<Symbol:{}>", s),
            Cons(list) => {
                let strs: Vec<_> = list.iter().map(|expr| format!("{:?}", expr)).collect();
                let inner = strs.join(", ");
                write!(f, "<Cons:[{}]>", inner)
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
            (Lambda(args_a, body_a, cap_a), Lambda(args_b, body_b, cap_b)) => {
                args_a == args_b && body_a == body_b && cap_a == cap_b
            }
            (Quote(a), Quote(b)) => a == b,
            (Cons(a), Cons(b)) => a == b,
            _ => false,
        }
    }
}

impl Default for Expression {
    fn default() -> Self {
        nil()
    }
}
