//! This module provides intrinsic macros to the rlisp language. An intrinsic
//! macro is a function that acts on unevaluated arguments. This gives the
//! macro reign to do whatever it will with the arguments.

use rlisp_interpreter::{
    context::Context,
    exception::Exception,
    expression::{
        Callable::*,
        Expression::{self, *},
        LambdaData, StructData, ValidIdentifier,
    },
    im::ConsList,
    pattern::{pattern_match, replace_symbols},
    util::{nil, wrap_begin, Str},
};
use std::rc::Rc;

/// Creates a lambda with the specified parameters and body, capturing
/// variables from the specified context. At the time of creation.
fn create_lambda(
    params: ConsList<Expression>,
    body: ConsList<Expression>,
    ctx: &Context,
) -> Expression {
    let params: Result<ConsList<Str>, ()> = params
        .iter()
        .map(|param| match *param {
            Symbol(ref name) => Ok(name.clone()),
            _ => Err(()),
        })
        .collect();
    params
        .map(|params| {
            let body = if body.len() == 1 {
                body.head().map(|expr| expr.as_ref().clone())
            } else {
                Some(wrap_begin(body))
            }
            .unwrap_or_default();
            let mut capture = body.extract_symbols(ctx);
            if let Some(file) = ctx.get("__FILE__") {
                capture.insert("__FILE__".into(), file.clone());
            }
            let capture = Some(capture);
            Callable(Lambda(Rc::new(LambdaData {
                params,
                body: Rc::new(body.clone()),
                capture: capture.map(Rc::new),
            })))
        })
        .unwrap_or_else(|_| {
            Error(Rc::new(Exception::syntax(17, "(lambda [args...] body)")))
        })
}

/// `(lambda [<param1> ...] <body1> ...)`
///
/// Produces a `Lambda` with the specified parameters and body.
pub fn lambda(list: ConsList<Expression>, ctx: &mut Context) -> Expression {
    let params = list.tail().and_then(|list| list.head());
    let body = list.tail().and_then(|list| list.tail());

    match (params, body) {
        (Some(params), Some(body)) => match params.as_ref() {
            Cons(list) => create_lambda(list.clone(), body, ctx),
            _ => {
                Error(Rc::new(Exception::syntax(17, "(lambda [args...] body)")))
            }
        },
        _ => Error(Rc::new(Exception::syntax(17, "(lambda [args...] body)"))),
    }
}

/// `(define <ident> <value>) | (define (<ident> <param1> ...) <expr1> ...)`
///
/// Defines either a constant or a function with the specified name and value.
pub fn define(list: ConsList<Expression>, ctx: &mut Context) -> Expression {
    list.tail()
        .and_then(|list| list.head())
        .map(|head| (*head).clone())
        .ok_or_else(|| Exception::arity(2, list.len() - 1))
        .and_then(|head| match head {
            Symbol(ident) => {
                // Simple binding
                // Check arity
                match list.len() {
                    len if len == 3 => {
                        if ident.is_valid_identifier() {
                            // Safe to unwrap because we just checked the length
                            let value = list
                                .iter()
                                .nth(2)
                                .map(|expr| expr.eval(ctx))
                                .unwrap();
                            if let Error(ex) = value {
                                Err(ex.as_ref().clone())
                            } else {
                                ctx.insert(ident, value);
                                Ok(Expression::default())
                            }
                        } else {
                            Err(Exception::custom(
                                28,
                                format!("reserved identifier: {}", ident),
                            ))
                        }
                    }
                    len => {
                        // Arity mismatch
                        Err(Exception::arity(3, len))
                    }
                }
            }
            Cons(func) => {
                // Function binding
                let ident = func.head().map(|ident| (*ident).clone());
                ident
                    .ok_or_else(|| Exception::arity(2, 0))
                    .and_then(|ident| {
                        if let Symbol(ref s) = ident {
                            if s.is_valid_identifier() {
                                Ok(ident.clone())
                            } else {
                                Err(Exception::custom(
                                    28,
                                    format!("reserved identifier: {}", s),
                                ))
                            }
                        } else {
                            Err(Exception::signature("symbol", ident.type_of()))
                        }
                    })
                    .and_then(|ident| match ident {
                        Symbol(ident) => {
                            // Continue
                            let params = func.tail().unwrap_or_default();
                            let params: Result<ConsList<_>, _> = params
                                .iter()
                                .map(|param| match param.as_ref() {
                                    ident @ Symbol(..) => Ok(ident.clone()),
                                    _ => Err(Exception::syntax(
                                        27,
                                        "function parameters must be symbols",
                                    )),
                                })
                                .collect();
                            params.map(|params| {
                                let body =
                                    list.tail().and_then(|list| list.tail());
                                body.map(|body| {
                                    let lambda =
                                        create_lambda(params, body, ctx);
                                    ctx.insert(ident, lambda);
                                });
                                Expression::default()
                            })
                        }
                        _ => {
                            // Error, must have symbol as function identifier
                            Err(Exception::syntax(
                                25,
                                "value must be bound to a symbol",
                            ))
                        }
                    })
            }
            _ => Err(Exception::syntax(
                26,
                "define must bind either a function or a symbol",
            )),
        })
        .unwrap_or_else(|ex| Error(Rc::new(ex)))
}

