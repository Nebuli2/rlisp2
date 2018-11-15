//! This module provides intrinsic functions to the rlisp language. An
//! intrinsic function is one where all of its parameters are evaluated, and
//! then the intrinsic function is provided the evaluated arguments to produce
//! its output.

use crate::{
    context::Context,
    exception::Exception::{self, *},
    expression::Expression::{self, *},
    parser::{preprocessor::*, Parser},
    quat::Quat,
    util::{print_pretty, wrap_begin, Style},
};
use im::ConsList;
use std::{
    error::Error,
    fs::File,
    io::{self, prelude::*, stdin, stdout, BufReader},
    ops::{Add, Div, Mul, Rem, Sub},
};
use termcolor::Color;

/// Evaluates the specified unary function, checking arity and type signatures.
fn unary_fn(args: &[Expression], f: impl Fn(f64) -> f64) -> Expression {
    match args {
        [Num(x)] => Num(f(*x)),
        [value] => Exception(Signature("num".into(), value.type_of())),
        arr => Exception(Arity(1, arr.len())),
    }
}

/// Evaluates the specified binary function, checking arity and type
/// signatures.
fn binary_fn(args: &[Expression], f: impl Fn(f64, f64) -> f64) -> Expression {
    match args {
        [Num(x), Num(y)] => Num(f(*x, *y)),
        [x, y] => Exception(Signature(
            "num, num".into(),
            format!("{}, {}", x.type_of(), y.type_of()).into(),
        )),
        arr => Exception(Arity(2, arr.len())),
    }
}

/// `and :: bool ... -> bool`
///
/// Produces `true` if and only if all values are `true`. Otherwise, `false` is
/// returned.
pub fn and(args: &[Expression], _: &mut Context) -> Expression {
    let bools: Result<Vec<_>, &Expression> = args
        .iter()
        .map(|expr| match expr {
            Bool(b) => Ok(*b),
            other => Err(other),
        }).collect();
    bools
        .map(|bs| Bool(bs.iter().all(|&b| b)))
        .unwrap_or_else(|ex| Exception(Signature("bool".into(), ex.type_of())))
}

/// `or :: bool ... -> bool`
///
/// Produces `true` if and only if at least one value is `true`. Otherwise,
/// `false` is returned.
pub fn or(args: &[Expression], _: &mut Context) -> Expression {
    let bools: Result<Vec<_>, &Expression> = args
        .iter()
        .map(|expr| match expr {
            Bool(b) => Ok(*b),
            other => Err(other),
        }).collect();
    bools
        .map(|bs| Bool(bs.iter().any(|&b| b)))
        .unwrap_or_else(|ex| Exception(Signature("bool".into(), ex.type_of())))
}

/// `not :: bool -> bool`
///
/// Produces the opposite of the specified value.
pub fn not(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Bool(b)] => Bool(!b),
        [x] => Exception(Signature("bool".into(), x.type_of())),
        xs => Exception(Arity(1, xs.len())),
    }
}

/// `+ :: num ... -> num`
///
/// Produces the sum of the two specified values.
pub fn add(args: &[Expression], _: &mut Context) -> Expression {
    let xs: Result<Vec<_>, &Expression> = args
        .iter()
        .map(|expr| match expr {
            Num(n) => Ok(*n),
            other => Err(other),
        }).collect();

    if let Ok(xs) = xs {
        Num(xs.into_iter().fold(0.0, Add::add))
    } else {
        // Try quaternions
        let xs: Result<Vec<_>, &Expression> = args
            .iter()
            .map(|expr| match expr {
                Num(n) => Ok(Quat::from(*n)),
                Quaternion(n) => Ok(*n),
                other => Err(other),
            }).collect();
        if let Ok(xs) = xs {
            Quaternion(xs.into_iter().fold(Quat::default(), Add::add))
        } else {
            Exception(Arity(0, 0))
        }
    }
    // xs.map(|xs| xs.into_iter().fold(0.0, Add::add))
    //     .map(|x| Num(x))
    //     .unwrap_or_else(|e| Exception(Signature("num".into(), e.type_of())))
}

