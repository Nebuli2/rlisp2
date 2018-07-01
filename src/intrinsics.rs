use expression::Expression;
use expression::Expression::*;

fn unary_fn(args: &[Expression], f: impl Fn(f64) -> f64) -> Expression {
    match args {
        [Num(x)] => Num(f(*x)),
        [value] => Error(format!("expected num, found {}", value)),
        arr => Error(format!("arity mismatch: expected 1, found {}", arr.len()))
    }
}

/// 
fn binary_fn(args: &[Expression], f: impl Fn(f64, f64) -> f64) -> Expression {
    match args {
        [Num(x), Num(y)] => Num(f(*x, *y)),
        [x, y] => Error(format!("expected num, num, found {}, {}", x, y)),
        arr => Error(format!("arity mismatch: expected 2, found {}", arr.len()))
    }
}

use std::ops::{Add, Sub, Mul, Div, Rem};

/// `+ : num num -> num`
pub fn _add(args: &[Expression]) -> Expression {
    binary_fn(args, Add::add)
}

/// `- : num num -> num`
pub fn _sub(args: &[Expression]) -> Expression {
    binary_fn(args, Sub::sub)
}

/// `* : num num -> num`
pub fn _mul(args: &[Expression]) -> Expression {
    binary_fn(args, Mul::mul)
}

/// `/ : num num -> num`
pub fn _div(args: &[Expression]) -> Expression {
    binary_fn(args, Div::div)
}

/// `% : num num -> num`
pub fn _rem(args: &[Expression]) -> Expression {
    binary_fn(args, Rem::rem)
}