/// `(env <ident>)`
///
/// Looks up the specified identifier in the current evaluation context.
pub fn env(list: ConsList<Expression>, ctx: &mut Context) -> Expression {
    let arg = list
        .tail()
        .and_then(|tail| tail.head())
        .map(|arg| arg.eval(ctx));
    arg.map(|arg| match arg {
        Symbol(ident) => {
            ctx.get(ident).map(|expr| expr.clone()).unwrap_or_else(nil)
        }
        _ => Error(Rc::new(Exception::signature("symbol", arg.type_of()))),
    })
    .unwrap_or_else(|| Error(Rc::new(Exception::arity(1, 99))))
}

/// `(if <cond> <then> <else>)`
///
/// If the condition is true, <then> is returned. Otherwise, <else> is
/// returned.
///
/// # Examples
/// ```rustlisp
/// (define x 10)
/// ; ...
/// (if (eq? x 10)
///     'ten
///     'not-ten)
/// ; Is equal to 'ten
/// ```
pub fn if_expr(list: ConsList<Expression>, ctx: &mut Context) -> Expression {
    let cond = list
        .tail()
        .and_then(|tail| tail.head())
        .map(|expr| expr.eval(ctx));
    let then_branch = list
        .tail()
        .and_then(|tail| tail.tail())
        .and_then(|tail| tail.head());
    let else_branch = list
        .tail()
        .and_then(|tail| tail.tail())
        .and_then(|tail| tail.tail())
        .and_then(|tail| tail.head());
    match (cond, then_branch, else_branch) {
        (Some(ex @ Error(_)), ..) => ex.clone(),
        (Some(Bool(cond)), Some(then_branch), Some(else_branch)) => {
            if cond {
                then_branch.eval(ctx)
            } else {
                else_branch.eval(ctx)
            }
        }
        (Some(a), Some(b), Some(c)) => Error(Rc::new(Exception::signature(
            "bool, any, any",
            format!("{}, {}, {}", a.type_of(), b.type_of(), c.type_of()),
        ))),
        _ => Error(Rc::new(Exception::arity(3, list.len()))),
    }
}

