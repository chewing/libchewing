
#!/bin/bash

mkdir -p ../build/swift
pushd ../build/swift

cmake ../../../

make

swiftc \
   -Xcc -I../../../capi/include \
   -import-objc-header ../../swift/BridgingHeader.h \
   ../../swift/main.swift \
   -Xlinker -L. \
   -Xlinker -rpath \
   -Xlinker @executable_path/../build/swift \
   -lchewing \
   -o chewing-sample-swift

cp ./chewing-sample-swift ../../swift/chewing-sample-swift
popd