#!/usr/bin/env bash
# Build from pinned source; never use an encoder from PATH as the product binary.
set -euo pipefail
export PATH="/usr/bin:$PATH"
target=${1:?Usage: build-audio-encoders.sh TARGET OUTPUT_DIRECTORY}
out=${2:?output directory required}
mkdir -p "$out"
out=$(cd "$out" && pwd)
if [ -f "$out/manifest.json" ]; then echo "Refusing to rebuild a sealed package: $out" >&2; exit 2; fi
case "$target" in
  x86_64-pc-windows-msvc) export PATH="/ucrt64/bin:/usr/bin:$PATH"; host=x86_64-w64-mingw32; arch=x86_64; os=mingw32; suffix=.exe ;;
  aarch64-apple-darwin) host=; arch=arm64; os=darwin; suffix= ;;
  x86_64-apple-darwin) host=; arch=x86_64; os=darwin; suffix= ;;
  *) echo "Unsupported encoder target: $target" >&2; exit 2 ;;
esac
for tool in make cmp curl tar "$([ "$os" = darwin ] && echo clang || echo gcc)" pkg-config; do
  command -v "$tool" >/dev/null || { echo "Missing encoder build tool: $tool" >&2; exit 2; }
done
work=$(mktemp -d /tmp/yes-master-encoder.XXXXXXXX)
trap 'rm -rf -- "$work"' EXIT
mkdir -p "$out/sources" "$out/licenses" "$out/build"
cp "$0" "$out/build/build-audio-encoders.sh"
{ "$([ "$os" = darwin ] && echo clang || echo gcc)" --version; make --version; pkg-config --version; } >"$out/build/toolchain.txt"
fetch() {
  local name=$1 url=$2 hash=$3
  if [ ! -f "$out/sources/$name.tar.xz" ]; then
    curl --fail --location --proto '=https' --tlsv1.2 "$url" -o "$out/sources/$name.tar.xz"
  fi
  if command -v sha256sum >/dev/null; then
    echo "$hash  $out/sources/$name.tar.xz" | sha256sum -c -
  else
    echo "$hash  $out/sources/$name.tar.xz" | shasum -a 256 -c -
  fi
  tar -xf "$out/sources/$name.tar.xz" -C "$work"
}
fetch ffmpeg-8.0.3 https://ffmpeg.org/releases/ffmpeg-8.0.3.tar.xz 6136812ea6d4e68bdba27e33c2a94382711cdf4f8602ffef056ff792bd6f9818
fetch libogg-1.3.5 https://downloads.xiph.org/releases/ogg/libogg-1.3.5.tar.xz c4d91be36fc8e54deae7575241e03f4211eb102afb3fc0775fbbc1b740016705
fetch libvorbis-1.3.7 https://downloads.xiph.org/releases/vorbis/libvorbis-1.3.7.tar.xz b33cc4934322bcbf6efcbacf49e3ca01aadbea4114ec9589d1b1e9d20f72954b
prefix="$work/prefix"
export PKG_CONFIG_PATH="$prefix/lib/pkgconfig"
export PKG_CONFIG_LIBDIR="$prefix/lib/pkgconfig"
configure_host=()
ff_target=()
if [ "$os" = darwin ]; then
  export CC=clang CFLAGS="-O2 -arch $arch -mmacosx-version-min=11.0" LDFLAGS="-arch $arch -mmacosx-version-min=11.0"
  ff_target=(--cc=clang "--extra-cflags=$CFLAGS" "--extra-ldflags=$LDFLAGS")
else
  export CC=gcc CFLAGS=-O2 LDFLAGS=-static
  configure_host=("--host=$host")
  ff_target=(--cc=gcc --extra-ldflags=-static)
fi
jobs=${ENCODER_BUILD_JOBS:-4}
for lib in libogg-1.3.5 libvorbis-1.3.7; do
  (
    cd "$work/$lib"
    ./configure --prefix="$prefix" --disable-shared --enable-static "${configure_host[@]}" >"$out/build/$lib-configure.log" 2>&1
    make -j"$jobs" >"$out/build/$lib-make.log" 2>&1
    make install >"$out/build/$lib-install.log" 2>&1
    cp COPYING "$out/licenses/$lib-COPYING"
  )
done
cd "$work/ffmpeg-8.0.3"
args=(--disable-everything --disable-autodetect --disable-network --disable-doc
  --disable-debug --disable-x86asm --disable-shared --enable-static
  --disable-ffplay --disable-ffprobe --enable-ffmpeg --enable-small
  --disable-avdevice --disable-swscale --enable-swresample --enable-avfilter
  --enable-libvorbis --pkg-config-flags=--static
  --enable-protocol=file,pipe --enable-demuxer=wav,flac,aac,mov,ogg,aiff
  --enable-muxer=flac,ipod,adts,ogg,aiff,wav
  --enable-decoder=pcm_f32le,pcm_s16le,pcm_s24le,pcm_s32le,pcm_s16be,pcm_s24be,flac,aac,vorbis
  --enable-encoder=flac,aac,libvorbis,pcm_s16be,pcm_s24be,pcm_s16le,pcm_s24le,pcm_f32le
  --enable-parser=aac,flac,vorbis --enable-filter=aresample,aformat,anull
  "--arch=$arch" "--target-os=$os" "${ff_target[@]}")
printf '%s\n' "${args[@]}" >"$out/build/configure-arguments.txt"
./configure "${args[@]}" >"$out/build/ffmpeg-configure.log" 2>&1
cp ffbuild/config.log "$out/build/ffmpeg-config.log"
make -j"$jobs" >"$out/build/ffmpeg-make.log" 2>&1
cp "ffmpeg$suffix" "$out/yes-master-encoder-$target$suffix"
cp COPYING.LGPLv2.1 LICENSE.md "$out/licenses/"
if [ "$os" = darwin ]; then
  otool -L "$out/yes-master-encoder-$target" >"$out/build/runtime-dependencies.txt"
  codesign --force --sign - "$out/yes-master-encoder-$target"
else
  objdump -p "$out/yes-master-encoder-$target.exe" >"$out/build/pe-headers.txt"
  sed -n 's/.*DLL Name: //p' "$out/build/pe-headers.txt" >"$out/build/runtime-dependencies.txt"
fi
"$out/yes-master-encoder-$target$suffix" -version >"$out/build/version.txt"
printf 'Built %s\n' "$out/yes-master-encoder-$target$suffix"
