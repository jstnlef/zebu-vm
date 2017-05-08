def fib(n):
    if n <= 1:
        return n
    return fib(n - 2) + fib(n - 1)


def measure(N):
    from time import time
    t0 = time()
    fib(N)
    t1 = time()
    return t0, t1


def rpy_entry(N):
    t0, t1 = measure(N)
    # from rpython.rlib import rfloat
    # print rfloat.double_to_string(t1 - t0, 'e', %(fprec)d, rfloat.DTSF_ADD_DOT_0)
    return t1 - t0

if __name__ == '__main__':
    import sys
    t0, t1 = measure(int(sys.argv[1]))
    print '%.15f' % (t1 - t0)


def target(*args):
    from rpython.rlib.entrypoint import export_symbol
    export_symbol(rpy_entry)
    return rpy_entry, [int]