(define-macro (set2! var val)
    `(set! ',var ,val))