#!/bin/bash

# 1) Create a separate build directory (optional, but recommended):
mkdir -p cmake-build && cd cmake-build

# 2) Configure with CMake.
#    You can override TOOLCHAIN_FILE or other variables here. For example:
cmake .. \
  -DTOOLCHAIN_FILE=~/GitHub/ios-cmake/ios.toolchain.cmake

# 3) Build everything (device slice, simulator slices, lipo, xcframework, copy resources):
cmake --build .
