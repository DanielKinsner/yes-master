# Synthetic import fixtures

These small synthetic files support `advertised_extensions_decode`; they contain
no owner audio. The AIFF/AIF aliases were generated from the existing
`synthetic-1s.wav` with the qualified FFmpeg 8.0.3 encoder, `-c:a pcm_s16be`.
Both aliases contain identical AIFF bytes and exercise extension-based probing.
