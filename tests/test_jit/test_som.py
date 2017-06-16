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

from rpython.rtyper.lltypesystem import rffi, lltype
from rpython.rlib.rmu import zebu as rmu
from rpython.translator.platform import platform
from util import fncptr_from_rpy_func, fncptr_from_py_script, may_spawn_proc
import ctypes, py, stat, os
import pytest

from test_rpython import run_boot_image

@may_spawn_proc
def test_RPySOM():
    from som.vm.universe import main, Exit

    def entry_point(argv):
        try:
            main(argv)
        except Exit, e:
            return e.code
        except Exception, e:
            os.write(2, "ERROR: %s thrown during execution.\n" % e)
            return 1
        return 1

    RPYSOM = os.environ.get('RPYSOM', str(py.path.local(__file__).join('..', 'RPySOM')))

    res = run_boot_image(entry_point, '/tmp/RPySOM-no-jit-mu',
                         args=['-cp', '%(RPYSOM)s/Smalltalk' % locals(),
                               '%(RPYSOM)s/TestSuite/TestHarness.som' % locals()])
    assert res.returncode == 0, res.err
    expected_out = \
"""\
Testing...
Running test EmptyTest
Running test SystemTest
Running test ArrayTest
Running test ClassLoadingTest
Running test ClosureTest
Running test CoercionTest
Running test CompilerReturnTest
Running test DoubleTest
Running test HashTest
Running test IntegerTest
Warning: Test instance of IntegerTest failed: Identity failed. Expected: true, but Actual: false
Warning: Test instance of IntegerTest failed: Identity failed. Expected: true, but Actual: false
Running test ObjectSizeTest
Warning: Test instance of ObjectSizeTest failed: Plain object does not have size 1.
Warning: Test instance of ObjectSizeTest failed: Integer object does not have size 1.
Warning: Test instance of ObjectSizeTest failed: hello String object does not have size 1.
Warning: Test instance of ObjectSizeTest failed: Empty array object does not have size 1.
Warning: Test instance of ObjectSizeTest failed: Array object (length 4) does not have size 5.
Running test PreliminaryTest
Running test ReflectionTest
Running test SelfBlockTest
Running test SuperTest
Running test SymbolTest
Running test VectorTest
Running test BlockTest
Running test StringTest
Running test ClassStructureTest
Definition of Class changed. Testcase needs to be updated.
Running test DoesNotUnderstandTest
...done
"""
    assert res.out == expected_out