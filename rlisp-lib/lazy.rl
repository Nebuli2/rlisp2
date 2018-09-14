; (lazy a) = (-> a)

; lazy-fib :: (lazy num) -> (lazy num)
(define (lazy-fib ln)
    (lambda []
        (define n (ln))
        (cond [{n < 2} n]
              [else {((lazy-fib (lambda [] {n - 1}))) + ((lazy-fib (lambda [] {n - 2})))}])))

(define ln (lambda [] 20))
(define big-calc (lazy-fib ln))
(println big-calc)
(println (big-calc))