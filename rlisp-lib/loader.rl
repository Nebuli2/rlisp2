(define RLISP_HOME (env-var "RLISP_HOME"))

; Imports the file prefixed with RLISP_HOME
(define-macro-rule (import-lib name)
    (import (string-concat RLISP_HOME "/" name)))

(define-macro-rule (load)
    (begin
        (import-lib "stdio.rl")
        (import-lib "stdlib.rl")
        (import-lib "repl.rl")
        (import-lib "error.rl")
        (import-lib "map.rl")
        (import-lib "array.rl")
        (import-lib "macros.rl")))
    
(load)

; Define interactive entry point
(define interactive-start start-repl)