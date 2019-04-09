#!/usr/local/bin/rlisp

; fix-sign :: number -> number
; Inverts the sign of the number if it is even.
(define (fix-sign x)
  (cond [{{x % 2} = 0} (- x)]
        [else x]))

; iterate-inner :: number number -> number
; Performs `n` iterations and produces the sum of the iterations, given an
; initial sum `sum`.
(define (iterate-inner n sum)
  (cond [{n = 0} sum]
        [else (iterate-inner
          {n - 1} {{1 / (fix-sign n)} + sum})]))

; iterate :: number -> number
; Performs `n` iterations and produces the sum of the iterations.
(define (iterate iters)
  (iterate-inner iters 0))

; abs :: number -> number
; Produces the absolute value of the specified number.
(define (abs x)
  (cond [{x < 0} (- x)]
        [else x]))

; test-diff :: number -> number
; Produces the differene between `iters` iterations of the formula and the
; expected value, `ln(2)`.
(define (test-diff iters)
  (abs {(ln 2) - (iterate iters)}))

; test-n-times :: number -> nil
; Performs `test-diff` `iters` times and prints the results of each number of
; interactions.
(define (test-n-times iters)
  (define xs (range 1 {iters + 1}))
  (define (f x)
    (define diff (test-diff x))
    (printfln "#{x} => #{diff}"))
  (for-each xs xs))

(define (main)
  (define args (args))
    (cond [{(length args) = 2} 
      (begin
        (define n (parse (head (tail args))))
        (test-n-times n))]))

(main)