# Desktop MP3 dependencies

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
The current local app was neither packaged nor released by this change. That
distribution work and Mac encoder build verification are recorded in the
[live release gate](docs/plans/beta-go-no-go.md).
