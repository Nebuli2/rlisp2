(define-macro (swap! a b)
    `(let ([tmp ,a])
        (set! ,a ,b)
        (set! ,b tmp)))

(define-macro (set! var val)
    `(set-internal! ',var ,val))