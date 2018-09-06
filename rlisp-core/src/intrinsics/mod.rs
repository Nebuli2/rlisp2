use context::Context;
use expression::Expression;
use im::ConsList;
use std::rc::Rc;

pub mod functions;
pub mod macros;

pub fn load(ctx: &mut Context) {
    load_macros(ctx);
    load_functions(ctx);
}

fn define_intrinsic(
    ctx: &mut Context,
    ident: impl ToString,
    f: impl Fn(&[Expression], &mut Context) -> Expression + 'static,
) {
    ctx.insert(ident.to_string(), Expression::Intrinsic(Rc::new(f)));
}

fn define_macro(
    ctx: &mut Context,
    ident: impl ToString,
    f: impl Fn(ConsList<Expression>, &mut Context) -> Expression + 'static,
) {
    ctx.insert(ident.to_string(), Expression::Macro(Rc::new(f)));
}

fn load_macros(ctx: &mut Context) {
    define_macro(ctx, "define", macros::_define);
    define_macro(ctx, "lambda", macros::_lambda);
    define_macro(ctx, "λ", macros::_lambda);
    define_macro(ctx, "env", macros::_env);
    define_macro(ctx, "if", macros::_if);
    define_macro(ctx, "cond", macros::_cond);
    define_macro(ctx, "quote", macros::_quote);
    define_macro(ctx, "let", macros::_let);
    define_macro(ctx, "try", macros::_try);
    define_macro(ctx, "define-struct", macros::_define_struct);
}

fn load_functions(ctx: &mut Context) {
    // Mathematical functions
    define_intrinsic(ctx, "+", functions::_add);
    define_intrinsic(ctx, "-", functions::_sub);
    define_intrinsic(ctx, "*", functions::_mul);
    define_intrinsic(ctx, "/", functions::_div);
    define_intrinsic(ctx, "%", functions::_rem);
    define_intrinsic(ctx, "eq?", functions::_eq);
    define_intrinsic(ctx, "=", functions::_eq);
    define_intrinsic(ctx, ">", functions::_gt);
    define_intrinsic(ctx, ">=", functions::_gte);
    define_intrinsic(ctx, "<", functions::_lt);
    define_intrinsic(ctx, "<=", functions::_lte);

    // Boolean logic
    define_intrinsic(ctx, "and", functions::_and);
    define_intrinsic(ctx, "or", functions::_or);

    define_intrinsic(ctx, "&&", functions::_and);
    define_intrinsic(ctx, "||", functions::_or);

    // Lists
    define_intrinsic(ctx, "cons", functions::_cons);
    define_intrinsic(ctx, ":", functions::_cons);
    define_intrinsic(ctx, "head", functions::_head);
    define_intrinsic(ctx, "tail", functions::_tail);

    define_intrinsic(ctx, "exit", functions::_exit);
    define_intrinsic(ctx, "begin", functions::_begin);
    define_intrinsic(ctx, "display", functions::_display);
    define_intrinsic(ctx, "newline", functions::_newline);
    define_intrinsic(ctx, "readline", functions::_readline);

    define_intrinsic(ctx, "++", functions::_append);
    define_intrinsic(ctx, "append", functions::_append);
    define_intrinsic(ctx, "empty?", functions::_empty);

    define_intrinsic(ctx, "eval", functions::_eval);
    define_intrinsic(ctx, "import", functions::_import);
    define_intrinsic(ctx, "parse", functions::_parse);

    define_intrinsic(ctx, "type-of", functions::_type_of);
    define_intrinsic(ctx, "format", functions::_format);
}
