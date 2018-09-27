; PROMPT :: string
(define PROMPT "rlisp> ")

; _ :: any
; Represents the last expression evaluated by the REPL.
(define _ nil)

; warn :: string -> nil
; Prints a warning containing the specified message in the following format:
; `warn: msg`
(define (warn msg)
    (display-pretty 'yellow 'bold "warning")
    (display-pretty 'none 'bold #": #{msg}")
    (newline))

; err :: error -> nil
; Prints the specified error in the following format:
; `error(error-code): error-description`
(define (err error)
    (let ([code (error-code error)]
          [description (error-description error)])
        (display-pretty 'red 'bold #"error(#{code})")
        (display-pretty 'none 'bold #": #{description}")
        (newline)))

; flush-stdout :: -> nil
; Flushes the standard output buffer.
(define (flush-stdout)
    (display))

; prompt :: string -> nil
; Prints the specified text in bold, green text. The text is not followed by a
; newline.
(define (prompt text)
    (display-pretty 'green 'bold text)
    (flush-stdout))

; repl :: -> nil
(define (repl)
    (prompt PROMPT)
    (try
        (begin
            (define value (eval (parse (readline))))
            (if {value /= nil}
                (println value)
                nil)
            (set! '_ value))
        err)
    (repl))

; version :: -> string
(define (version)
    "0.1.1")

; help :: -> nil
(define (help)
    (println #"Welcome to Rlisp #{(version)}!")
    (println "To interact with the REPL, simply enter an expression after the prompt."))

; greet :: string -> nil
(define (greet version)
    (println #"Rlisp #{version}")
    (println "Type `(help)` for more information."))

(define (start-repl)
    (greet (version))
    (repl))
