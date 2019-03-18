import time

def time_fn(fn):
    start = time.time()
    fn()
    stop = time.time()
    return stop - start

def test_body(times, fn):
    for _ in range(0, times):
        fn()

def dec1(n):
    return n - 1

def dec2(n):
    return dec1(dec1(n))

def fib(n):
    if n < 2:
        return n
    else:
        return fib(n - 1) + fib(n - 2)

def test(times, fn):
    return time_fn(lambda: test_body(times, fn)) / times

body = lambda: fib(30)
avg = test(100, body)
print 'Time per fn: {}s'.format(avg)
    