/// `- :: num ... -> num`
///
/// Produces the difference of the two specified values.
pub fn sub(args: &[Expression], _: &mut Context) -> Expression {
    match args.len() {
        0 => Exception(Custom(
            4,
            "arity mismatch: expected at least 1 argument, found 0".into(),
        )),
        1 => match &args[0] {
            Num(n) => Num(-n),
            other => Exception(Signature("num".into(), other.type_of())),
        },
        _ => match &args[0] {
            Num(head) => {
                let tail = &args[1..];
                let nums: Option<Vec<_>> = tail
                    .iter()
                    .map(|expr| match expr {
                        Num(n) => Some(*n),
                        _ => None,
                    }).collect();

                let res = nums
                    .map(|nums| nums.into_iter().fold(*head, Sub::sub))
                    .unwrap_or_else(|| *head);

                Num(res)
            }
            other => Exception(Signature("num".into(), other.type_of())),
        },
    }
}

/// `* :: num ... -> num`
///
/// Produces the product of the two specified values.
pub fn mul(args: &[Expression], _: &mut Context) -> Expression {
    let xs: Result<Vec<_>, &Expression> = args
        .iter()
        .map(|expr| match expr {
            Num(n) => Ok(*n),
            other => Err(other),
        }).collect();

    if let Ok(xs) = xs {
        Num(xs.into_iter().fold(1.0, Mul::mul))
    } else {
        // Try quaternions
        let xs: Result<Vec<_>, &Expression> = args
            .iter()
            .map(|expr| match expr {
                Num(n) => Ok(Quat::from(*n)),
                Quaternion(n) => Ok(*n),
                other => Err(other),
            }).collect();
        if let Ok(xs) = xs {
            Quaternion(xs.into_iter().fold(Quat(1.0, 0.0, 0.0, 0.0), Mul::mul))
        } else {
            Exception(Arity(0, 0))
        }
    }
}

/// `/ :: num ... -> num`
///
/// Produces the quotient of the two specified values.
pub fn div(args: &[Expression], _: &mut Context) -> Expression {
    match args.len() {
        0 => Exception(Custom(
            4,
            "arity mismatch: expected at least 1 argument, found 0".into(),
        )),
        1 => match &args[0] {
            Num(n) => Num(1.0 / n),
            other => Exception(Signature("num".into(), other.type_of())),
        },
        _ => match &args[0] {
            Num(head) => {
                let tail = &args[1..];
                let nums: Option<Vec<_>> = tail
                    .iter()
                    .map(|expr| match expr {
                        Num(n) => Some(*n),
                        _ => None,
                    }).collect();

                let res = nums
                    .map(|nums| nums.into_iter().fold(*head, Div::div))
                    .unwrap_or_else(|| *head);

                Num(res)
            }
            other => Exception(Signature("num".into(), other.type_of())),
        },
    }
}

/// `% :: num num -> num`
///
/// Produces the remainder of the two specified values.
pub fn rem(args: &[Expression], _: &mut Context) -> Expression {
    binary_fn(args, Rem::rem)
}

// Exceptions

/// `arity-exception :: num num -> exception`
///
/// Produces an arity exception with the specified parameters.
pub fn arity_exception(args: &[Expression], _: &mut Context) -> Expression {
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
pub fn cons(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [car, Cons(cdr)] => Cons(cdr.cons(car)),
        [a, b] => Exception(Signature(
            "any, cons".into(),
            format!("{}, {}", a.type_of(), b.type_of()).into(),
        )),
        xs => Exception(Arity(2, xs.len())),
    }
}

/// `head :: [a] -> a`
///
/// Produces the first element of the specified list.
pub fn head(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Cons(list)] => {
            list.head().map(|head| (*head).clone()).unwrap_or_else(|| {
                Exception(Custom(
                    10,
                    "cannot get the tail of an empty list".into(),
                ))
            })
        }
        _ => Exception(Signature("any, cons".into(), "not that".into())),
    }
}

/// `tail :: [a] -> [a]`
///
/// Produces the remainder of the specified list after the first element.
pub fn tail(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Cons(list)] => {
            list.tail().map(|tail| Cons(tail)).unwrap_or_else(|| {
                Exception(Custom(
                    11,
                    "cannot get the tail of an empty list".into(),
                ))
            })
        }
        _ => Exception(Signature("any, cons".into(), "not that".into())),
    }
}