/// `(cond [<pred> <expr>] ...)`
///
/// Iterates through the predicates until one evaluaes to true. That
/// predicate's matching value is returned.
///
/// # Examples
/// ```rustlisp
/// (define x 10)
/// ; ...
/// (cond [(eq? x 5) 'five]
///       [(eq? x 10) 'ten]
///       [else 'other])
/// ; Is equal to 'ten
/// ```
pub fn cond(list: ConsList<Expression>, ctx: &mut Context) -> Expression {
    ctx.ascend_scope();

    // Ensure that "else" branch works
    ctx.insert("else", Bool(true));

    let branches = list.tail().unwrap_or_else(|| ConsList::new());
    for branch in branches.iter() {
        match branch.as_ref() {
            Cons(list) if list.len() == 2 => {
                let cond = list.head();
                let value = list.tail().and_then(|tail| tail.head());

                match (cond, value) {
                    (Some(cond), Some(value)) => match cond.eval(ctx) {
                        ex @ Error(_) => return ex.clone(),
                        Bool(false) => (),
                        Bool(true) => {
                            ctx.descend_scope();
                            return value.eval(ctx);
                        }
                        _ => {
                            ctx.descend_scope();
                            return Error(Rc::new(Exception::syntax(
                                18,
                                "condition must be a boolean value",
                            )));
                        }
                    },
                    _ => {
                        ctx.descend_scope();
                        return Error(Rc::new(Exception::syntax(
                            19,
                            "condition case must contain 2 elements",
                        )));
                    }
                }
            }
            _ => {
                ctx.descend_scope();
                return Error(Rc::new(Exception::syntax(
                    20,
                    "condition case must be a list",
                )));
            }
        }
    }

    ctx.descend_scope();
    Expression::default()
}

/// `(let ([<name> <value>] ...) <expr> ...)`
///
/// Binds the specified values to the specified identifiers, creating a new
/// context, and evaluating the specified body expressions in that new context.
///
/// # Examples
/// ```rustlisp
/// (let ([x 1]
///       [y (+ x 1)])
///     (+ x y))
/// ; Is equal to 3
/// ```
pub fn let_expr(list: ConsList<Expression>, ctx: &mut Context) -> Expression {
    let bindings = list.tail().and_then(|tail| tail.head());
    let body = list.tail().and_then(|list| list.tail());

    ctx.ascend_scope();
    let bindings = bindings
        .ok_or_else(|| Exception::arity(2, 0))
        .and_then(|bindings| match bindings.as_ref().clone() {
            Cons(bindings_list) => Ok(bindings_list),
            _ => Err(Exception::syntax(21, "binding list must be a list of bindings")), // Better error handling than none
        }).and_then(|bindings| {
            for binding in bindings.iter() {
                match binding.as_ref() {
                    Cons(binding) if binding.len() == 2 => {
                        // Proper binding here
                        // Unwrap is safe here as we have already checked the length
                        let ident = binding.head().unwrap();
                        let value = binding.tail().and_then(|x| x.head()).unwrap();

                        match ident.as_ref() {
                            Symbol(ident) => {
                                let value = value.eval(ctx);
                                if !value.is_exception() {
                                    ctx.insert(ident, value);
                                }
                            }
                            other => {
                                return Err(Exception::syntax(
                                    22,
                                    format!(
                                        "identifier in binding must be a symbol, found {}",
                                        other
                                    ),
                                ))
                            }
                        }
                    }
                    Cons(list) => return Err(Exception::arity(2, list.len())),
                    other => {
                        return Err(Exception::syntax(
                            23,
                            format!(
                                "binding must be a list containing a symbol and a value, found {}",
                                other
                            ),
                        ))
                    }
                }
            }
            Ok(())
        });

    let body = bindings
        .and(body.ok_or_else(|| Exception::syntax(24, "let body not found")))
        .map(|body| match body.len() {
            1 => body.head().unwrap().as_ref().clone(),
            _ => wrap_begin(body),
        })
        .map(|body| body.eval(ctx));
    ctx.descend_scope();
    body.unwrap_or_else(|ex| Error(Rc::new(ex)))
}

