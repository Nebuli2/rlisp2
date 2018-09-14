; make-option :: bool a -> (option a)
; option-present? :: (option a) -> bool
; option-value 
;   | option-present? :: (option a) -> a
;   | else :: (option a) -> nil
(define-struct option [present? value])

; some :: a -> (option a)
(define (some x)
    (make-option true x))

; none :: (option a)
(define none
    (make-option false nil))

; make-result :: bool (a | b) -> (result a b)
; result-ok? :: (result a b) -> bool
; result-value
;   | result-ok? :: (result a b) -> a
;   | else :: (result a b) -> b
(define-struct result [ok? value])

; ok :: a -> (result a b)
(define (ok x)
    (make-result true x))

; err :: b -> (result a b)
(define (err x)
    (make-result false x))

; monad-success? :: (m a) -> bool
; where m = option | result
(define (monad-success? obj)
    (cond [(is-result? obj) (result-ok? obj))]
          [(is-option? obj) (option-present? obj)]
          [else false]))

; monad-value :: (m a) -> a
; where m = option | result
(define (monad-value obj)
    (cond [(is-result? obj) (result-value obj)]
          [(is-option? obj) (option-value obj)]
          [else nil]))

; monad-flatmap :: (a -> (m b)) (m a) -> (m b)
; where m = option | result
(define (monad-flatmap fn obj)
    (cond [(monad-success? obj) (fn (monad-value obj))]
          [else obj]))