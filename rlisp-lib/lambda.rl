(define zero (λ [f] (λ [x] 
    x)))

(define succ (λ [n] (λ [f] (λ [x] 
    (f ((n f) x))))))

; church n = \f -> \x -> f (church (n-1) f x)
(define int->church (λ [n] (λ [f] (λ [x]
    (if {n = 0}
        zero
        (f ((church->int {n - 1}) f) x))))))

; unchurch cn = cn (+ 1) 0
(define church->int (λ [n] 
    ((n (λ [x] {x + 1})) 0)))