/// `exit :: num -> nil`
///
/// Exits the program with the specified exit code.
pub fn exit(args: &[Expression], _: &mut Context) -> Expression {
    use std::process::exit;

    match args {
        [Num(code)] => {
            let code = *code as i32;
            exit(code);
        }
        [x] => Exception(Signature("num".into(), x.type_of())),
        [] => exit(0),
        args => Exception(Custom(
            4,
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
pub fn eq(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [a, b] => Bool(a == b),
        args => Exception(Arity(2, args.len())),
    }
}

/// `< :: a a -> bool`
///
/// Determines whether or not the first argument is less than the second.
pub fn lt(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b)] => Bool(a < b),
        [a, b] => Exception(Signature(
            "num, num".into(),
            format!("{}, {}", a, b).into(),
        )),
        args => Exception(Arity(2, args.len())),
    }
}

/// `<= :: a a -> bool`
///
/// Determines whether or not the first argument is less than or equal to the
/// second.
pub fn lte(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b)] => Bool(a <= b),
        [a, b] => Exception(Signature(
            "num, num".into(),
            format!("{}, {}", a, b).into(),
        )),
        args => Exception(Arity(2, args.len())),
    }
}

/// `> :: a a -> bool`
///
/// Determines whether or not the first argument is greater than the second.
pub fn gt(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b)] => Bool(a > b),
        [a, b] => Exception(Signature(
            "num, num".into(),
            format!("{}, {}", a, b).into(),
        )),
        args => Exception(Arity(2, args.len())),
    }
}

/// `>= :: a a -> bool`
///
/// Determines whether or not the first argument is greater than or equal to
/// the second.
pub fn gte(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b)] => Bool(a >= b),
        [a, b] => Exception(Signature(
            "num, num".into(),
            format!("{}, {}", a, b).into(),
        )),
        args => Exception(Arity(2, args.len())),
    }
}

/// `println :: a ... -> nil`
///
/// Prints the specified values, separated by spaces.
pub fn display(args: &[Expression], _: &mut Context) -> Expression {
    for arg in args {
        let fmt = match arg {
            Str(s) => s.to_string(),
            other => other.to_string(),
        };
        print!("{}", fmt);
    }
    stdout()
        .flush()
        .map_err(|_| Custom(12, "could not flush stdout".into()))
        .map(|_| Expression::default())
        .unwrap_or_else(|ex| Exception(ex))
}

/// `display-debug :: a ... -> nil`
///
/// Prints the specified values in debug mode, separated by spaces.
pub fn display_debug(args: &[Expression], _: &mut Context) -> Expression {
    for arg in args {
        let fmt = match arg {
            Str(s) => s.to_string(),
            other => other.to_string(),
        };
        print!("{:?}", fmt);
    }
    stdout()
        .flush()
        .map_err(|_| Custom(12, "could not flush stdout".into()))
        .map(|_| Expression::default())
        .unwrap_or_else(|ex| Exception(ex))
}

/// `newline :: -> nil`
///
/// Prints a new line.
pub fn newline(args: &[Expression], _: &mut Context) -> Expression {
    if args.len() == 0 {
        println!();
        Cons(ConsList::new())
    } else {
        Exception(Arity(0, args.len()))
    }
}

/// `append :: [a] ... -> [a]`
///
/// Append all specified lists to the first specified list.
pub fn append(args: &[Expression], _: &mut Context) -> Expression {
    // Try lists
    let lists: Option<Vec<_>> = args
        .iter()
        .map(|arg| match arg {
            Cons(list) => Some(list),
            _ => None,
        }).collect();

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
        }).collect();

    if let Some(strs) = strs {
        let len: usize = strs.iter().map(|s| s.len()).sum();
        let mut buf = String::with_capacity(len);
        for s in strs {
            buf.push_str(s);
        }
        return Str(buf.into());
    }

    Exception(Custom(13, "invalid types in append function".into()))
}

/// `empty? :: [a] -> bool`
///
/// Determines whether or not the specified list is empty.
pub fn empty(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Cons(list)] => Bool(list.is_empty()),
        [a] => Exception(Signature("(list a)".into(), a.type_of().into())),
        xs => Exception(Arity(1, xs.len())),
    }
}

