use exception::Exception::*;
use expression::Expression::{self, *};
use im::ConsList;
use std::iter::Peekable;

#[macro_use]
use util::*;

pub mod preprocessor;

pub struct Parser<I>
where
    I: Iterator<Item = char>,
{
    iter: Peekable<I>,
    stack: Vec<char>,
    name: Option<String>,
    row: usize,
    col: usize,
}

impl<I> Parser<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            stack: Vec::new(),
            name: None,
            row: 1,
            col: 1,
        }
    }

    pub fn with_name(iter: I, name: String) -> Self {
        Self {
            iter: iter.peekable(),
            stack: Vec::new(),
            name: Some(name),
            row: 1,
            col: 1,
        }
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

    fn peek_char(&mut self) -> Option<char> {
        self.iter.peek().map(|&ch| ch)
    }

    fn unread(&mut self, ch: char) {
        self.stack.push(ch)
    }

    pub fn parse_expr(&mut self) -> Option<Expression> {
        // Ignore whitespace
        self.read_to(|ch| !ch.is_whitespace());

        // Look at char
        self.next_char().and_then(|ch| match ch {
            '\'' => self.parse_expr().map(|expr| Quote(Box::new(expr))),
            '(' => self.parse_cons(')'),
            '[' => self.parse_cons(']'),
            '#' => {
                let ex = self.parse_expr()?;
                let list = cons![Symbol("format".into()), ex];
                Some(Cons(list))
            }
            '"' => self.parse_str(),
            ')' | ']' => Some(Exception(Syntax(5, format!("illegal list close").into()))),
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
                                if let None = op {
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
        while let Some(ch) = self.next_char() {
            match ch {
                // Skip whitespace
                ch if ch.is_whitespace() => (),
                ch if ch == end => break,
                ch => {
                    self.unread(ch);
                    match self.parse_expr() {
                        Some(ref expr) if expr.is_exception() => return Some(expr.clone()),
                        Some(expr) => list = list + ConsList::singleton(expr),
                        _ => return Some(Exception(Syntax(6, "unclosed list".into()))),
                    }
                }
            }
        }
        Some(Cons(list))
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
                    "nil" | "empty" => Quote(Box::new(Cons(ConsList::new()))),
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