/// `(try <expr> <handler>)`
///
/// Attempts to evaluate the specified expression. If an exception is thrown,
/// the specified handler is called with data on the exception as its argument.
pub fn try_expr(list: ConsList<Expression>, ctx: &mut Context) -> Expression {
    // Check arity
    match list.len() - 1 {
        2 => {
            let expr = list.iter().nth(1).unwrap();
            let handler = list.iter().nth(2).unwrap().eval(ctx);

            if handler.is_callable() {
                let expr = expr.eval(ctx);
                if let Error(ex) = expr {
                    let description = ex.to_string();
                    let expr = Struct(Rc::new(StructData {
                        name: "error".into(),
                        data: vec![
                            (ex.error_code() as f64).into(), // error-code
                            description.into(), // error-description
                            Cons(ex.stack().clone()), // error-stack
                        ],
                    }));
                    let handle_list = ConsList::from(vec![handler, expr]);
                    Cons(handle_list).eval(ctx)
                } else {
                    expr
                }
            } else {
                Error(Rc::new(Exception::custom(
                    2,
                    format!("not a callable value: `{}`", handler),
                )))
            }
        }
        n => Error(Rc::new(Exception::arity(2, n))),
    }
}

/// `(define-struct <name> [<field1> ...])`
///
/// Creates a custom struct type, providing a name and field names. A number of
/// functions are in turn created:
/// * `make-<name> :: a ... -> <name>`
/// The constructor for types of this struct.
/// * `is-<name>? :: a -> bool`
/// Determines whether or not a value is of the type of this struct.
/// * `<name>-<field> :: <name> -> a`
/// A "getter" for the specified field in the struct.
///
/// As an example, calling `(define-struct point [x y])` would create the
/// following functions:
/// * `make-point`
/// * `is-point?`
/// * `point-x`
/// * `point-y`
pub fn define_struct(
    list: ConsList<Expression>,
    env: &mut Context,
) -> Expression {
    match list.len() - 1 {
        2 => {
            // These are safe to unwrap as we just checked the length
            let name = list.iter().nth(1).unwrap().clone();

            let name_str;
            if let Symbol(s) = name.as_ref() {
                name_str = s;
            } else {
                return Error(Rc::new(Exception::signature(
                    "symbol",
                    name.type_of(),
                )));
            }

            let id;
            if let Some(id_inner) = env.define_struct(name_str) {
                id = id_inner;
            } else {
                return Error(Rc::new(Exception::custom(
                    31,
                    "could not define struct",
                )));
            }

            let members = list.iter().nth(2).unwrap();

            let members_symbols;
            if let Cons(list) = members.as_ref() {
                members_symbols = list;
            } else {
                return Error(Rc::new(Exception::signature(
                    "cons",
                    name.type_of(),
                )));
            }

            let mut member_names: Vec<Str> =
                Vec::with_capacity(members_symbols.len());
            for ex in members_symbols.iter() {
                match ex.as_ref() {
                    Symbol(member) => member_names.push(member.clone()),
                    other => {
                        return Error(Rc::new(Exception::signature(
                            "symbol",
                            other.type_of(),
                        )));
                    }
                }
            }

            // Create accessors
            for (i, member) in member_names.iter().enumerate() {
                let get = move |args: &[Expression], _: &mut Context| match args
                {
                    [Struct(data)] => {
                        let StructData { data, .. } = data.as_ref();
                        data.get(i).map(|expr| expr.clone()).unwrap_or_else(
                            || {
                                Error(Rc::new(Exception::custom(
                                    29,
                                    "struct does not contain specified field",
                                )))
                            },
                        )
                    }
                    // [x] => Error(Exception::signature(name_symbol.clone(), x.type_of())),
                    xs => Error(Rc::new(Exception::arity(1, xs.len()))),
                };
                let accessor = format!("{}-{}", name, member);
                env.insert(accessor.clone(), Callable(Intrinsic(Rc::new(get))));
            }

            // Create is-type function
            let check = move |args: &[Expression], env: &mut Context| match args
            {
                [Struct(data)] => {
                    let StructData { name, data: _ } = data.as_ref();
                    if let Some(struct_id) = env.get_struct_id(name) {
                        Bool(struct_id == id)
                    } else {
                        Bool(false)
                    }
                }
                _ => Bool(false),
            };
            let check_name = format!("is-{}?", name_str);
            env.insert(check_name, Callable(Intrinsic(Rc::new(check))));

            // Create constructor
            let member_count = member_names.len();
            let make = move |args: ConsList<Expression>,
                             env: &mut Context|
                  -> Expression {
                let arg_count = args.len() - 1;
                let arg_iter = args.iter().skip(1);
                let mut member_data = Vec::with_capacity(arg_count);
                for ex in arg_iter {
                    let res = ex.eval(env);
                    if res.is_exception() {
                        return res;
                    } else {
                        member_data.push(res);
                    }
                }

                let prefix_len = "make-".len();

                match arg_count {
                    n if n == member_count => {
                        let make_expr = args.head(); // We have checked the length already
                        match make_expr {
                            Some(expr) => match expr.as_ref() {
                                Symbol(ident) => {
                                    let (_, name) = ident.split_at(prefix_len);
                                    // let id = env.get_struct_id(name);
                                    let name: Str = name.into();
                                    let data = StructData {
                                        name,
                                        data: member_data,
                                    };
                                    Struct(Rc::new(data))
                                }
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    }
                    n => Error(Rc::new(Exception::arity(member_count, n))),
                }
            };
            let constructor = format!("make-{}", name_str);
            env.insert(constructor, Callable(Macro(Rc::new(make))));
            Expression::default()
        }
        n => Error(Rc::new(Exception::arity(2, n))),
    }
}

/// `(begin <expr> ...)`
///
/// Evalulates all provided expressions. The result of the last expression is
/// returned.
pub fn begin(list: ConsList<Expression>, env: &mut Context) -> Expression {
    let mut last_expr = Expression::default();
    for expr in list.tail().unwrap_or_else(ConsList::new) {
        let result = expr.eval(env);
        if result.is_exception() {
            return result;
        }
        last_expr = result;
    }
    last_expr
}

macro_rules! check_arity {
    ($expected:expr, $found:expr) => {{
        use rlisp_interpreter::exception::Exception;
        use rlisp_interpreter::expression::Expression::Error;

        if $found != $expected {
            return Error(Rc::new(Exception::arity($expected, $found)));
        }
    }};
}

trait GetUnwrap {
    type Item;

    fn get_unwrap(&self, n: usize) -> Self::Item;
}

impl<T> GetUnwrap for ConsList<T>
where
    T: Clone,
{
    type Item = T;

    fn get_unwrap(&self, n: usize) -> T {
        self.iter().nth(n).unwrap().as_ref().clone()
    }
}

pub fn define_syntax_rule(
    list: ConsList<Expression>,
    ctx: &mut Context,
) -> Expression {
    check_arity!(2, list.len() - 1);

    let pattern = list.get_unwrap(1);
    let body = list.get_unwrap(2);

    match pattern {
        Cons(ref pat) if pat.len() < 1 => {
            return Error(Rc::new(Exception::syntax(
                37,
                "macro definition must include a name",
            )));
        }
        Cons(pat) => {
            let name = match pat.iter().nth(0).unwrap().as_ref().clone() {
                Symbol(name) => name,
                _ => {
                    return Error(Rc::new(Exception::custom(
                        38,
                        "macro name must be a symbol",
                    )));
                }
            };

            let syntax = [name.clone()];
            let pattern = Cons(pat.clone());

            let defined_macro =
                move |list: ConsList<Expression>, ctx: &mut Context| {
                    match pattern_match(&syntax, &pattern, &Cons(list)) {
                        Ok(matches) => {
                            let replaced = replace_symbols(&body, &matches);
                            replaced.eval(ctx)
                        }
                        Err(ex) => Error(Rc::new(ex)),
                    }
                };
            let wrapped_macro = Callable(Macro(Rc::new(defined_macro)));
            ctx.insert(name, wrapped_macro);
            Expression::default()
        }
        _ => {
            Error(Rc::new(Exception::syntax(40, "syntax rule must be a list")))
        }
    }
}
