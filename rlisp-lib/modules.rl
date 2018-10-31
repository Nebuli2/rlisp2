(module point
    (define-struct point [x y])
    (define (dist from to)
        (let ([x1 (point-x from)]
              [y1 (point-y from)]
              [x2 (point-x to)]
              [y2 (point-y to)]
              [dx {x2 - x1}]
              [dy {y2 - y1}])
            {{dx * dx} + {dy * dy}}))
    
    )

(import point)

(define x (point.make-point 10 10))
(define zero (point.dist x x))
