(define (fibonacci n)
  (if {n < 2}
    n
    {(fibonacci {n - 1}) + (fibonacci {n - 2})}))

(define (fibonacci2 n)
  (cond [{n < 2} n]
        [else {(fibonacci {n - 1}) + (fibonacci {n - 2})}]))

(define (test times fn)
  (define fn-to-test (lambda () (repeat times fn)))
  (define total-time (time fn-to-test))
  {total-time / times})

(define (main)
  (define times 50)
  (define n 20)
  (define fn (lambda () (fibonacci2 n)))
  (define time-per-fn (test times fn))
  (printfln "time per function: #{time-per-fn}s"))

(main)