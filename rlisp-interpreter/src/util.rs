//! This module contains a set of utility functions and types that did not fit
//! into other modules.

use crate::{
    exception::Exception,
    expression::Expression::{self, *},
};
use im::ConsList;
use std::{io::prelude::*, rc::Rc};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[macro_export]
macro_rules! cons {
    [$($ex:expr),*] => ({
        use im::ConsList;

        ConsList::from(vec![$($ex),*])
    })
}

/// An immutable, reference-counted string.
pub type Str = Rc<str>;

/// Produces an expression equal to `nil`.
pub fn nil() -> Expression {
    Cons(ConsList::new())
}

/// Wraps the specified list of expressions in a `begin` statement. For
/// example, it will transform this:
/// ```rustlisp
/// '(1 2 3 4)
/// ```
/// into:
/// ```rustlisp
/// '(begin 1 2 3 4)
/// ```
pub fn wrap_begin(exprs: ConsList<Expression>) -> Expression {
    Cons(exprs.cons(Expression::Symbol("begin".into())))
}

/// Sets the color of `stdout` to the specified color. If `None` is provided,
/// the current color of `stdout` is cleared.
pub fn set_stdout_color(color: Option<Color>) {
    let mut sout = StandardStream::stdout(ColorChoice::Always);
    sout.set_color(ColorSpec::new().set_fg(color))
        .expect("failed to set terminal color");
}

/// Sets the color of `stdout` to red.
pub fn set_red() {
    set_stdout_color(Some(Color::Red));
}

/// Sets the color of `stdout` to green.
pub fn set_green() {
    set_stdout_color(Some(Color::Green));
}

/// Clears the color of `stdout`.
pub fn clear_color() {
    set_stdout_color(None);
}

fn print_err_no_ln(ex: &Exception) {
    let mut sout = StandardStream::stdout(ColorChoice::Always);
    sout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
        .expect("failed to set stdout color");
    write!(sout, "error({})", ex.error_code())
        .expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(true))
        .expect("failed to set stdout color");
    write!(sout, ": {}", ex).expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(false))
        .expect("failed to set stdout color");
}

/// Prints the specified exception in the following format:
/// ```rustlisp
/// error(<error_code>): <description>
/// ```
/// The initial part before the colon is printed in bold red text, and the
/// description in bold, uncolored text.
pub fn print_err(ex: &Exception) {
    print_err_no_ln(ex);
    println!("");
}

/// Prints the specified prompt in bold green text.
pub fn print_prompt(prompt: impl AsRef<str>) {
    let mut sout = StandardStream::stdout(ColorChoice::Always);
    sout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))
        .expect("failed to set stdout color");
    write!(sout, "{}", prompt.as_ref()).expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(false))
        .expect("failed to set stdout color");
}

/// The style options available for printing text.
pub enum Style {
    /// Bold text.
    Bold,
    /// Normal text.
    Normal,
}

/// Prints the specified text using the specified color and style options.
pub fn print_pretty(text: impl AsRef<str>, color: Option<Color>, style: Style) {
    let mut sout = StandardStream::stdout(ColorChoice::Always);
    let bold = match style {
        Style::Bold => true,
        Style::Normal => false,
    };
    sout.set_color(ColorSpec::new().set_fg(color).set_bold(bold))
        .expect("failed to set stdout color");
    write!(sout, "{}", text.as_ref()).expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(false))
        .expect("failed to set stdout color");
}

pub fn print_stack_trace(ex: &Exception) {
    let mut sout = StandardStream::stdout(ColorChoice::Always);
    let stack: Vec<_> = ex.stack().iter().collect();
    print_err_no_ln(ex);
    for (i, item) in stack.into_iter().rev().enumerate() {
        sout.set_color(ColorSpec::new().set_fg(None).set_bold(false))
            .unwrap();
        write!(sout, "\n at ").unwrap();
        sout.set_color(ColorSpec::new().set_fg(None).set_bold(true))
            .unwrap();
        write!(sout, "[{}]", i).unwrap();
        sout.set_color(ColorSpec::new().set_fg(None).set_bold(false))
            .unwrap();
        write!(sout, " {}", item).unwrap();
    }
    write!(sout, "\n").unwrap();
    clear_color();
}
