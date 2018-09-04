use exception::Exception;
use expression::Expression;
use im::ConsList;
use std::{io::prelude::*, rc::Rc};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub type Str = Rc<str>;

pub fn nil() -> Expression {
    Expression::Cons(ConsList::new())
}

pub fn wrap_begin(exprs: ConsList<Expression>) -> Expression {
    Expression::Cons(exprs.cons(Expression::Symbol("begin".into())))
}

pub fn set_stdout_color(color: Option<Color>) {
    let mut sout = StandardStream::stdout(ColorChoice::Always);
    sout.set_color(ColorSpec::new().set_fg(color))
        .expect("failed to set terminal color");
}

pub fn set_red() {
    set_stdout_color(Some(Color::Red));
}

pub fn set_green() {
    set_stdout_color(Some(Color::Green));
}

pub fn clear_color() {
    set_stdout_color(None);
}

pub fn print_err(ex: &Exception) {
    let mut sout = StandardStream::stdout(ColorChoice::Always);
    sout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
        .expect("failed to set stdout color");
    write!(sout, "error[{:02}]: ", ex.error_code()).expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(true))
        .expect("failed to set stdout color");
    write!(sout, "{}\n", ex).expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(false))
        .expect("failed to set stdout color");
}
