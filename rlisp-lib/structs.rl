(define-struct point2d [x y])

(define (point-dist p1 p2)
    (let ([x1 (point2d-x p1)]
          [y1 (point2d-y p1)]
          [x2 (point2d-x p2)]
          [y2 (point2d-y p2)]
          [dx {x2 - x1}]
          [dy {y2 - y1}]
          [dist2 {{dx * dx} + {dy * dy}}])
        (sqrt dist2)))