/// `eval :: a -> b`
///
/// Evaluates the specified expression.
pub fn eval(args: &[Expression], ctx: &mut Context) -> Expression {
    match args {
        [ex] => ex.eval(ctx),
        xs => Exception(Arity(1, xs.len())),
    }
}

/// `readfile :: str -> str`
///
/// Reads the file with the specified name to a string.
pub fn readfile(args: &[Expression], _: &mut Context) -> Expression {
    fn read_file(name: &str) -> Result<String, io::Error> {
        let mut buf = String::new();

        let file = File::open(name)?;
        let mut reader = BufReader::new(file);

        reader.read_to_string(&mut buf)?;

        Ok(buf)
    }

    match args {
        [Str(s)] => read_file(s.as_ref())
            .map(|s| Str(s.into()))
            .unwrap_or_else(|_| {
                Exception(Custom(
                    14,
                    format!("could not read file {}", s).into(),
                ))
            }),
        [x] => Exception(Signature("str".into(), x.type_of())),
        xs => Exception(Arity(1, xs.len())),
    }
}

/// Attempts to read and parse the specified file, using the preprocessor as
/// needed.
fn load_file(file_name: impl AsRef<str>) -> Result<Expression, Box<Error>> {
    let file = File::open(file_name.as_ref())?;
    let mut reader = BufReader::new(file);

    let mut buf = String::new();

    reader.read_to_string(&mut buf)?;

    // Look for directive lines
    let mut use_preprocessor = false;
    {
        let iter = buf
            .lines()
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

    let removed_commands: String = buf
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    // println!("{}", removed_commands);
    let processed;
    let iter = match use_preprocessor {
        true => {
            let stripped = first_pass(removed_commands);
            processed = second_pass(stripped);
            processed.chars()
        }
        false => removed_commands.chars(),
    };

    let mut parser = Parser::new(iter);

    let mut exprs = Vec::new();
    while let Some(expr) = parser.parse_expr() {
        exprs.push(expr);
    }

    let expr = wrap_begin(exprs.into());
    Ok(expr)
}

/// `import :: string -> a`
///
/// Reads, parses, and runs the specified file, returning its result.
pub fn import(args: &[Expression], ctx: &mut Context) -> Expression {
    match args {
        [Str(file_name)] => {
            let res = load_file(file_name);
            res.map(|ex| ex.eval(ctx)).unwrap_or_else(|_| {
                Exception(Custom(
                    14,
                    format!("could not read file {}", file_name).into(),
                ))
            })
        }
        xs => Exception(Arity(1, xs.len())),
    }
}

/// `readline :: -> string`
///
/// Waits for the user to enter a line and returns the contents of the line.
pub fn readline(args: &[Expression], _: &mut Context) -> Expression {
    match args.len() {
        0 => {
            let mut buf = String::new();
            stdin()
                .read_line(&mut buf)
                .map_err(|_| Custom(15, "failed to read stdin".into()))
                .map(|_| Str(buf.trim().into()))
                .unwrap_or_else(|ex| Exception(ex))
        }
        n => Exception(Arity(0, n)),
    }
}

/// `parse :: string -> expr`
///
/// Parses the specified string as an expression.
pub fn parse(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Str(s)] => {
            let mut parser = Parser::new(s.chars());
            parser.parse_expr().unwrap_or_else(|| {
                Exception(Custom(16, format!("could not parse {}", s).into()))
            })
        }
        [x] => Exception(Signature("string".into(), x.type_of())),
        xs => Exception(Arity(1, xs.len())),
    }
}

/// `type-of :: a -> symbol`
///
/// Produces the type of the specified expression.
pub fn type_of(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [ex] => Symbol(ex.type_of()),
        xs => Exception(Arity(1, xs.len())),
    }
}

