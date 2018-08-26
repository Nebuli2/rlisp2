use exception::Exception::*;
use expression::Expression;
use expression::Expression::*;
use context::Context;
use im::ConsList;

fn unary_fn(args: &[Expression], f: impl Fn(f64) -> f64) -> Expression {
    match args {
        [Num(x)] => Num(f(*x)),
        [value] => Exception(Signature("num".into(), value.to_string().into())),
        arr => Exception(Arity(1, arr.len())),
    }
}

///
fn binary_fn(args: &[Expression], f: impl Fn(f64, f64) -> f64) -> Expression {
    match args {
        [Num(x), Num(y)] => Num(f(*x, *y)),
        [x, y] => Exception(Signature("num, num".into(), format!("{}, {}", x, y).into())),
        arr => Exception(Arity(2, arr.len())),
    }
}

use std::ops::{Add, Div, Mul, Rem, Sub};

/// `+ :: num num -> num`
///
/// Produces the sum of the two specified values.
pub fn _add(args: &[Expression], _: &mut Context) -> Expression {
    let xs: Result<Vec<_>, &Expression> = args
        .iter()
        .map(|expr| match expr {
            Num(n) => Ok(*n),
            other => Err(other),
        })
        .collect();

    xs.map(|xs| xs.into_iter().fold(0.0, Add::add))
        .map(|x| Num(x))
        .unwrap_or_else(|e| Exception(Signature("num".into(), e.to_string().into())))
}

/// `- :: num num -> num`
///
/// Produces the difference of the two specified values.
pub fn _sub(args: &[Expression], _: &mut Context) -> Expression {
    match args.len() {
        0 => Exception(Custom(
            "arity mismatch: expected at least 1 argument, found 0".into(),
        )),
        1 => match &args[0] {
            Num(n) => Num(-n),
            other => Exception(Signature("num".into(), other.to_string().into())),
        },
        _ => match &args[0] {
            Num(head) => {
                let tail = &args[1..];
                let nums: Option<Vec<_>> = tail
                    .iter()
                    .map(|expr| match expr {
                        Num(n) => Some(*n),
                        _ => None,
                    })
                    .collect();

                let res = nums
                    .map(|nums| nums.into_iter().fold(*head, Sub::sub))
                    .unwrap_or_else(|| *head);

                Num(res)
            }
            other => Exception(Signature("num".into(), other.to_string().into())),
        },
    }
}

/// `* :: num num -> num`
///
/// Produces the product of the two specified values.
pub fn _mul(args: &[Expression], _: &mut Context) -> Expression {
    let xs: Result<Vec<_>, &Expression> = args
        .iter()
        .map(|expr| match expr {
            Num(n) => Ok(*n),
            other => Err(other),
        })
        .collect();

    xs.map(|xs| xs.into_iter().fold(1.0, Mul::mul))
        .map(|x| Num(x))
        .unwrap_or_else(|e| Exception(Signature("num".into(), e.to_string().into())))
}

/// `/ :: num num -> num`
///
/// Produces the quotient of the two specified values.
pub fn _div(args: &[Expression], _: &mut Context) -> Expression {
    match args.len() {
        0 => Exception(Custom(
            "arity mismatch: expected at least 1 argument, found 0".into(),
        )),
        1 => match &args[0] {
            Num(n) => Num(1.0 / n),
            other => Exception(Signature("num".into(), other.to_string().into())),
        },
        _ => match &args[0] {
            Num(head) => {
                let tail = &args[1..];
                let nums: Option<Vec<_>> = tail
                    .iter()
                    .map(|expr| match expr {
                        Num(n) => Some(*n),
                        _ => None,
                    })
                    .collect();

                let res = nums
                    .map(|nums| nums.into_iter().fold(*head, Div::div))
                    .unwrap_or_else(|| *head);

                Num(res)
            }
            other => Exception(Signature("num".into(), other.to_string().into())),
        },
    }
}

/// `% :: num num -> num`
///
/// Produces the remainder of the two specified values.
pub fn _rem(args: &[Expression], _: &mut Context) -> Expression {
    binary_fn(args, Rem::rem)
}

// Exceptions

/// `arity-exception :: num num -> exception`
///
/// Produces an arity exception with the specified parameters.
pub fn _arity(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(expected), Num(found)] => {
            let (expected, found) = (*expected as usize, *found as usize);
            Exception(Arity(expected, found))
        }
        _ => Exception(Signature("num, num".into(), "not that".into())),
    }
}

// Lists

/// `cons :: a [a] -> [a]`
///
/// Produces a new list with the specified value prepended to it.
pub fn _cons(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [car, Cons(cdr)] => Cons(cdr.cons(car)),
        _ => Exception(Signature("any, cons".into(), "not that".into())),
    }
}

/// `head :: [a] -> a`
///
/// Produces the first element of the specified list.
pub fn _head(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Cons(list)] => list
            .head()
            .map(|head| (*head).clone())
            .unwrap_or_else(|| Exception(Custom("cannot get the tail of an empty list".into()))),
        _ => Exception(Signature("any, cons".into(), "not that".into())),
    }
}

/// `tail :: [a] -> [a]`
///
/// Produces the remainder of the specified list after the first element.
pub fn _tail(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Cons(list)] => list
            .tail()
            .map(|tail| Cons(tail))
            .unwrap_or_else(|| Exception(Custom("cannot get the tail of an empty list".into()))),
        _ => Exception(Signature("any, cons".into(), "not that".into())),
    }
}

