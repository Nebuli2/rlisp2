//! This module provides the core of the interpreter, as well as functionalist
//! relating to expressions within the rlisp language. The function
//! `Expression::eval` is the heart of the interpreter.

use crate::{
    context::Context,
    exception::{
        self,
        Exception::{self, *},
    },
    util::Str,
};
use im::ConsList;
use std::{fmt, rc::Rc};

/// The expressions captured by a closure.
pub type Capture = HashMap<Str, Expression>;

/// The data stored by an instance of a custom struct type.
pub struct StructData {
    /// The name of the struct type.
    pub name: Str,

    /// A list of the values stored in the struct.
    pub data: Vec<Expression>,
}

/// Any value that may be called as a function.
#[derive(Clone)]
pub enum Callable {
    /// A quote, i.e. `(quote hello)`. When a quote expression is evaluated,
    /// the inner expression is returned, unevaluated.
    Quote,

    /// A quasiquote, i.e. `(quasiquote (1 2 (unquote (+ 1 2))))`. A
    /// quasiquoted expression is similar to a quoted expression, however parts
    /// of it may be "unquoted", wherein they are evaluated, while the rest is
    /// not.
    Quasiquote,

    /// An unquote, i.e. `... (unquote (+ 1 2)) ...`. Unquotes are used only
    /// within quasiquoted expressions to indicate that the unquoted expression
    /// should be evaluated.
    Unquote,

    /// A custom function, provided a list of parameter symbols, a body 
    /// expression, and a map of captured expressions. All values referenced 
    /// in the body of the `Lambda` are captured by value at the site of its 
    /// creation.
    Lambda(ConsList<Str>, Rc<Expression>, Option<Capture>),

    /// An intrinsic function, taking a slice of expressions and
    /// returning another expression.
    Intrinsic(Rc<Fn(&[Expression], &mut Context) -> Expression>),

    /// A macro that transforms the expression into a new expression.
    Macro(Rc<Fn(ConsList<Expression>, &mut Context) -> Expression>),
}

/// An expression in the rlisp language.
#[derive(Clone)]
pub enum Expression {
    /// A boolean expression.
    Bool(bool),

    /// A numerical expression. Numbers are represented in double floating
    /// point precision, adhering to the IEEE 754 standard.
    Num(f64),

    /// An immutable string expression.
    Str(Str),

    /// A symbol expression. When a symbol is evaluated, a lookup for its value
    /// is performed in the given evaluation context.
    Symbol(Str),

    /// A singly-linked list of expressions.
    Cons(ConsList<Expression>),

    /// A callable expression.
    Callable(Callable),

    /// An exception.
    Exception(exception::Exception),

    /// A custom struct.
    Struct(Rc<StructData>),
}

use self::Callable::*;
use self::Expression::*;
use std::collections::HashMap;

impl Expression {
    /// Determines the type of the expression.
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
        }
    }

    /// Determines whether or not the expression is nil.
    pub fn is_nil(&self) -> bool {
        match self {
            Cons(list) => list.is_empty(),
            _ => false,
        }
    }

    /// Determines whether or not the expression is an exception.
    pub fn is_exception(&self) -> bool {
        match self {
            Exception(..) => true,
            _ => false,
        }
    }

    /// Determines whether or not the expression is callable as a function.
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

    /// Evaluates the quasiquoted expression, evaluating all unquoted inner
    /// expressions.
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

    /// Attempts to call the specified expression as a function, producing the
    /// result of the function as an expression. If the expression is not
    /// callable as a function, an exception is thrown.
    pub fn call(
        &self,
        list: &ConsList<Expression>,
        ctx: &mut Context,
    ) -> Expression {
        match self {
            ex @ Exception(..) => ex.clone(),
            Callable(func) => match func {
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
                    "unquote expression must be contained in a quasiquote"
                        .into(),
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
                    args.map(|args| {
                        eval_lambda(
                            params.clone(),
                            &body,
                            args,
                            ctx,
                            capture.as_ref(),
                        )
                    }).unwrap_or_else(|e| e)
                }
            },
            _ => Exception(Custom(
                3,
                format!("not a callable value: `{}`", self).into(),
            )),
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
                    func.call(list, ctx)
                } else {
                    Exception(Custom(
                        3,
                        format!("{:?} has no function to call", list.clone())
                            .into(),
                    ))
                }
            }

            // Otherwise just clone the value
            expr => expr.clone(),
        }
    }
}

/// Evaluates the specified `Lambda`. A new scope is created and the parameter
/// names are bound to the supplied arguments, after which the body is
/// evaluated in this new context.
fn eval_lambda(
    params: ConsList<Str>,
    body: &Expression,
    args: ConsList<Expression>,
    ctx: &mut Context,
    capture: Option<&Capture>,
) -> Expression {
    // Check arity
    match (params.len(), args.len()) {
        (expected, found) if expected == found => {
            ctx.ascend_scope();

            // Apply values from capture
            if let Some(capture) = capture {
                for (key, value) in capture.iter() {
                    ctx.insert(key, value.clone());
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
            Symbol(s) => write!(f, "{}", s),
            Cons(list) => {
                // Check for quote, quasiquote, unquote special cases
                if list.len() == 2 {
                    let head = list.head().unwrap();
                    let body =
                        list.tail().and_then(|tail| tail.head()).unwrap();
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

                let strs: Vec<_> =
                    list.iter().map(|expr| expr.to_string()).collect();
                let inner = strs.join(" ");
                write!(f, "({})", inner)
            }
            Callable(callable) => match callable {
                Quote => write!(f, "quote"),
                Quasiquote => write!(f, "quasiquote"),
                Unquote => write!(f, "unquote"),
                _ => write!(f, "<procedure>"),
            },
            Exception(ex) => write!(f, "error[{:03}]: {}", ex.error_code(), ex),
            Struct(data) => {
                let StructData { name, data } = data.as_ref();
                write!(f, "(make-{}", name)?;
                for param in data.iter() {
                    write!(f, " {}", param)?;
                }
                write!(f, ")")
            }
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
                let strs: Vec<_> =
                    list.iter().map(|expr| format!("{:?}", expr)).collect();
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

impl PartialEq for Expression {
    fn eq(&self, other: &Expression) -> bool {
        match (self, other) {
            (Num(a), Num(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (Callable(a), Callable(b)) => match (a, b) {
                (
                    Lambda(args_a, body_a, cap_a),
                    Lambda(args_b, body_b, cap_b),
                ) => args_a == args_b && body_a == body_b && cap_a == cap_b,
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
        crate::util::nil()
    }
}

/// An extension trait to identify whether or not a value is a valid
/// identifier.
pub trait ValidIdentifier {
    /// Determines whether or not the value is a valid identifier.
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

impl Into<Result<Expression, Exception>> for Expression {
    fn into(self) -> Result<Expression, Exception> {
        match self {
            Exception(ex) => Err(ex),
            other => Ok(other),
        }
    }
}
