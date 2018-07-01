use context::Context;
use im::ConsList;
use std::fmt;

#[derive(Clone)]
pub enum Expression {
    Bool(bool),
    Num(f64),
    Str(String),
    Symbol(String),
    Variable(String),

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

use self::Expression::*;

impl Expression {
    pub fn eval(&self, ctx: &mut Context) -> Expression {
        match self {
            // Look up variable
            Variable(ident) => ctx.get(ident)
                .map(|expr| expr.clone())
                .unwrap_or_else(|| Error(format!("undefined symbol {}", ident))),

            // Evaluate function
            Cons(list) => {
                if let Some(func) = list.head() {
                    let func = func.eval(ctx);
                    match func {
                        Macro(f) => f(self.clone()),
                        Intrinsic(f) => {
                            let args: Vec<_> = list.tail()
                                .unwrap_or_else(|| ConsList::new())
                                .iter()
                                .map(|expr| expr.eval(ctx))
                                .collect();
                            f(&args)
                        }
                        Lambda(params, body) => eval_lambda(
                            &params, 
                            &body, 
                            list.tail().unwrap_or_else(|| ConsList::new()), 
                            ctx
                        ),
                        _ => Error(format!("not a callable value"))
                    }
                } else {
                    Error("no function to call".to_string())
                }
            }
            
            // Otherwise just clone the value
            _ => self.clone() 
        }
    }
}

fn eval_lambda(params: &[String], body: &Expression, args: ConsList<Expression>, ctx: &mut Context) -> Expression {
    // Check arity
    match (params.len(), args.len()) {
        (a, b) if a == b => {
            ctx.ascend_scope();
            for (param, arg) in params.iter().zip(args.iter()) {
                ctx.insert(param.to_string(), (*arg).clone());
            }
            let res = body.eval(ctx);
            ctx.descend_scope();
            res
        }
        (a, b) => Expression::Error(format!("arity mismatch: expected {}, found {}", a, b))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bool(b) => write!(f, "{}", b)?,
            Num(n) => write!(f, "{}", n)?,
            Str(s) => write!(f, "\"{}\"", s)?,
            Symbol(s) => write!(f, "'{}", s)?,
            Variable(s) => write!(f, "'{}", s)?,
            Cons(list) => {
                
            },
            Lambda(..) => write!(f, "<lambda>")?,
            Intrinsic(..) => write!(f, "<intrinsic>")?,
            Macro(..) => write!(f, "<macro>")?,
            Error(s) => write!(f, "{}", s)?
        }
        Ok(())
    }
}