/// `exit :: num -> nil`
///
/// Exits the program with the specified exit code.
pub fn _exit(args: &[Expression], _: &mut Context) -> Expression {
    use std::process::exit;

    match args {
        [Num(code)] => {
            let code = *code as i32;
            exit(code);
        }
        [] => exit(0),
        args => Exception(Custom(
            format!(
                "arity mismatch: expected 0 or 1 arguments, found {}",
                args.len()
            ).into(),
        )),
    }
}

/// `eq? :: a a -> bool`
///
/// Tests the two arguments for equality.
pub fn _eq(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [a, b] => Bool(a == b),
        args => Exception(Arity(2, args.len())),
    }
}

/// `< :: a a -> bool`
///
/// Determines whether or not the first argument is less than the second.
pub fn _lt(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b)] => Bool(a < b),
        [a, b] => Exception(Signature("num, num".into(), format!("{}, {}", a, b).into())),
        args => Exception(Arity(2, args.len())),
    }
}

/// `<= :: a a -> bool`
///
/// Determines whether or not the first argument is less than or equal to the
/// second.
pub fn _lte(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b)] => Bool(a <= b),
        [a, b] => Exception(Signature("num, num".into(), format!("{}, {}", a, b).into())),
        args => Exception(Arity(2, args.len())),
    }
}

/// `> :: a a -> bool`
///
/// Determines whether or not the first argument is greater than the second.
pub fn _gt(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b)] => Bool(a > b),
        [a, b] => Exception(Signature("num, num".into(), format!("{}, {}", a, b).into())),
        args => Exception(Arity(2, args.len())),
    }
}

/// `>= :: a a -> bool`
///
/// Determines whether or not the first argument is greater than or equal to
/// the second.
pub fn _gte(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b)] => Bool(a >= b),
        [a, b] => Exception(Signature("num, num".into(), format!("{}, {}", a, b).into())),
        args => Exception(Arity(2, args.len())),
    }
}

/// `begin :: any... a -> a`
///
/// Evaluates all passed expressions and produces the last.
pub fn _begin(args: &[Expression], _: &mut Context) -> Expression {
    args.last()
        .map(|expr| expr.clone())
        .unwrap_or_else(|| Quote(Box::new(Cons(ConsList::new()))))
}

/// `println :: a... -> nil`
///
/// Prints the specified values, separated by spaces, and terminated with a
/// linebreak.
pub fn _println(args: &[Expression], _: &mut Context) -> Expression {
    for arg in args {
        let fmt = match arg {
            Str(s) => s.to_string(),
            other => other.to_string(),
        };
        print!("{} ", fmt);
    }
    println!();
    Cons(ConsList::new())
}

pub fn _append(args: &[Expression], _: &mut Context) -> Expression {
    // Try lists
    let lists: Option<Vec<_>> = args
        .iter()
        .map(|arg| match arg {
            Cons(list) => Some(list),
            _ => None,
        })
        .collect();

    if let Some(lists) = lists {
        let total = lists
            .into_iter()
            .fold(ConsList::new(), |acc, list| acc.append(list));
        return Cons(total);
    }

    let strs: Option<Vec<_>> = args
        .iter()
        .map(|arg| match arg {
            Str(s) => Some(s),
            _ => None,
        })
        .collect();

    if let Some(strs) = strs {
        let len: usize = strs.iter().map(|s| s.len()).sum();
        let mut buf = String::with_capacity(len);
        for s in strs {
            buf.push_str(s);
        }
        return Str(buf.into());
    }

    Exception(Custom("invalid types".into()))
}

pub fn _empty(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Cons(list)] => Bool(list.is_empty()),
        [a] => Exception(Signature("[a]".into(), a.to_string().into())),
        xs => Exception(Arity(1, xs.len())),
    }
}

pub fn _eval(args: &[Expression], ctx: &mut Context) -> Expression {
    match args {
        [ex] => ex.eval(ctx),
        xs => Exception(Arity(1, xs.len())),
    }
}

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use parser::Parser;
use parser::preprocessor::*;
use util::wrap_begin;

fn load_file(file_name: impl AsRef<str>) -> Result<Expression, Box<Error>> {
    let file = File::open(file_name.as_ref())?;
    let mut reader = BufReader::new(file);

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    // Look for directive lines
    let mut use_preprocessor = false;
    {
        let iter = buf.lines()
            .filter(|line| !line.is_empty())
            .filter(|line| line.trim().starts_with('#'))
            .map(|line| line.split_at(1).1);
        for line in iter {
            if line == "enable-preprocessor" {
                use_preprocessor = true;
            } else {
                Err(format!("{} is not a known preprocessor command", line))?;
            }
        }
    }

    let removed_commands: String = buf.lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    // println!("{}", removed_commands);
    let processed;
    let iter = match use_preprocessor {
        true => {
            let stripped = first_pass(removed_commands);
            processed = process(stripped);
            processed.chars()
        }
        false => removed_commands.chars(),
    };

    let mut parser = Parser::with_name(iter, file_name.as_ref().to_string());

    let mut exprs = Vec::new();
    while let Some(expr) = parser.parse_expr() {
        exprs.push(expr);
    }

    let expr = wrap_begin(exprs.into());
    Ok(expr)
    // Ok(expr)
}

pub fn _import(args: &[Expression], ctx: &mut Context) -> Expression {
    match args {
        [Str(file_name)] => {
            let res = load_file(file_name);
            res
                .map(|ex| ex.eval(ctx))
                .unwrap_or_else(|_| Exception(Custom("Could not read file.".into())))
        },
        xs => Exception(Arity(1, xs.len())),
    }
}