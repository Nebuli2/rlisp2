#!/usr/local/bin/rlisp
#enable-preprocessor

define : fibonacci n
  cond
    {n < 2} n
    else {(fibonacci {n - 1}) + (fibonacci {n - 2})}

define : main
  define n
    parse
      head
        tail : args
  define fib-n : fibonacci n
  printfn "fibonacci(#{n}) = #{fib-n}"

main