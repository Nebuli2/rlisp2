(define RLISP_HOME (env-var "RLISP_HOME"))

(import (format "#{RLISP_HOME}/stdio.rl"))
(import (format "#{RLISP_HOME}/stdlib.rl"))
(import (format "#{RLISP_HOME}/repl.rl"))
(import (format "#{RLISP_HOME}/error.rl"))
(import (format "#{RLISP_HOME}/map.rl"))

; Define interactive entry point
(define interactive-start start-repl)