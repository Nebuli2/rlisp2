(define foo (quasiquote (1 2 (unquote (+ 1 2)))))
(define bar `(1 2 ,(+ 1 2)))
(display foo)
(newline)
(display bar)
(newline)

(display (quote (quasiquote (1 2 (unquote (+ 1 2))))))
(newline)