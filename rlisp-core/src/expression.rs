use context::Context;
use exception::{self, Exception, Exception::*};
use im::ConsList;
use std::fmt;
use std::rc::Rc;
use util::Str;

pub type Capture = HashMap<Str, Expression>;

pub struct StructData {
    pub name: Str,
    pub data: Vec<Expression>,
}

#[derive(Clone)]
pub enum Callable {
    Quote,
    Quasiquote,
    Unquote,

    // Lambda
    Lambda(ConsList<Str>, Rc<Expression>, Option<Capture>),

    // Represents an intrinsic function, taking a slice of expressions and
    // returning another expression.
    Intrinsic(Rc<Fn(&[Expression], &mut Context) -> Expression>),

    // Represents a macro that transforms the expression into a new expression.
    Macro(Rc<Fn(ConsList<Expression>, &mut Context) -> Expression>),
}

#[derive(Clone)]
pub enum Expression {
    Bool(bool),
    Num(f64),
    Str(Str),
    Symbol(Str),

    Cons(ConsList<Expression>),

    DottedList(Rc<Expression>, Rc<Expression>),
    Nil,

    Callable(Callable),

    // Represents an exception
    Exception(exception::Exception),

    Struct(Rc<StructData>),
}

use self::Callable::*;
use self::Expression::*;
use std::collections::HashMap;

impl Expression {
    pub fn type_of(&self) -> Str {
        match self {
            Num(..) => "num".into(),
            Bool(..) => "bool".into(),
            Str(..) => "string".into(),
            Cons(..) => "cons".into(),
            Exception(..) => "error".into(),
            Symbol(..) => "symbol".into(),
            Callable(..) => "procedure".into(),
            Struct(data) => data.name.clone(),
            _ => "unknown".into(),
        }
    }

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

