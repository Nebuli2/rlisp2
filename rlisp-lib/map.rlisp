; (option a) = a | nil
; (map k v) = k -> (option a)

; map-empty :: k v -> (map k v)
(define (map-empty k)
    empty)

; map-insert :: k v (map k v) -> (map k v)
(define (map-insert k v m)
    (lambda [key] (if {k = key}
        v
        (m key))))



