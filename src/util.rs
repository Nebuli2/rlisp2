use expression::Expression;
use im::ConsList;
use std::rc::Rc;

pub type Str = Rc<str>;

pub fn nil() -> Expression {
    Expression::Cons(ConsList::new())
}

pub fn wrap_begin(exprs: ConsList<Expression>) -> Expression {
    Expression::Cons(exprs.cons(Expression::Symbol("begin".into())))
}
