#!/usr/bin/env bash
set -euo pipefail

#  Recompile the Rust/C API (chewing_capi) for two ABIs via `cargo ndk`.
#  Link chewingJNI.cpp + chewing-simplified.cpp + libchewing_capi.a → libchewing.so
#    (for each ABI: arm64-v8a & armeabi-v7a).
#  Build the host chewing-cli tool, then run `init-database` on *.src → *.dat.
#  Copy the resulting `libchewing.so` into a user‐supplied jniLibs directory,
#  and copy all generated `.dat`/`.cin` files into a user‐supplied assets directory.



#  Parse command-line arguments (require all three)
NDK_ROOT=""
OUT_JNI=""
OUT_ASSETS=""

while [ $# -gt 0 ]; do
  case "$1" in
    --ndk)
      shift
      NDK_ROOT="$1"
      ;;
    --out-jni)
      shift
      OUT_JNI="$1"
      ;;
    --out-assets)
      shift
      OUT_ASSETS="$1"
      ;;
    *)
      echo "ERROR: Unexpected argument: $1"
      usage
      exit 1
      ;;
  esac
  shift
done

if [ -z "$NDK_ROOT" ] || [ -z "$OUT_JNI" ] || [ -z "$OUT_ASSETS" ]; then
  echo "ERROR: You must supply --ndk, --out-jni, and --out-assets."
  usage
  exit 1
fi

if [ ! -d "$NDK_ROOT" ]; then
  echo "ERROR: NDK_ROOT='$NDK_ROOT' does not exist or is not a directory."
  exit 1
fi

TOOLCHAIN="$NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64"
CLANGPP="$TOOLCHAIN/bin/clang++"
SYSROOT="$TOOLCHAIN/sysroot"

for p in "$TOOLCHAIN" "$CLANGPP" "$SYSROOT"; do
  if [ ! -e "$p" ]; then
    echo "ERROR: Cannot find required NDK path: $p"
    exit 1
  fi
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIBCHEWING_ROOT="$SCRIPT_DIR"
CAPI_DIR="$LIBCHEWING_ROOT/capi"
DATA_DIR="$LIBCHEWING_ROOT/data"
TARGET_ANDROID="$LIBCHEWING_ROOT/target/android"

echo ""
echo "-------------------------------------------------------------------"
echo "  libchewing/build-android.sh starting"
echo "-------------------------------------------------------------------"
echo "  NDK_ROOT:        $NDK_ROOT"
echo "  clang++ path:    $CLANGPP"
echo "  Output (JNI):    $OUT_JNI"
echo "  Output (Assets): $OUT_ASSETS"
echo "  libchewing root: $LIBCHEWING_ROOT"
echo "  C API folder:    $CAPI_DIR"
echo "  Data folder:     $DATA_DIR"
echo "  Rust Android target dir: $TARGET_ANDROID"
echo "-------------------------------------------------------------------"
echo

echo "Building static libchewing_capi.a via cargo-ndk"
pushd "$CAPI_DIR" > /dev/null

cargo ndk \
  -t aarch64-linux-android \
  -t armv7-linux-androideabi \
  -- build --release --target-dir ../target/android

popd > /dev/null

A64_A="$TARGET_ANDROID/aarch64-linux-android/release/libchewing_capi.a"
A77_A="$TARGET_ANDROID/armv7-linux-androideabi/release/libchewing_capi.a"

mkdir -p "$OUT_JNI/arm64-v8a"
mkdir -p "$OUT_JNI/armeabi-v7a"

JNI_BRIDGE_CPP="$CAPI_DIR/src/chewingJNI.cpp"
SIMPLIFIED_CPP="$CAPI_DIR/src/chewing-simplified.cpp"

for f in "$JNI_BRIDGE_CPP" "$SIMPLIFIED_CPP" "$A64_A" "$A77_A"; do
  if [ ! -e "$f" ]; then
    echo "ERROR: Required file does not exist: $f"
    exit 1
  fi
done

