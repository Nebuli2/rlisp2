(define (println x)
    (display x)
    (newline))

(define (printf s)
    (print (format s)))

(define (printfln s)
    (printf s)
    (newline))

(define print display)