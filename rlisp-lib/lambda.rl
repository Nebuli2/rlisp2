(define zero 
    (lambda [f] (lambda [x] 
        x)))

(define succ 
    (lambda [n] (lambda [f] (lambda [x] 
        (f ((n f) x))))))

; church n = \f -> \x -> f (church (n-1) f x)
(define int->church 
    (lambda [n] (lambda [f] (lambda [x]
        (if {n = 0}
            zero
            (f ((church->int {n - 1}) f) x))))))

; unchurch cn = cn (+ 1) 0
(define church->int 
    (lambda [n] 
        ((n (lambda [x] {x + 1})) 0)))