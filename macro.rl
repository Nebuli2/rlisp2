(define-macro (square x)
    `(* x x))

(define-macro (make-square-named name)
    `(define (,name x)
        (* x x)))