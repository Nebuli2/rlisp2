; empty : list[a]
; (define empty '())

; compose :: (b -> c) (a -> b) -> (a -> c)
(define (compose f g)
    (lambda [x] (f (g x))))

; apply :: (a... -> b) (list a) -> b
(define (apply f xs)
    (eval (cons 'f xs)))

; foldl :: (a b -> b) b (list a) -> b
(define (foldl f acc xs)
    (cond [(empty? xs) acc]
          [else (foldl f (f (head xs) acc) (tail xs))]))

; foldl :: (a b -> b) b (list a) -> b
(define (foldr f acc xs)
    (cond [(empty? xs) acc]
          [else (f (head xs) (foldr f acc (tail xs)))]))

; map :: (a -> b) (list a) -> (list b)
(define (map f xs)
    (cond [(empty? xs) empty]
          [else (cons (f (head xs)) (map f (tail xs)))]))

; sum :: (list num) -> num
(define (sum xs)
    (apply + xs))

; product :: (list num) -> num
(define (product xs)
    (apply * xs))

; range :: num num -> (list num)
(define (range from to)
    (cond [{from = to} empty]
          [else (cons from (range {from + 1} to))]))

; to :: num num -> (list num)
(define {x to y}
    (range x {y + 1}))

; singleton :: a -> (list a)
(define (singleton x)
    (cons x empty))

; append :: a (list a) -> (list a)
(define (append x xs)
    (cond [(empty? xs) (singleton x)]
          [else (cons (head xs) (append x (tail xs)))]))

; (define foldr (lambda [f] (lambda [acc] (lambda [xs]
;     (cond [(empty? xs) xs]
;           [else (foldr f (f (head xs) acc) (tail xs))])))))

; PROMPT :: string
; Defines the prompt used by the REPL. If none is defined, it defaults to "> ".
(define PROMPT "rlisp> ")

; (define-syntax reload
;     (syntax-rules []
;         (reload)
;         (import "stdlib.rlisp")))

; prompt :: string -> string
(define (prompt p)
    (display p)
    (readline))

; print :: a -> nil
(define print display)

; println :: a -> nil
(define (println s)
    (display s)
    (newline))

; factorial :: num -> num
(define (factorial n)
    (product {1 to n}))

; make-point :: num num -> point
; point-x :: point -> num
; point-y :: point -> num
(define-struct point [x y])

; printf :: string -> nil
(define printf (compose print format))

(define (is-even? n)
    {{n % 2} = 0})

(define \ lambda)
(define x (\()
    (println "Hello world")
    (println "Goodbye world")))