    pub fn is_callable(&self) -> bool {
        match self {
            Callable(..) => true,
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

    fn eval_quasiquote(&self, ctx: &mut Context) -> Expression {
        match self {
            Cons(list) => {
                // Handle unquote
                if list.len() == 2 {
                    if let Some(head) = list.head() {
                        if let Callable(Unquote) = head.as_ref() {
                            let expr = list.iter().nth(1).unwrap();
                            return expr.eval(ctx);
                        }
                    }
                }

                let new_list: ConsList<_> =
                    list.iter().map(|expr| expr.eval_quasiquote(ctx)).collect();
                Cons(new_list)
            }
            other => other.clone(),
        }
    }

    /// Evaluates the specified expression within the specified context.
    pub fn eval(&self, ctx: &mut Context) -> Expression {
        match self {
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
                        Callable(f) => match f {
                            Quote => match list.len() - 1 {
                                1 => {
                                    // Safe to unwrap after checking length
                                    let expr = list.iter().nth(1).unwrap();
                                    expr.as_ref().clone()
                                }
                                len => Exception(Arity(1, len)),
                            },
                            Quasiquote => match list.len() - 1 {
                                1 => {
                                    // Safe to unwrap after checking length
                                    let expr = list.iter().nth(1).unwrap();
                                    expr.eval_quasiquote(ctx)
                                }
                                len => Exception(Arity(1, len)),
                            },
                            Unquote => Exception(Syntax(
                                33,
                                "unquote expression must be contained in a quasiquote".into(),
                            )),

                            Macro(f) => f(list.clone(), ctx),
                            Intrinsic(f) => {
                                let args: Result<Vec<_>, _> = list
                                    .tail()
                                    .unwrap_or_else(|| ConsList::new())
                                    .iter()
                                    .map(|expr| match expr.eval(ctx) {
                                        Exception(e) => Err(e),
                                        expr => Ok(expr),
                                    }).collect();
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
                                    }).collect();
                                args.map(|args| eval_lambda(params, &body, args, ctx, capture))
                                    .unwrap_or_else(|e| e)
                            }
                        },
                        Exception(ex) => Exception(ex.clone()),
                        other => Exception(Custom(
                            2,
                            format!("{} is not a callable value", other).into(),
                        )),
                    }
                } else {
                    Exception(Custom(
                        3,
                        format!("{} has no function to call", Cons(list.clone())).into(),
                    ))
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
        (expected, found) => {
            println!(
                "{:?}",
                Callable(Lambda(params.clone(), Rc::new(body.clone()), None))
            );
            Exception(Arity(expected, found))
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bool(b) => write!(f, "{}", b),
            Num(n) => write!(f, "{}", n),
            Str(s) => write!(f, "\"{}\"", s),
            Symbol(s) => write!(f, "'{}", s),
            Cons(list) => {
                // Check for quote, quasiquote, unquote special cases
                if list.len() == 2 {
                    let head = list.head().unwrap();
                    let body = list.tail().and_then(|tail| tail.head()).unwrap();
                    match head.as_ref() {
                        Callable(Quote) => {
                            return write!(f, "'{}", body);
                        }
                        Callable(Quasiquote) => {
                            return write!(f, "`{}", body);
                        }
                        Callable(Unquote) => {
                            return write!(f, ",{}", body);
                        }
                        _ => {
                            // Otherwise we can ignore it
                        }
                    }
                }

                let strs: Vec<_> = list.iter().map(|expr| expr.to_string()).collect();
                let inner = strs.join(" ");
                write!(f, "({})", inner)
            }
            Callable(callable) => match callable {
                Quote => write!(f, "quote"),
                Quasiquote => write!(f, "quasiquote"),
                Unquote => write!(f, "unquote"),
                _ => write!(f, "<procedure>"),
            },
            // Lambda(params, body, ..) => {
            //     let params_vec: Vec<_> =
            //         params.iter().map(|param| param.as_ref().clone()).collect();
            //     let inner = params_vec.join(" ");
            //     write!(f, "(lambda [{}] {})", inner, body)
            // }
            Exception(ex) => write!(f, "error[{:03}]: {}", ex.error_code(), ex),
            Struct(data) => {
                let StructData { name, data } = data.as_ref();
                write!(f, "(make-{}", name)?;
                for param in data.iter() {
                    write!(f, " {}", param)?;
                }
                write!(f, ")")
            }
            DottedList(a, b) => write!(f, "{} . {}", a, b),
            Nil => write!(f, "()"),
        }
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bool(b) => write!(f, "<Bool:{}>", b),
            Num(n) => write!(f, "<Num:{}>", n),
            Str(s) => write!(f, "<Str:\"{}\">", s),
            Symbol(s) => write!(f, "<Symbol:{}>", s),
            Cons(list) => {
                let strs: Vec<_> = list.iter().map(|expr| format!("{:?}", expr)).collect();
                let inner = strs.join(", ");
                write!(f, "<Cons:[{}]>", inner)
            }
            Struct(data) => {
                let StructData { name, data } = data.as_ref();
                write!(f, "<{}:{:?}>", name, data)?;
                Ok(())
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
            (Callable(a), Callable(b)) => match (a, b) {
                (Lambda(args_a, body_a, cap_a), Lambda(args_b, body_b, cap_b)) => {
                    args_a == args_b && body_a == body_b && cap_a == cap_b
                }
                _ => false,
            },
            (Cons(a), Cons(b)) => a == b,
            (Struct(d1), Struct(d2)) => {
                let StructData {
                    name: name1,
                    data: data1,
                } = d1.as_ref();
                let StructData {
                    name: name2,
                    data: data2,
                } = d2.as_ref();

                name1 == name2 && data1 == data2
            }
            _ => false,
        }
    }
}

impl Default for Expression {
    fn default() -> Self {
        nil()
    }
}

pub trait ValidIdentifier {
    fn is_valid_identifier(&self) -> bool;
}

impl ValidIdentifier for Str {
    fn is_valid_identifier(&self) -> bool {
        match self.as_ref() {
            "define" | "cond" | "lambda" | "if" | "let" => false,
            _ => true,
        }
    }
}

impl ValidIdentifier for Expression {
    fn is_valid_identifier(&self) -> bool {
        match self {
            Symbol(s) => s.is_valid_identifier(),
            _ => false,
        }
    }
}

// Conversions

macro_rules! impl_num_to_expr {
    ($($type:ty),*) => {
        $(
            impl Into<Expression> for $type {
                fn into(self) -> Expression {
                    let n = self as f64;
                    Num(n)
                }
            }
        )*
    };
}

impl_num_to_expr!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl Into<Expression> for bool {
    fn into(self) -> Expression {
        Bool(self)
    }
}

impl Into<Expression> for Str {
    fn into(self) -> Expression {
        Str(self)
    }
}

impl Into<Expression> for String {
    fn into(self) -> Expression {
        Str(self.into())
    }
}

impl<'a> Into<Expression> for &'a str {
    fn into(self) -> Expression {
        Str(self.into())
    }
}

// impl<T, U> From<T> for Expression
// where
//     T: Iterator<Item = U>,
//     U: Into<Expression>,
// {
//     fn from(t: T) -> Expression {

//         unimplemented!()
//     }
// }

impl Into<Result<Expression, Exception>> for Expression {
    fn into(self) -> Result<Expression, Exception> {
        match self {
            Exception(ex) => Err(ex),
            other => Ok(other),
        }
    }
}
