; (option a) = a | nil
; (map k v) = k -> (option a)

; map-empty :: k v -> (map k v)
(define (map-empty k)
    nil)

; map-insert :: k v (map k v) -> (map k v)
(define (map-insert k v m)
    (lambda [key] (if {k = key}
        v
        (m key))))

(define ABC 'abc)

(define (map-of pairs)
    (if (empty? pairs)
        map-empty
        (let ([pair (head pairs)]
              [key (head pair)]
              [val (head (tail pair))])
            (map-insert key val (map-of (tail pairs))))))

(define (length xs)
    (if (empty? xs)
        0
        (+ 1 (length (tail xs)))))

(define (array-of-internal xs i)
    (if (empty? xs)
        map-empty
        (map-insert i (head xs) (array-of-internal (tail xs) (+ i 1)))))

(define (array-of vals)
    (array-of-internal vals 0))

(define test-map 
    (map-of '([0 10]
              [1 20]
              [2 30])))

