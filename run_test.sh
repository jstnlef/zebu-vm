#!/bin/sh

RUST_BACKTRACE=1 RUST_TEST_THREADS=1 cargo test "$@"
