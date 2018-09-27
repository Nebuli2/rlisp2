; (lazy a) = (-> a)

; lazy-fib :: (lazy num) -> (lazy num)
(define (fib* n*)
    (lambda []
        (define n (n*))
        (cond [{n < 2} n]
              [else {((fib* (lambda [] {n - 1}))) + ((fib* (lambda [] {n - 2})))}])))

(define ln (lambda [] 20))
(define big-calc (lazy-fib ln))
(println big-calc)
(println (big-calc))