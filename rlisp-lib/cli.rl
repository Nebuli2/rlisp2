(define (handle-args types fn)
  (define argv (tail (args)))
  (define parsed-args (map parse argv))
  (define parsed-types (map type-of parsed-args))
  (if {parsed-types = types}
    (apply fn parsed-args)
    (begin
      (define error-text (format "signature mismatch: expected #{types}, found #{parsed-types}"))
      (define error (make-error 0 error-text nil))
      (err error))))