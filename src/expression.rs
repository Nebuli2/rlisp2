use context::Context;
use im::ConsList;

#[derive(Clone)]
pub enum Expression {
    Bool(bool),
    Num(f64),
    Str(String),
    Symbol(String),

    Cons(ConsList<Expression>),

    Lambda(Vec<String>, Box<Expression>),

    // Represents an intrinsic function, taking a slice of expressions and
    // returning another expression.
    Intrinsic(fn(&[Expression]) -> Expression),

    // Represents a macro that transforms the expression into a new expression.
    Macro(fn(Expression) -> Expression),

    // Represents an exception
    Error(String),
}

impl Expression {
    pub fn eval(self, ctx: Context) -> (Expression, Context) {
        (self, ctx)
    }
}

fn eval_lambda(lambda: Expression, args: Vec<Expression>, ctx: Context) -> (Expression, Context) {
    use self::Expression::*;

    // Check that it's a lambda
    if let Lambda(params, body) = lambda {
        // Check arity
        match (params.len(), args.len()) {
            (a, a) => {}
            (a, b) => (Error(format!("arity mismath: expected {}, found {}", a, b))),
        }
    } else {
        (Error("not a lambda".to_string()), ctx)
    }
    // Check arity

    // let ctx = ctx.new_scope();
}

// impl Value {
//     pub fn eval(self) -> Expression {
//         use Self::*;
//         match self {
//             Cons(cons) => {
//                 // Extract out the first symbol; call as function

//                 // ...
//             },
//             other => other
//         }
//     }
// }
