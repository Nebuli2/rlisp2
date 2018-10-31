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

pub mod preprocessor;

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
                        // Attempt to parse number
                        if let Ok(num) = s.parse::<f64>() {
                            Num(num)
                        } else {
                            Symbol(s.into())
                        }
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
