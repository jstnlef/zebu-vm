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
import os
import py

from rpython.translator.platform import platform
from rpython.translator.platform import log as log_platform
from util import bin_dir, may_spawn_proc

@may_spawn_proc
def test_PyPy():
    pypy_dir = py.path.local(os.environ.get(
        'PYPY_MU', str(py.path.local(__file__).join('..', 'mu-client-pypy'))))

    python = os.environ.get('PYTHON', 'pypy')   # by default use pypy
    target = bin_dir.join('pypy-zebu')

    cmd = [pypy_dir.join('rpython/bin/rpython')]
    flags = ['-O3', '--no-shared', '--backend=mu', '--mu-impl=zebu',
             '--mu-vmargs', '--gc-immixspace-size=10737418240', '--mu-suplibdir=%(bin_dir)s' % globals()]
    # flags = ['-O3', '--no-shared', '--backend=c', '--no-profopt']
    args = ['--no-allworkingmodules']
    cmd.extend(flags)

    cmd.extend(['--output=%s' % target])

    cmd.append(pypy_dir.join('pypy', 'goal', 'targetpypystandalone.py'))
    cmd.extend(args)

    cmd = map(str, cmd)

    log_platform.execute(' '.join([python] + cmd))
    res = platform.execute(python, map(str, cmd))
    assert res.returncode == 0, res.err

    fib_py = bin_dir.join('fib.py')
    with fib_py.open('w') as fp:
        fp.write("""
def fib(n):
    if n in (0, 1):
        return n
    return fib(n - 1) + fib(n - 2)

def main(argv):
    print fib(int(argv[1]))

if __name__ == "__main__":
    import sys
    main(sys.argv)
""")

    log_platform.execute(' '.join([str(target), str(fib_py), '10']))
    res = platform.execute(str(target), [str(fib_py), '10'])
    assert res.returncode == 0, res.err
    assert res.out == "55\n"
