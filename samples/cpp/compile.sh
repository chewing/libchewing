#!/bin/bash

mkdir -p ../build/cpp
pushd ../build/cpp
cmake ../../cpp
make
cp ./chewing-sample-cpp ../../cpp/chewing-sample-cpp
popd