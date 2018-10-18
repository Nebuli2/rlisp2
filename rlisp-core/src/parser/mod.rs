use exception::Exception::*;
use expression::Callable::*;
use expression::Expression::{self, *};
use im::ConsList;
use std::rc::Rc;
use util::{nil, wrap_begin};

pub mod preprocessor;

pub struct Parser<I>
where
    I: IntoIterator<Item = char>,
{
    iter: I::IntoIter,
    stack: Vec<char>,
    name: Option<String>,
    row: usize,
    col: usize,
}

impl<I> Parser<I>
where
    I: IntoIterator<Item = char>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.into_iter(),
            stack: Vec::new(),
            name: None,
            row: 1,
            col: 1,
        }
    }

    pub fn with_name(iter: I, name: String) -> Self {
        let mut parser = Parser::new(iter);
        parser.name = Some(name);
        parser
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = if !self.stack.is_empty() {
            self.stack.pop()
        } else {
            self.iter.next()
        };

        if let Some(ch) = ch {
            if ch == '\n' {
                self.row += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }

        ch
    }

    fn unread(&mut self, ch: char) {
        self.stack.push(ch)
    }

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
                        None => return Some(Exception(Syntax(7, "unclosed infix list".into()))),
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

        // Some(
        //     op.map(|op| Cons(ConsList::from(buf).cons(op)))
        //         .unwrap_or_else(|| match buf.len() {
        //             0 => Cons(ConsList::new()),
        //             1 => (&buf[0]).clone(),
        //             _ => Cons(ConsList::new()),
        //         }),
        // )
    }

    pub fn read_to(&mut self, predicate: impl Fn(char) -> bool) -> Option<String> {
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

    pub fn parse_cons(&mut self, end: char) -> Option<Expression> {
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
                        Some(ref expr) if expr.is_exception() => return Some(expr.clone()),
                        Some(expr) => list = list + ConsList::singleton(expr),
                        None => return Some(Exception(Syntax(6, "unclosed list".into()))),
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

    pub fn parse_str(&mut self) -> Option<Expression> {
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

    pub fn parse_atom(&mut self) -> Option<Expression> {
        self.read_to(|ch| ch.is_whitespace() || !is_valid_ident(ch))
            .map(|s| {
                // println!("{}", s);
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

// fn is_valid_ident(ch: char) -> bool {
//     match ch {
//         | 'a'..='z'
//         | 'A'..='Z'
//         | '0'..='9'
//         | '!'
//         | '@'
//         | '#'
//         | '$'
//         | '%'
//         | '^'
//         | '&'
//         | '*'
//         | '-'
//         | '_'
//         | '+'
//         | '='
//         | '/'
//         | '?'
//         | ':'
//         | ';'
//         | '>'
//         | '<'
//         | '.' => true,
//         _ => false,
//     }
// }

/// Determines whether or not the specified character is a valid identifier.
fn is_valid_ident(ch: char) -> bool {
    match ch {
        '(' | ')' | '[' | ']' | '{' | '}' | '\'' | '"' | '`' | ',' => false,
        _ => true,
    }
}

fn quote(expr: Expression) -> Expression {
    let list: ConsList<_> = [Callable(Quote), expr].into_iter().collect();
    Cons(list)
}

fn quasiquote(expr: Expression) -> Expression {
    let list: ConsList<_> = [Callable(Quasiquote), expr].into_iter().collect();
    Cons(list)
}

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
