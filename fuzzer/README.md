# Fuzzing libchewing

Fuzzing is a technique to find edge cases that could crash or hang the library.
We use the AFL++ fuzzing framework. See https://aflplus.plus/ for information.

## Dependencies

We need AFL++ itself and cargo-afl to compile instrumented binaries.

**Install AFL++**

```sh
sudo apt install afl++
```

or

```sh
sudo dnf install american-fuzzy-lop
```

**Install cargo-afl**

```sh
cargo install cargo-afl
```

## Build the Fuzzers

It's recommended to build the fuzzers with instrumentation using cargo-afl.
AFL++ can use the instrumentation to find interesting inputs much faster.

```sh
cargo afl build --release
```

## Fuzzing the Fuzzers

First prepare the required input/output directory, populate the input with
initial seeds.

```sh
mkdir in out
dd if=/dev/urandom of=in/seed.bin bs=256 count=1
```

Then pick the fuzzer you want to run, invoke AFL.
Each fuzzer might require different input. Check `--help`.

```sh
afl-fuzz -i in -o out -- ../target/release/fuzzer ../out/build/rust/data/
```

## Fuzzers

### fuzzer

This fuzzer is similar to the `testchewing` command. It interprets binary input
as chewing commands then call corresponding methods to simulate user inputs.

```
ARGS:
    <syspath>
      system dictionary path

OPTIONS:
    -h, --help
      Prints help information.
```

### trieloader

This fuzzer tries to load input as a trie dictionary, then query metadata
and look up phrases.

```
ARGS:
    <dict_path>
      Trie dictionary path

OPTIONS:
    -h, --help
      Prints help information.
```

### cdbloader

This fuzzer tries to load input as a CDB dictionary, then query metadata
and look up phrases.

```
ARGS:
    <dict_path>
      Trie dictionary path

OPTIONS:
    -h, --help
      Prints help information.
```