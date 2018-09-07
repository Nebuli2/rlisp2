(define-struct option [present? value])

(define (some x)
    (make-option true x))

(define none
    (make-option false empty))

(define-struct result [ok? value])

(define (ok x)
    (make-result true x))

(define (err x)
    (make-result false x))

(define (monad-success? obj)
    (cond [(is-result? obj) (result-ok? obj))]
          [(is-option? obj) (option-present? obj)]
          [else false]))

(define (monad-value obj)
    (cond [(is-result? obj) (result-value obj)]
          [(is-option? obj) (option-value obj)]
          [else empty]))

(define (monad-flatmap fn obj)
    (cond [(monad-success? obj) (fn (monad-value obj))]
          [else obj]))