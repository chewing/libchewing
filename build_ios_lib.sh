cmake -B build-ios/iphone \
    -DCMAKE_TOOLCHAIN_FILE=~/GitHub/ios-cmake/ios.toolchain.cmake \
    -DPLATFORM=OS64 \
    -DDEPLOYMENT_TARGET=16.0 \
    -DCMAKE_BUILD_TYPE=Release \
    -DRust_CARGO_TARGET=aarch64-apple-ios \
    -DBUILD_TESTING=off
# Build the project for iPhone


cmake --build build-ios/iphone


cmake -B build-ios/simulator \
    -DCMAKE_TOOLCHAIN_FILE=~/GitHub/ios-cmake/ios.toolchain.cmake \
    -DPLATFORM=SIMULATORARM64 \
    -DDEPLOYMENT_TARGET=16.0 \
    -DCMAKE_BUILD_TYPE=Release \
    -DRust_CARGO_TARGET=aarch64-apple-ios-sim \
    -DBUILD_TESTING=off
# Build the project for simulator

cmake --build build-ios/simulator

cmake \
    -B build-ios/simulator-x86 \
    -DCMAKE_TOOLCHAIN_FILE=~/GitHub/ios-cmake/ios.toolchain.cmake \
    -DPLATFORM=SIMULATOR64 \
    -DDEPLOYMENT_TARGET=16.0 \
    -DCMAKE_BUILD_TYPE=Release \
    -DRust_CARGO_TARGET=x86_64-apple-ios \
    -DBUILD_TESTING=OFF

cmake --build build-ios/simulator-x86

lipo -create \
  build-ios/simulator/libchewing.dylib \
  build-ios/simulator-x86/libchewing.dylib \
  -output build-ios/simulator-fat/libchewing.dylib

xcodebuild -create-xcframework \
  -library build-ios/iphone/libchewing.dylib -headers capi/include \
  -library build-ios/simulator-fat/libchewing.dylib -headers capi/include \
  -output libchewing.xcframework