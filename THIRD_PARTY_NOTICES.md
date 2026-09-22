# Desktop MP3 dependencies

New-format encoder preparation is documented in
[Desktop audio encoder package](docs/third-party/audio-encoders.md). It retains
exact FFmpeg/libogg/libvorbis sources and licenses beside each generated binary.
The sidecar has not yet completed the installed release gates and does not
replace the static LAME obligations below.

The desktop `app-runner` feature embeds LAME for offline MP3 export. Mobile
bridge builds do not enable this encoder. No external encoder installation or
network service is used by export.

| Locked dependency | Source | License text retained here |
| --- | --- | --- |
| `mp3lame-encoder` 0.2.5 | https://github.com/DoumanAsh/mp3lame-encoder | [LGPL v3](docs/third-party/mp3lame-encoder-0.2.5-LICENSE.txt) |
| `mp3lame-sys` 0.1.11 | https://github.com/DoumanAsh/mp3lame-sys | [LGPL v3](docs/third-party/mp3lame-encoder-0.2.5-LICENSE.txt), byte-identical license text to the wrapper |
| LAME 3.100 (vendored by mp3lame-sys) | https://lame.sourceforge.io/ | [COPYING](docs/third-party/LAME-3.100-COPYING.txt) and [LICENSE](docs/third-party/LAME-3.100-LICENSE.txt) |

The license files were copied unchanged from the exact Cargo registry sources
used in the Windows build. Cargo.lock records the crate checksums. The LAME
source and the wrapper were not modified. LAME is developed by the LAME team;
the Rust bindings/wrapper are maintained by DoumanAsh and their contributors.

This record is implementation provenance, not a claim that a distributable
installer has completed license integration. Before distributing a build with
these statically linked libraries, include their applicable license/copyright
notices and provide the source and application relinking materials required by
their licenses, with reproducible instructions for the exact release build.
The desktop encoder packaging overlay includes these retained license texts,
this notice and the application license. Source/relink permissions and rebuild
proof still gate public redistribution. That work and Mac verification are recorded in the
[live release gate](docs/plans/beta-go-no-go.md).