/// Stores data for splitting an interpolated string into its various parts.
#[derive(Debug)]
enum StrSection<'a> {
    /// A literal string.
    Literal(&'a str),

    // The contents of an interpolated string to be interpolated.
    Expr(&'a str),
}

/// Splits the specified string into a list of its literal and expression
/// parts.
fn split_str(s: &str) -> Result<Vec<StrSection>, Exception> {
    use self::StrSection::*;
    let mut strs = Vec::new();
    let mut in_expr = false;
    let mut last = 0usize;
    let mut i = 0usize;
    let mut layers = 0usize;
    let mut last_ch = '\0';
    for ch in s.chars() {
        const INTERPOLATION_CHAR: char = '#';
        match ch {
            '{' if last_ch == INTERPOLATION_CHAR => {
                strs.push(Literal(&s[last..i - 1]));
                in_expr = true;
                last = i + 1; // Begin expression after opening brace
            }
            '{' if in_expr => {
                layers += 1;
            }
            '}' if in_expr => {
                if layers == 0 {
                    strs.push(Expr(&s[last..i])); // Push section from expression
                    in_expr = false;
                    last = i + 1; // Begin next section after ending brace
                } else {
                    layers -= 1;
                }
            }
            _ => (),
        }
        i += 1;
        last_ch = ch;
    }
    if last != i {
        strs.push(Literal(&s[last..i]));
    }

    if in_expr {
        Err(Syntax(
            32,
            "unclosed expression while interpolating string".into(),
        ))
    } else {
        Ok(strs)
    }
}

/// Formats the specified string sections, evaluation the expression portions.
fn format_str(sections: &[StrSection], env: &mut Context) -> Expression {
    use self::StrSection::*;

    let mut buf = String::new();

    for section in sections {
        match section {
            Literal(s) => buf.push_str(s),
            Expr(s) => {
                let mut parser = Parser::new(s.chars());

                // Get contents
                let expr = parser.parse_expr();
                match expr {
                    Some(expr) => {
                        env.ascend_scope();
                        let res = expr.eval(env);
                        if res.is_exception() {
                            return res;
                        }
                        env.descend_scope();
                        let fmt = match res {
                            Str(s) => s.to_string(),
                            other => other.to_string(),
                        };
                        buf.push_str(&fmt);
                    }
                    None => {
                        return Exception(Syntax(
                            31,
                            "format string must contain expression to interpolate".into(),
                        ));
                    }
                }
            }
        }
    }

    Str(buf.into())
}

/// `format :: string -> string`
///
/// Interpolates all sections of the specified string enclosed with #{}. As an
/// example:
/// `(format "1 + 2 = #{(+ 1 2)}")`
///
/// Produces "1 + 2 = 3".
pub fn format(args: &[Expression], env: &mut Context) -> Expression {
    match args {
        [Str(s)] => match split_str(s.as_ref()) {
            Ok(sections) => format_str(&sections, env),
            Err(ex) => Exception(ex),
        },
        [x] => Exception(Signature("string".into(), x.type_of())),
        xs => Exception(Arity(1, xs.len())),
    }
}

/// `set :: symbol a -> nil`
///
/// Sets the value stored at the specified symbol to the specified value,
/// overriding any previous value.
pub fn set(args: &[Expression], env: &mut Context) -> Expression {
    match args {
        [Symbol(s), ex] => {
            if let Some(mut reference) = env.get_mut(s) {
                *reference = ex.clone();
                Expression::default()
            } else {
                Exception(Undefined(s.clone()))
            }
        }
        [x, _] => Exception(Signature("symbol".into(), x.type_of())),
        xs => Exception(Arity(2, xs.len())),
    }
}

/// `sqrt :: num -> num`
///
/// Produces the square root of the specified number.
pub fn sqrt(args: &[Expression], _: &mut Context) -> Expression {
    // unary_fn(args, f64::sqrt)
    match args {
        &[Num(n)] if n >= 0.0 => Num(f64::sqrt(n)),
        &[Num(n)] => Quaternion(Quat(0.0, f64::sqrt(-n), 0.0, 0.0)),
        [x] => Exception(Signature("num".into(), x.type_of())),
        xs => Exception(Arity(1, xs.len())),
    }
}

/// `sin :: num -> num`
///
/// Produces the spin of the specified number.
pub fn sin(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, f64::sin)
}

/// `sqrt :: num -> num`
///
/// Produces the cosine of the specified number.
pub fn cos(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, f64::cos)
}

/// `tan :: num -> num`
///
/// Produces the tangent of the specified number.
pub fn tan(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, f64::tan)
}

