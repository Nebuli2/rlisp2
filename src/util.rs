use expression::Expression;
use im::ConsList;
use std::rc::Rc;
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
