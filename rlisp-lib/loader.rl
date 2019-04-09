(define RLISP_HOME (env-var "RLISP_HOME"))

(import "./stdlib.rl")
(import "./stdio.rl")
(import "./repl.rl")
(import "./error.rl")
(import "./array.rl")
(import "./macros.rl")

; Define interactive entry point
(define interactive-start start-repl)