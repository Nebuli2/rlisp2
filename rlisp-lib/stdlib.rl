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
    (cond [(empty? xs) nil]
          [else (cons (f (head xs)) (map f (tail xs)))]))

; sum :: (list num) -> num
(define (sum xs)
    (apply + xs))

; product :: (list num) -> num
(define (product xs)
    (apply * xs))

; range :: num num -> (list num)
(define (range from to)
    (cond [{from = to} nil]
          [else (cons from (range {from + 1} to))]))

; to :: num num -> (list num)
(define {x to y}
    (range x {y + 1}))

; singleton :: a -> (list a)
(define (singleton x)
    (cons x nil))

; (define foldr (lambda [f] (lambda [acc] (lambda [xs]
;     (cond [(empty? xs) xs]
;           [else (foldr f (f (head xs) acc) (tail xs))])))))

; print :: a -> nil
(define print display)

; println :: a -> nil
(define (println s)
    (display s)
    (newline))

(define (string-append a b)
    (format "#{a}#{b}"))

(define ++ string-append)

(define (println-debug ex)
    (display-debug ex)
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

(define {a /= b}
    (not {a = b}))

(define-macro-rule (reload)
    (import (string-concat RLISP_HOME "/loader.rl")))

; (define-macro def
;     [(def (name args...) body...)
;      (define name (lambda [...args] ...body))]
;     [(def name body...)
;      (define name (begin ...body))])

; (define-macro map (syntax [:])
;     [(map )])

; (tree a) = (tree a) | nil
(define-struct tree [value left right])

(define (singleton-tree val)
    (make-tree val nil nil))

(define (tree-insert val tree)
    (cond [{tree = nil} (singleton-tree val)]
          [else (begin
            (define head (tree-value tree))
            (define left (tree-left tree))
            (define right (tree-right tree))
            (cond [{val < head} 
                    (make-tree head (tree-insert val left) right)]
                  [else
                    (make-tree head left (tree-insert val right))]))]))

(define (list->tree list)
    (foldr tree-insert nil list))

(define (tree->list tree)
    (cond [{tree = nil} tree]
          [else (let ([val (tree-value tree)]
                      [left (tree-left tree)]
                      [right (tree-right tree)])
                    (append (tree->list left) (singleton val) (tree->list right)))]))

(define (tree-sum tree)
    (cond [{tree = nil} 0]
          [else (begin
            (define val (tree-value tree))
            (define left (tree-left tree))
            (define right (tree-right tree))
            (+ val (tree-sum left) (tree-sum right)))]))

; sort :: (list a) -> (list a)
(define sort (compose tree->list list->tree))
        
(define tree
    (make-tree 10
        (make-tree 9
            (make-tree 3
                nil
                nil)
            nil)
        (make-tree 7
            nil
            nil)))

(define (time action)
    (define start (current-time))
    (action)
    (define stop (current-time))
    (- stop start))