/// `csc :: num -> num`
///
/// Produces the cosecant of the specified number.
pub fn csc(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, |x| 1.0 / x.sin())
}

/// `sec :: num -> num`
///
/// Produces the secant of the specified number.
pub fn sec(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, |x| 1.0 / x.cos())
}

/// `cot :: num -> num`
///
/// Produces the cotangent of the specified number.
pub fn cot(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, |x| 1.0 / x.tan())
}

/// `asin :: num -> num`
///
/// Produces the arcsine of the specified number.
pub fn asin(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, f64::asin)
}

/// `acos :: num -> num`
///
/// Produces the arccosine of the specified number.
pub fn acos(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, f64::acos)
}

/// `atan :: num -> num`
///
/// Produces the arctangent of the specified number.
pub fn atan(args: &[Expression], _: &mut Context) -> Expression {
    unary_fn(args, f64::atan)
}

/// `atan2 :: num num -> num`
///
/// Produces the atan2 of the specified numbers.
pub fn atan2(args: &[Expression], _: &mut Context) -> Expression {
    binary_fn(args, f64::atan2)
}

/// `display-pretty :: symbol symbol str -> nil`
///
/// Prints the specified string using the specified formatting options.
pub fn display_pretty(args: &[Expression], _: &mut Context) -> Expression {
    fn get_style(style: impl AsRef<str>) -> Result<Style, Exception> {
        match style.as_ref() {
            "bold" => Ok(Style::Bold),
            "normal" => Ok(Style::Normal),
            other => {
                Err(Custom(35, format!("style not found: {}", other).into()))
            }
        }
    }

    fn get_color(color: impl AsRef<str>) -> Result<Option<Color>, Exception> {
        match color.as_ref() {
            "red" => Ok(Some(Color::Red)),
            "yellow" => Ok(Some(Color::Yellow)),
            "green" => Ok(Some(Color::Green)),
            "blue" => Ok(Some(Color::Blue)),
            "none" => Ok(None),
            other => {
                Err(Custom(34, format!("color not found: {}", other).into()))
            }
        }
    }

    match args {
        [Symbol(color), Symbol(style), Str(text)] => {
            match (get_color(color), get_style(style)) {
                (Ok(color), Ok(style)) => {
                    print_pretty(text, color, style);
                    Expression::default()
                }
                (Err(ex), _) => Exception(ex),
                (_, Err(ex)) => Exception(ex),
            }
        }
        [x, y, z] => Exception(Signature(
            "(symbol, symbol, any)".into(),
            format!("({}, {}, {})", x.type_of(), y.type_of(), z.type_of())
                .into(),
        )),
        xs => Exception(Arity(2, xs.len())),
    }
}

/// `quaternion :: num num num num -> quaternion`
pub fn quaternion(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a), Num(b), Num(c), Num(d)] => {
            Quaternion(Quat(a.clone(), b.clone(), c.clone(), d.clone()))
        }
        [a, b, c, d] => Exception(Signature(
            "(num, num, num, num)".into(),
            format!("({}, {}, {}, {})", a, b, c, d).into(),
        )),
        xs => Exception(Arity(4, xs.len())),
    }
}

pub fn exp(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a)] => Num(f64::exp(*a)),
        [Quaternion(q)] => Quaternion(q.exp()),
        [x] => Exception(Signature("num".into(), x.type_of())),
        xs => Exception(Arity(1, xs.len())),
    }
}

pub fn ln(args: &[Expression], _: &mut Context) -> Expression {
    match args {
        [Num(a)] => Num(f64::ln(*a)),
        [Quaternion(q)] => Quaternion(Quat::ln(q)),
        [x] => Exception(Signature("num".into(), x.type_of())),
        xs => Exception(Arity(1, xs.len())),
    }
}

pub fn env_var(args: &[Expression], _: &mut Context) -> Expression {
    use std::env;
    match args {
        [Str(s)] => {
            if let Ok(var) = env::var(s.as_ref()) {
                Str(var.into())
            } else {
                Exception(Custom(
                    36,
                    format!("undefined environment variable: \"{}\"", s).into(),
                ))
            }
        }
        [x] => Exception(Signature("str".into(), x.type_of())),
        xs => Exception(Arity(1, xs.len())),
    }
}
