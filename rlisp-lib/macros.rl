; Swaps the two specified variables.
(define-macro-rule (swap! a b)
    (let ([tmp b])
        (set! b a)
        (set! a tmp)))

(define-macro-rule (set! var val)
    (set-internal! 'var val))