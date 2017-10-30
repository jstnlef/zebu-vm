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

set -e
export MU_ZEBU=$(dirname $0)
export MU_LOG_LEVEL=none
export RUST_TEST_THREADS=1
export RUST_BACKTRACE=0
export PYTHONPATH="$MU_ZEBU/tests/test_jit/mu-client-pypy/:$MU_ZEBU/tests/test_jit/RPySOM/src"
export LD_LIBRARY_PATH="$MU_ZEBU/target/$ZEBU_BUILD:$MU_ZEBU/tests/test_jit/:$MU_ZEBU/tests/test_jit/emit/:$LD_LIBRARY_PATH"
export ZEBU_BUILD=release

rm -rf $MU_ZEBU/emit
rm -rf $MU_ZEBU/tests/test_jit/emit

cargo update
#cargo clean
cargo-fmt -- --write-mode=diff --verbose -- src/ast/src/lib.rs src/gc/src/lib.rs src/utils/src/lib.rs | tee cargo_fmt_out.txt
cargo test --release --no-run --color=always 2>&1 | tee build_out.txt
$(exit ${PIPESTATUS[0]}) # this command will exit the shell but only if the above cargo test failed

/usr/bin/time -f "finished in %e secs" -a -o cargo_test_out.txt ./test-release --color=always 2>/dev/null | tee cargo_test_out.txt

cd $MU_ZEBU/tests/test_jit/

if [ -d "./mu-client-pypy" ]
then
        git -C ./mu-client-pypy pull
else
        git clone https://gitlab.anu.edu.au/mu/mu-client-pypy.git
        git -C ./mu-client-pypy checkout mu-rewrite
        git -C ./mu-client-pypy apply pypy.patch
fi


if [ -d "./RPySOM" ]
then
        git -C ./RPySOM pull
else
        git clone https://github.com/microvm/RPySOM.git
        git -C ./RPySOM submodule init
        git -C ./RPySOM submodule update
fi
shopt -s extglob
pytest ./test_!(pypy).py -v --color=yes 2>&1 | tee $MU_ZEBU/pytest_jit_out.txt

cd $MU_ZEBU/tests/test_muc
pytest ./test_*.py -v --color=yes 2>&1 | tee $MU_ZEBU/pytest_muc_out.txt
