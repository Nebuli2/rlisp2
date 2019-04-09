(define print display)

(define (println x)
    (display x)
    (newline))

(define printf (compose print format))

(define (printfn s)
    (printf s)
    (newline))

(define printfln printfn)

(define print display)

(define-macro-rule (debug msg)
  (print "[" __FILE__ "] " msg "\n"))