echo "Linking libchewing.so for arm64-v8a"
CMD_A64=(
  "$CLANGPP"
    --target=aarch64-none-linux-android30
    --sysroot="$SYSROOT"
    -fPIC -shared
    -std=c++14 -stdlib=libc++
    -I"$LIBCHEWING_ROOT"
    -I"$CAPI_DIR/include"
    "$JNI_BRIDGE_CPP"
    "$SIMPLIFIED_CPP"
    "$A64_A"
    -llog
    -o "$OUT_JNI/arm64-v8a/libchewing.so"
)
echo "  > ${CMD_A64[*]}"
"${CMD_A64[@]}"
echo "→ arm64-v8a .so → $OUT_JNI/arm64-v8a/libchewing.so"
echo

echo "Linking libchewing.so for armeabi-v7a"
CMD_A77=(
  "$CLANGPP"
    --target=armv7a-linux-androideabi30
    --sysroot="$SYSROOT"
    -march=armv7-a -mfloat-abi=softfp -mfpu=vfpv3-d16
    -fPIC -shared
    -std=c++14 -stdlib=libc++
    -I"$LIBCHEWING_ROOT"
    -I"$CAPI_DIR/include"
    "$JNI_BRIDGE_CPP"
    "$SIMPLIFIED_CPP"
    "$A77_A"
    -llog
    -o "$OUT_JNI/armeabi-v7a/libchewing.so"
)
echo "  > ${CMD_A77[*]}"
"${CMD_A77[@]}"
echo "→ armeabi-v7a .so → $OUT_JNI/armeabi-v7a/libchewing.so"
echo

echo "Building host‐side chewing-cli (macOS)"
pushd "$LIBCHEWING_ROOT" > /dev/null

cargo build --release -p chewing-cli
HOST_CLI="$LIBCHEWING_ROOT/target/release/chewing-cli"

if [ ! -x "$HOST_CLI" ]; then
  echo "ERROR: Could not build host chewing-cli at: $HOST_CLI"
  exit 1
fi

popd > /dev/null
echo "→ Host chewing-cli is at: $HOST_CLI"
echo

echo "Generating .dat from .src via chewing-cli"
DATA_SRC="$DATA_DIR"
DATA_BIN="$DATA_DIR"
mkdir -p "$DATA_BIN"

DATA_COPYRIGHT="Copyright (c) 2022 libchewing Core Team"
DATA_LICENSE="LGPL-2.1-or-later"
DATA_VERSION="0.9.1"

echo "  • Generating tsi.dat..."
"$HOST_CLI" init-database \
  -c "$DATA_COPYRIGHT" \
  -l "$DATA_LICENSE" \
  -r "$DATA_VERSION" \
  -t trie \
  -n "內建詞庫" \
  "$DATA_SRC/tsi.src" \
  "$DATA_BIN/tsi.dat"

echo "  • Generating word.dat..."
"$HOST_CLI" init-database \
  -c "$DATA_COPYRIGHT" \
  -l "$DATA_LICENSE" \
  -r "$DATA_VERSION" \
  -t trie \
  -n "內建字庫" \
  "$DATA_SRC/word.src" \
  "$DATA_BIN/word.dat"

echo "  • Generating mini.dat..."
"$HOST_CLI" init-database \
  -c "$DATA_COPYRIGHT" \
  -l "$DATA_LICENSE" \
  -r "$DATA_VERSION" \
  -t trie \
  -n "內嵌字庫" \
  "$DATA_SRC/mini.src" \
  "$DATA_BIN/mini.dat"

echo "→ Generated .dat files in: $DATA_BIN"
ls -1 "$DATA_BIN"/*.dat | sed 's/^/    /'
echo

echo "Copying dictionary files into Android assets"
mkdir -p "$OUT_ASSETS"

cp -pv "$DATA_BIN"/*.dat    "$OUT_ASSETS/"
cp -pv "$DATA_SRC"/*.cin    "$OUT_ASSETS/"

echo "→ Final contents of $OUT_ASSETS:"
ls -1 "$OUT_ASSETS" | sed 's/^/    /'
echo

echo "-------------------------------------------------------------------"
echo "  build-android.sh finished successfully."
echo "    • libchewing.so → $OUT_JNI/[arm64-v8a, armeabi-v7a]/libchewing.so"
echo "    • chewing_data/*.dat, *.cin → $OUT_ASSETS"
echo
