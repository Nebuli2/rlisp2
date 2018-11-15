//! This module provides a means of parsing text input into s-expressions. In
//! addition, it provides additional syntax that is not typical of lisp
//! dialects.
//!
//! As an example, infix function calls are allowed, provided they
//! are delimited with `'{'` and `'}``. Within an infix function call, every
//! other expression is considered to be the "function."

use crate::{
    exception::Exception::*,
    expression::{
        Callable::*,
        Expression::{self, *},
    },
    util::{nil, wrap_begin},
};
use im::ConsList;
use regex::Regex;

pub mod preprocessor;

const QUAT_REGEX_STR_ABCD: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-]?[0-9]+(\.[0-9]*)?)i([+-]?[0-9]+(\.[0-9]*)?)j([+-]?[0-9]+(\.[0-9]*)?)k";

const QUAT_REGEX_STR_AB: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-]?[0-9]+(\.[0-9]*)?)i";

const QUAT_REGEX_STR_AC: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-]?[0-9]+(\.[0-9]*)?)j";

const QUAT_REGEX_STR_AD: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-]?[0-9]+(\.[0-9]*)?)k";

const QUAT_REGEX_STR_BC: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)i([+-]?[0-9]+(\.[0-9]*)?)j";

const QUAT_REGEX_STR_BD: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)i([+-]?[0-9]+(\.[0-9]*)?)k";

const QUAT_REGEX_STR_CD: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)j([+-]?[0-9]+(\.[0-9]*)?)k";

const QUAT_REGEX_STR_ABC: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-]?[0-9]+(\.[0-9]*)?)i([+-]?[0-9]*(\.[0-9]*)?)j";

const QUAT_REGEX_STR_ABD: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-]?[0-9]+(\.[0-9]*)?)i([+-]?[0-9]*(\.[0-9]*)?)k";

const QUAT_REGEX_STR_ACD: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-]?[0-9]+(\.[0-9]*)?)j([+-]?[0-9]+(\.[0-9]*)?)k";

const QUAT_REGEX_STR_BCD: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)i([+-]?[0-9]+(\.[0-9]*)?)j([+-]?[0-9]+(\.[0-9]*)?)k";

const QUAT_REGEX_STR_B: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)i";

const QUAT_REGEX_STR_C: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)j";

const QUAT_REGEX_STR_D: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)k";

lazy_static! {
    static ref QUAT_REGEX_ABCD: Regex =
        Regex::new(QUAT_REGEX_STR_ABCD).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_AB: Regex =
        Regex::new(QUAT_REGEX_STR_AB).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_AC: Regex =
        Regex::new(QUAT_REGEX_STR_AC).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_AD: Regex =
        Regex::new(QUAT_REGEX_STR_AD).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_BC: Regex =
        Regex::new(QUAT_REGEX_STR_BC).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_BD: Regex =
        Regex::new(QUAT_REGEX_STR_BD).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_CD: Regex =
        Regex::new(QUAT_REGEX_STR_CD).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_ABC: Regex =
        Regex::new(QUAT_REGEX_STR_ABC).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_ABD: Regex =
        Regex::new(QUAT_REGEX_STR_ABD).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_ACD: Regex =
        Regex::new(QUAT_REGEX_STR_ACD).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_BCD: Regex =
        Regex::new(QUAT_REGEX_STR_BCD).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_B: Regex =
        Regex::new(QUAT_REGEX_STR_B).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_C: Regex =
        Regex::new(QUAT_REGEX_STR_C).expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_D: Regex =
        Regex::new(QUAT_REGEX_STR_D).expect("quaternion regex failed to compile");
}

/// Stores information regarding the current state of the parser, in particular
/// its progress within whatever it is parsing, and a stack of characters to be
/// re-read.
pub struct Parser<I>
where
    I: IntoIterator<Item = char>,
{
    iter: I::IntoIter,
    stack: Vec<char>,
}

use quat::Quat;
use std::str::FromStr;

#[derive(Debug)]
pub struct ParseQuatError;

impl FromStr for Quat {
    type Err = ParseQuatError;

