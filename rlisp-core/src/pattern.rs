use crate::{
    exception::Exception::{self, *},
    expression::Expression::{self, *},
    util::Str,
};
use im::ConsList;
use std::collections::HashMap;

type Matches = HashMap<Str, Expression>;

enum Value {
    Expression(Expression),
    Variadic(ConsList<Expression>),
}

const ELLIPSIS: &str = "...";

pub fn pattern_match(
    syntax: &[Str],
    pattern: &Expression,
    input: &Expression,
) -> Result<Matches, Exception> {
    let mut matches = HashMap::new();
    extract_matches(syntax, pattern, input, &mut matches)?;
    Ok(matches)
}

pub fn replace_symbols(expr: &Expression, matches: &Matches) -> Expression {
    match expr {
        Symbol(s) => match matches.get(s) {
            Some(val) => val.clone(),
            None => Symbol(s.clone()),
        },
        Cons(list) => Cons(
            list.iter()
                .map(|expr| replace_symbols(expr.as_ref(), matches))
                .collect(),
        ),
        other => other.clone(),
    }
}

pub fn extract_symbols(syntax: &[Str], expr: &Expression) -> Vec<Str> {
    let mut buf = Vec::new();
    extract_symbols_to(syntax, expr, &mut buf);
    buf
}

fn extract_symbols_to(syntax: &[Str], expr: &Expression, to: &mut Vec<Str>) {
    match expr {
        Symbol(s) if !syntax.contains(s) => to.push(s.clone()),
        Cons(xs) => {
            for expr in xs.iter() {
                extract_symbols_to(syntax, expr.as_ref(), to);
            }
        }
        _ => {}
    }
}

fn extract_matches(
    syntax: &[Str],
    pattern: &Expression,
    input: &Expression,
    to: &mut Matches,
) -> Result<(), Exception> {
    match (pattern, input) {
        // Check if it's a syntax symbol
        (Symbol(s1), Symbol(s2)) if syntax.contains(s1) && s1 == s2 => {}

        // Bind value to symbol
        (Symbol(s), expr) => {
            to.insert(s.clone(), expr.clone());
        }

        // Handle lists
        (Cons(l1), Cons(l2)) if l1.len() == l2.len() => {
            // Handle lists
            for (pat, found) in l1.iter().zip(l2.iter()) {
                extract_matches(syntax, pat.as_ref(), found.as_ref(), to)?;
            }
        }

        // Ignore if we matched a literal value
        (x, y) if x == y => {}

        // Otherwise it isn't a match; fail
        (x, y) => {
            return Err(Custom(
                42,
                format!(
                    "pattern match failure: expected `{}`, found `{}`",
                    x, y
                ).into(),
            ));
        }
    }

    Ok(())
}
