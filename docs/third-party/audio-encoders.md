# Desktop audio encoder package

The new-format backend builds FFmpeg 8.0.3, libogg 1.3.5 and libvorbis 1.3.7
from exact SHA-256-pinned upstream release archives. WAV and LAME MP3 retain
their existing implementations. This is implementation preparation, not a
declaration that a release installer has passed its gates.

`scripts/build-audio-encoders.sh` is the common source build for Windows x64,
Mac ARM64 and Mac x64. Windows runs it through `build-audio-encoders.ps1` and
MSYS2 UCRT64. Its prerequisite packages are `make`, `diffutils`,
`mingw-w64-ucrt-x86_64-gcc` and `mingw-w64-ucrt-x86_64-pkgconf`.
Mac needs Xcode command-line tools, make and pkg-config. Cross-architecture
execution on Apple Silicon requires Rosetta; record it as emulated evidence.
The minimum Mac OS is 11.0. Universal assembly and installed checks remain
separate from a thin-architecture build.

The source build disables component autodetection, networking, GPL/nonfree
components, video/device libraries and assembly requiring a separate assembler.
Only local file/pipe protocols and the audio demuxers, encoders and filters in
the recorded configure arguments are enabled. libogg/libvorbis are statically
linked into the separate executable. The generated runtime inventory must
contain only OS-provided libraries; a dependency on an MSYS2, Homebrew or
other development DLL/dylib fails the package check.

Each output directory contains exact source archives, their license texts,
configure arguments/logs, compiler information and the build script. Preserve
these with the distributed binary. `verify-audio-encoders.mjs --seal` creates
a write-once hash manifest; later verification compares bytes to that manifest.
Rebuilding a sealed package is refused. Use a fresh output directory.

Example Windows qualification (the independent FFmpeg/ffprobe are test tools,
never copied into the product):

```powershell
./scripts/build-audio-encoders.ps1 -OutputDirectory test-output/encoder-package
node scripts/verify-audio-encoders.mjs test-output/encoder-package x86_64-pc-windows-msvc (Get-Command ffmpeg).Source (Get-Command ffprobe).Source --seal
$env:YES_MASTER_ENCODER_QUALIFICATION = (Resolve-Path test-output/encoder-package/qualification).Path
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-rc --test encoder_package -- --ignored
```

## Programme and padding contract

Qualification compares independently decoded PCM to the defined input, including
beginning and tail correlation. Lossless FLAC/AIFF must equal quantized PCM
exactly. Lossy receipts do not claim a PCM bit depth.

- M4A AAC-LC: gapless decoder removes 1,024 priming frames; final block padding
  is at most 1,023 frames. The container records the intended programme length.
- ADTS AAC-LC: retains 1,024 priming frames and up to 1,023 final block frames.
  Inputs shorter than 2,048 frames receive trailing zeros to that minimum so
  automatic probing can recognize at least three encoded packets. Total excess
  is bounded by 3,071 frames. No source frame is removed. ADTS cannot express
  sample-accurate gapless length; M4A is preferable when that matters.
- Ogg Vorbis: use `-page_duration 1` to flush each packet onto its own page.
  Without it, a 257-frame file placed on a single first-and-last audio page
  decoded to 129 frames in independent FFmpeg. Per-packet pages retained all
  257 frames without adding/removing samples. The tradeoff is container overhead.

These rules must be carried into the actual delivery adapter and tested for
Track and Album; passing a command-line codec test alone does not prove export.

## Redistribution boundaries

FFmpeg's configured license is LGPL 2.1-or-later; libogg and libvorbis retain
their upstream BSD notices. Upstream [FFmpeg distribution guidance](https://ffmpeg.org/legal.html)
and the actual retained license texts govern distribution. FFmpeg source
archive authenticity was checked against upstream's published release key
`FCF986EA15E6E293A5644F10B4322F04D67658D8`; hashes are pinned in the build and
verification scripts. No source patches are currently applied.

The existing in-process LAME Rust wrappers have a separate LGPL v3 source/relink
obligation. Shipping this sidecar does not close it. The repository's current
source-available license restricts modification/distribution; supplying an
archive alone is not evidence that the required recipient relinking permissions
have been granted. Prepare exact source/rebuild materials and an owner-reviewed
license exception before public distribution. Do not silently change LICENSE.

## Evidence status

See [the live implementation ledger](../plans/2026-09-09-export-formats-evidence.md).
The owner confirmed that Mac checks must wait for a later Mac session. No Mac
build, emulated execution, universal assembly or installed result is claimed here.
