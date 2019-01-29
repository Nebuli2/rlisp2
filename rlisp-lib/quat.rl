(define-struct quaternion [a b c d])

(define (quat-add x y)
  (define a1 (quaternion-a x))
  (define b1 (quaternion-b x))
  (define c1 (quaternion-c x))
  (define d1 (quaternion-d x))
  (define a2 (quaternion-a y))
  (define b2 (quaternion-b y))
  (define c2 (quaternion-c y))
  (define d2 (quaternion-d y))
  (make-quaternion
    {a1 + a2}
    {b1 + b2}
    {c1 + c2}
    {d1 + d2}))