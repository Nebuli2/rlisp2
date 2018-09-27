(define zero 
    (lambda [f] (lambda [x] 
        x)))

(define succ 
    (lambda [n] (lambda [f] (lambda [x] 
        (f ((n f) x))))))

; church n = \f -> \x -> f (church (n-1) f x)
; int->church :: num -> (num -> num) -> num
(define int->church 
    (lambda [n] (lambda [f] (lambda [x]
        (if {n = 0}
            zero
            (f ((church->int {n - 1}) f) x))))))

; church
(define church->int
    ; inc :: num -> num
    (define (inc x) 
        {x + 1})
    (lambda [n] 
        ((n (lambda [x] {x + 1})) 0)))