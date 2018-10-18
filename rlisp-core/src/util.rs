use exception::Exception;
use expression::{
    Callable::*,
    Expression::{self, *},
};
use im::ConsList;
use std::{io::prelude::*, rc::Rc};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

macro_rules! cons {
    [$($ex:expr),*] => ({
        use im::ConsList;

        ConsList::from(vec![$($ex),*])
    })
}

pub type Str = Rc<str>;

pub fn nil() -> Expression {
    let list =
        ConsList::singleton(Callable(Quote)).append(ConsList::singleton(Cons(ConsList::new())));
    Cons(list)
}

pub fn wrap_begin(exprs: ConsList<Expression>) -> Expression {
    Cons(exprs.cons(Expression::Symbol("begin".into())))
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
    write!(sout, "error({:02})", ex.error_code()).expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(true))
        .expect("failed to set stdout color");
    write!(sout, ": {}\n", ex).expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(false))
        .expect("failed to set stdout color");
}

pub fn print_prompt(prompt: impl AsRef<str>) {
    let mut sout = StandardStream::stdout(ColorChoice::Always);
    sout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))
        .expect("failed to set stdout color");
    write!(sout, "{}", prompt.as_ref()).expect("failed to write to stdout");
    sout.set_color(ColorSpec::new().set_fg(None).set_bold(false))
        .expect("failed to set stdout color");
}

pub enum Style {
    Bold,
    Normal,
}

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