    fn from_str(s: &str) -> Result<Quat, Self::Err> {
        if QUAT_REGEX_ABCD.is_match(s) {
            let caps = QUAT_REGEX_ABCD.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let b_str = caps.get(3).map_or("1", |m| m.as_str());
            let c_str = caps.get(5).map_or("1", |m| m.as_str());
            let d_str = caps.get(7).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_BCD.is_match(s) {
            let caps = QUAT_REGEX_BCD.captures(s).unwrap();
            let b_str = caps.get(1).map_or("1", |m| m.as_str());
            let c_str = caps.get(3).map_or("1", |m| m.as_str());
            let d_str = caps.get(5).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_BC.is_match(s) {
            let caps = QUAT_REGEX_BC.captures(s).unwrap();
            let b_str = caps.get(1).map_or("1", |m| m.as_str());
            let c_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_BD.is_match(s) {
            let caps = QUAT_REGEX_BD.captures(s).unwrap();
            let b_str = caps.get(1).map_or("1", |m| m.as_str());
            let d_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = 0.0;
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_CD.is_match(s) {
            let caps = QUAT_REGEX_CD.captures(s).unwrap();
            let c_str = caps.get(1).map_or("1", |m| m.as_str());
            let d_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = 0.0;
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_ABC.is_match(s) {
            let caps = QUAT_REGEX_ABC.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let b_str = caps.get(3).map_or("1", |m| m.as_str());
            let c_str = caps.get(5).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_ABD.is_match(s) {
            let caps = QUAT_REGEX_ABD.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let b_str = caps.get(3).map_or("1", |m| m.as_str());
            let d_str = caps.get(5).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = 0.0;
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_ACD.is_match(s) {
            let caps = QUAT_REGEX_ACD.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let c_str = caps.get(3).map_or("1", |m| m.as_str());
            let d_str = caps.get(5).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = 0.0;
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_AD.is_match(s) {
            let caps = QUAT_REGEX_AD.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let d_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = 0.0;
            let c = 0.0;
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_AC.is_match(s) {
            let caps = QUAT_REGEX_AC.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let c_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = 0.0;
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_AB.is_match(s) {
            let caps = QUAT_REGEX_AB.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let b_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = 0.0;
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_B.is_match(s) {
            let caps = QUAT_REGEX_B.captures(s).unwrap();
            let b_str = caps.get(1).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = 0.0;
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_C.is_match(s) {
            let caps = QUAT_REGEX_C.captures(s).unwrap();
            let c_str = caps.get(1).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = 0.0;
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_D.is_match(s) {
            let caps = QUAT_REGEX_D.captures(s).unwrap();
            let d_str = caps.get(1).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = 0.0;
            let c = 0.0;
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else  {
            Err(ParseQuatError)
        }
    }
}

impl<I> Parser<I>
where
    I: IntoIterator<Item = char>,
{
    /// Produces a new parser reading from the specified iterator.
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.into_iter(),
            stack: Vec::new(),
        }
    }

    /// Produces the next char in the parser, if it is present. Otherwise,
    /// `None` is produced.
    fn next_char(&mut self) -> Option<char> {
        let ch = if !self.stack.is_empty() {
            self.stack.pop()
        } else {
            self.iter.next()
        };

        ch
    }

    /// "Unreads" the specified character. Returning it to the stack of unread
    /// characters.
    fn unread(&mut self, ch: char) {
        self.stack.push(ch)
    }

    /// Parses all whitespace-separated expressions into a `begin` expression,
    /// such that all will be evaulated, and the last returned.
    pub fn parse_all(&mut self) -> Expression {
        let mut exprs = ConsList::new();
        while let Some(expr) = self.parse_expr() {
            if let ex @ Exception(_) = expr {
                return ex;
            } else {
                exprs = exprs + ConsList::singleton(expr);
            }
        }
        wrap_begin(exprs)
    }

    /// Parses the next expression in the parser, producing it or `None` if no
    /// expression is found.
    pub fn parse_expr(&mut self) -> Option<Expression> {
        // Ignore whitespace
        self.read_to(|ch| !ch.is_whitespace());

        // Look at char
        self.next_char().and_then(|ch| match ch {
            '\'' => self.parse_expr().map(quote),
            '`' => self.parse_expr().map(quasiquote),
            ',' => self.parse_expr().map(unquote),
            '(' => self.parse_cons(')'),
            '[' => self.parse_cons(']'),
            '#' => {
                let ex = self.parse_expr()?;
                let list = cons![Symbol("format".into()), ex];
                Some(Cons(list))
            }
            '"' => self.parse_str(),
            ')' | ']' | '}' => Some(Exception(Syntax(
                5,
                format!("unexpected list close").into(),
            ))),
            ';' => {
                self.read_to(|ch| ch == '\n');
                self.parse_expr()
            }
            '{' => self.parse_infix(),
            ch => {
                self.unread(ch);
                self.parse_atom()
            }
        })
    }

    fn parse_quat(&mut self) -> Option<Expression> {
        // buffer
        let mut buf = String::new();
        while let Some(ch) = self.next_char() {
            if ch.is_whitespace() {
                break;
            }
            buf.push(ch);
        }

        None
    }

    /// Parses an infix function list. Every other element of the list is
    /// considered to be the first element of the list. As an example:
    /// ```rustlisp
    /// {1 + 2 + 3 + 4}
    /// ```
    /// Is parsed equivalently to:
    /// ```rustlisp
    /// (+ 1 2 3 4)
    /// ```
    fn parse_infix(&mut self) -> Option<Expression> {
        let mut buf: Vec<Expression> = Vec::new();
        let mut is_op = false;
        let mut op: Option<Expression> = None;

        while let Some(ch) = self.next_char() {
            match ch {
                ch if ch.is_whitespace() => (),
                '}' => break,
                ch => {
                    self.unread(ch);
                    match self.parse_expr() {
                        Some(expr) => {
                            if is_op {
                                if op.is_none() {
                                    op = Some(expr);
                                } else {
                                    // Ensure that different operators are not used in infix lists
                                    if Some(expr) != op {
                                        return Some(Exception(Syntax(
                                            6,
                                            "infix list operators must be equal".into(),
                                        )));
                                    }
                                }
                            } else {
                                buf.push(expr);
                            }
                            is_op = !is_op;
                        }
                        None => {
                            return Some(Exception(Syntax(
                                7,
                                "unclosed infix list".into(),
                            )))
                        }
                    }
                }
            }
        }

        match buf.len() {
            0 => Some(Cons(ConsList::new())),
            1 => Some((&buf[0]).clone()),
            _ => Some(Cons(
                ConsList::from(buf).cons(op.expect("this should not fail")),
            )),
        }
    }

    /// Reads from the data source until a specified predicate is matched. All
    /// the data read is returned as a string.
    fn read_to(&mut self, predicate: impl Fn(char) -> bool) -> Option<String> {
        let mut buf = String::new();
        while let Some(ch) = self.next_char() {
            if predicate(ch) {
                self.unread(ch);
                break;
            } else {
                buf.push(ch)
            }
        }
        if buf.is_empty() {
            None
        } else {
            Some(buf)
        }
    }

    /// Parses a list of expressions until a specified end delimiter, usually
    /// `')'`, `']'`, or `'}'`, is reached.
    fn parse_cons(&mut self, end: char) -> Option<Expression> {
        let mut list = ConsList::new();
        let mut closed = false;
        while let Some(ch) = self.next_char() {
            match ch {
                // Skip whitespace
                ch if ch.is_whitespace() => (),
                ch if ch == end => {
                    closed = true;
                    break;
                }
                ch => {
                    self.unread(ch);
                    match self.parse_expr() {
                        Some(ref expr) if expr.is_exception() => {
                            return Some(expr.clone())
                        }
                        Some(expr) => list = list + ConsList::singleton(expr),
                        None => {
                            return Some(Exception(Syntax(
                                6,
                                "unclosed list".into(),
                            )))
                        }
                    }
                }
            }
        }
        if closed {
            Some(Cons(list))
        } else {
            Some(Exception(Syntax(6, "unclosed list".into())))
        }
    }

    /// Parses a string.
    fn parse_str(&mut self) -> Option<Expression> {
        let mut buf = String::new();
        while let Some(ch) = self.next_char() {
            match ch {
                '\\' => match self.next_char() {
                    Some(ch) => match ch {
                        'r' => buf.push('\r'),
                        'n' => buf.push('\n'),
                        't' => buf.push('\t'),
                        ch => buf.push(ch),
                    },
                    None => (),
                },
                '"' => return Some(Str(buf.into())),
                ch => buf.push(ch),
            }
        }
        Some(Exception(Syntax(8, "unclosed string literal".into())))
    }

    /// Parses an atom, which is a boolean value, quote, quasiquote, unquote, a
    /// number, or a symbol.
    fn parse_atom(&mut self) -> Option<Expression> {
        self.read_to(|ch| ch.is_whitespace() || !is_valid_ident(ch))
            .map(|s| {
                match s.as_str() {
                    "#t" | "true" => Bool(true),
                    "#f" | "false" => Bool(false),
                    "nil" | "empty" => nil(),
                    "quote" => Callable(Quote),
                    "quasiquote" => Callable(Quasiquote),
                    "unquote" => Callable(Unquote),
                    _ => {
                        // Attempt to parse quaternion
                        if let Ok(q) = s.parse::<Quat>() {
                            return Quaternion(q);
                        }

                        // Attempt to parse number
                        if let Ok(num) = s.parse::<f64>() {
                            return Num(num);
                        }

                        Symbol(s.into())
                    }
                }
            })
    }
}

/// Determines whether or not the specified character is a valid identifier.
fn is_valid_ident(ch: char) -> bool {
    match ch {
        '(' | ')' | '[' | ']' | '{' | '}' | '\'' | '"' | '`' | ',' => false,
        _ => true,
    }
}

/// Wraps the specified expression in a quote. As an example:
/// ```rustlisp
/// 'foo
/// ```
/// Is transformed into:
/// ```rustlisp
/// (quote foo)
/// ```
fn quote(expr: Expression) -> Expression {
    let list: ConsList<_> = [Callable(Quote), expr].into_iter().collect();
    Cons(list)
}

/// Wraps the specified expression in a quasiquote. As an example:
/// ```rustlisp
/// `(1 2 ,(+ 1 2))
/// ```
/// Is transformed into:
/// ```rustlisp
/// (quasiquote (1 2 (unquote (+ 1 2)))
/// ```
fn quasiquote(expr: Expression) -> Expression {
    let list: ConsList<_> = [Callable(Quasiquote), expr].into_iter().collect();
    Cons(list)
}

/// Wraps the specified in an unquote. As en example:
/// ```rustlisp
/// ,foo
/// ```
/// Is transformed into:
/// ```rustlisp
/// (unquote foo)
/// ```
fn unquote(expr: Expression) -> Expression {
    let list: ConsList<_> = [Callable(Unquote), expr].into_iter().collect();
    Cons(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list() {
        let input = "(1 2 3)".chars();
        let mut parser = Parser::new(input);
        let found = parser.parse_expr();
        let expected = Some(Expression::Cons(
            ConsList::new()
                .cons(Expression::Num(3.0))
                .cons(Expression::Num(2.0))
                .cons(Expression::Num(1.0)),
        ));
        assert_eq!(&found, &expected);

        let input = "( 1 2 3 )".chars();
        let mut parser = Parser::new(input);
        let found = parser.parse_expr();
        let expected = Some(Expression::Cons(
            ConsList::new()
                .cons(Expression::Num(3.0))
                .cons(Expression::Num(2.0))
                .cons(Expression::Num(1.0)),
        ));
        assert_eq!(&found, &expected);
    }

    #[test]
    fn test_parse_num() {
        let input = "4.73".chars();
        let mut parser = Parser::new(input);
        let found = parser.parse_expr();
        let expected = Some(Expression::Num(4.73));
        assert_eq!(&found, &expected);
    }

    #[test]
    fn test_parse_str() {
        let input = "\"Hello, world!\"".chars();
        let mut parser = Parser::new(input);
        let found = parser.parse_expr();
        let expected = Some(Expression::Str("Hello, world!".into()));
        assert_eq!(&found, &expected);
    }
}
