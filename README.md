# Zebu VM (a fast implementation of Mu Micro VM)

[![build status](https://gitlab.anu.edu.au/mu/mu-impl-fast/badges/master/build.svg)](https://gitlab.anu.edu.au/mu/mu-impl-fast/commits/master)

Mu Micro VM is a substrate language virtual machine. Mu executes its
language-/platform-neutural immediate representation to provide common
low-level abstractions for programming language implementation, such as
machine code generation and execution, garbage collection and concurrency.
With Mu, language developers only need to care about language-specific
optimisation and execution while leaving the low-level part to Mu Micro VM.

Zebu VM is a fast implementation of Mu, developed in Australia National
University.

## Platform support

Zebu supports the follow platforms:
* x86_64/linux (CI)
* aarch64/linux
* x86_64/macos (macOS 10.12+)

## Mu specification coverage

Zebu does not implement full Mu specification yet.
[This label](https://gitlab.anu.edu.au/mu/mu-impl-fast/issues?label_name%5B%5D=spec+coverage)
in the issue tracker keeps track of unimplemented features, or features that
are not compliant to Mu spec.

## Building

You will need:
* rust version 1.19 (0ade33941 2017-07-17)
* clang 4.0+
* cmake 3.8+ (we do not depend on cmake, but some Rust crates use it)
* internet connection (as Rust will download dependencies)

To build Zebu with release build,
```
  cd path/to/zebu
  MU_ZEBU=. CC=clang cargo build --release
```
you will get shared and static libraries for Zebu under `target/release/`
that you can link against in your language implementation.

You can also build Zebu in debug.
```
  cd path/to/zebu
  MU_ZEBU=. CC=clang cargo build
```

## Testing

Zebu repository includes two test suites:
* cargo test
* pytest

#### Running tests with cargo test

```
  cd path/to/zebu
  MU_ZEBU=. CC=clang RUST_TEST_THREADS=1 cargo test --release 2>/dev/null
```

#### Running tests with pytest

To facilitate tests, Zebu uses RPython (which targets Mu as backend) for some
of tests in this suite.

Download [PyPyMu](https://gitlab.anu.edu.au/mu/mu-client-pypy)
```
  cd path/to/zebu
  git clone https://gitlab.anu.edu.au/mu/mu-client-pypy.git tests/test_jit/mu-client-pypy
  cd tests/test_jit/mu-client-pypy
  git checkout mu-rewrite
  git apply pypy.patch
```

Download [RPySOM](https://github.com/microvm/RPySOM)
```
  cd path/to/zebu
  git clone https://gitlab.anu.edu.au/mu/x-RPySOM.git tests/test_jit/RPySOM
  cd tests/test_jit/RPySOM
  git submodule init
  git submodule update
```

Running pytest (you will need Python 2.7 with pytest module)
```
  cd path/to/zebu/tests/test_jit
  export DYLD_LIBRARY_PATH=.
  export MU_ZEBU=path/to/zebu
  export MU_LOG_LEVEL=none
  export PYTHONPATH=mu-client-pypy:RPySOM/src
  export RPYSOM=RPySOM
  export ZEBU_BUILD=release
  export CC=clang
  export SPAWN_PROC=1
  python2 -m pytest test*.py -v
```

## Using Zebu for your language implementation

Zebu provides the C binding for its API declared in
[mu-fastimpl.h](src/vm/api/mu-fastimpl.h). The APIs are defined
in [Mu specification](https://gitlab.anu.edu.au/mu/mu-spec).
The header also includes Zebu-specific APIs, such as `mu_fastimpl_new()`.

Zebu allows the user to set options when creating a new instance.
The options can be found in [vm_options.rs](src/vm/vm_options.rs).

## Bug reports

As Zebu is still in its early development, we expect bugs and
missing features. We appreciate if you can report to
[Issues](https://gitlab.anu.edu.au/mu/mu-impl-fast/issues).
Our priority for Zebu development is driven by the issue tracker along with
two client implementations that are being actively developed in ANU,
[PyPy-Mu](https://gitlab.anu.edu.au/mu/mu-client-pypy)
and [GHC-Mu](https://gitlab.anu.edu.au/mu/mu-client-ghc).

## License

Zebu uses Apache 2.0. See the [license](LICENSE).
