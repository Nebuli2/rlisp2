(define (handle-ex ex)
    (print "[error] ")
    (println ex))

(define (repl)
    (print PROMPT)
    (try (println (eval (parse (readline))))
         handle-ex)
    (repl))