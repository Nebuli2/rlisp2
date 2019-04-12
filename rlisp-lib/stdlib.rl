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

; for-each :: (a -> b) (list a) -> nil
(define (for-each f xs)
    (cond [(empty? xs) nil]
          [else (begin
            (f (head xs))
            (for-each f (tail xs)))]))

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

(define (string-append a b)
    (format "#{a}#{b}"))

(define ++ string-append)

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

(define (length xs)
    (cond [(empty? xs) 0]
          [else (+ 1 (length (tail xs)))]))

(define (nth n xs)
    (cond [(empty? xs) nil]
          [(= n 0) (head xs)]
          [else (nth (- n 1) (tail xs))]))

(define (zip xs ys)
    (cond [(empty? xs) nil]
          [(empty? ys) nil]
          [else (cons `[,(head xs) ,(head ys)] (zip (tail xs) (tail ys)))]))