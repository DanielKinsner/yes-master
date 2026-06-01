#!/bin/sh
set -eu

cd "$PROJECT_DIR/rust"

export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

LIB_NAME="libyes_master_iphone_native_bridge.a"
PROFILE="debug"
CARGO_PROFILE_FLAG=""
if [ "${CONFIGURATION:-Debug}" = "Release" ]; then
  PROFILE="release"
  CARGO_PROFILE_FLAG="--release"
fi

OUT_DIR="$PROJECT_DIR/rust/build/${PLATFORM_NAME:-unknown}/${CONFIGURATION:-Debug}"
mkdir -p "$OUT_DIR"

build_target() {
  target="$1"
  cargo build --target "$target" $CARGO_PROFILE_FLAG
}

copy_target_lib() {
  target="$1"
  cp "target/$target/$PROFILE/$LIB_NAME" "$OUT_DIR/$LIB_NAME"
}

case "${PLATFORM_NAME:-}" in
  iphoneos)
    build_target aarch64-apple-ios
    copy_target_lib aarch64-apple-ios
    ;;
  iphonesimulator)
    libs=""
    case " ${ARCHS:-} " in
      *" arm64 "*)
        build_target aarch64-apple-ios-sim
        libs="$libs target/aarch64-apple-ios-sim/$PROFILE/$LIB_NAME"
        ;;
    esac
    case " ${ARCHS:-} " in
      *" x86_64 "*)
        build_target x86_64-apple-ios
        libs="$libs target/x86_64-apple-ios/$PROFILE/$LIB_NAME"
        ;;
    esac
    if [ -z "$libs" ]; then
      build_target aarch64-apple-ios-sim
      libs="target/aarch64-apple-ios-sim/$PROFILE/$LIB_NAME"
    fi
    # shellcheck disable=SC2086
    lipo -create $libs -output "$OUT_DIR/$LIB_NAME"
    ;;
  *)
    echo "Unsupported Apple platform for Rust bridge: ${PLATFORM_NAME:-unset}" >&2
    exit 1
    ;;
esac

echo "Built Rust bridge: $OUT_DIR/$LIB_NAME"
