# libchewing Sample Applications

This directory contains two sample applications demonstrating how to use the **simplified Interface** for libchewing:

- **C++** sample (in `cpp/`)
- **Swift** sample (in `swift/`)

Both samples show how to:

1. Initialize a `cs_context_t`.
2. Configure data paths, page size, and symbol lengths.
3. Register callbacks for logging, preedit, buffer, candidate list, and commit events.
4. Process input keys and select candidates.
5. Cleanly terminate the context.


## Prerequisites

- **CMake** ≥ 3.24.0
- **C++ compiler** with C++11 support (e.g. `gcc` or `clang`)
- **Swift** compiler (on macOS or Linux with Swift toolchain)
- Unix-like shell (`bash`)


## Directory Layout

```
samples/
├── cpp/
│   ├── compile.sh
│   ├── main.cpp
└── swift/
│   ├── compile.sh
│   ├── BridgingHeader.h
│   └── main.swift
├── .gitignore
└── README.md
```

## Building

### C++ Sample

```bash
cd samples/cpp
./compile.sh
```

### Swift Sample

```bash
cd samples/swift
./compile.sh
```

## Running

### C++ Sample

```bash
./chewing-sample-cpp
```

### Swift Sample

```bash
./chewing-sample-swift
```