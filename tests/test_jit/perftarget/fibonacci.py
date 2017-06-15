# Copyright 2017 The Australian National University
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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