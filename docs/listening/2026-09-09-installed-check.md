# September 9 installed-build check

Target: Windows YES Master 0.9.2, commit `07021f1b`.
Installed and launched September 9. Help/startup stamp:
`07021f1b · 2026-09-09 08:27`.

**OWNER PASS — September 9, 2026.** After using this installed build, the owner
reported: "actually ran through all those tests. all 5 passed". This closes all
five checks below for the named Windows build: A/B, live settings edits with
Volume Match, transport, heavier import/playback, and saved-file/Album checks as
listed. No additional source filenames, timings or measurement values were
reported; do not invent them or request the same questionnaire again.

This is the approved Windows baseline for the export-format expansion. It does
not certify unimplemented formats, a different installer or Mac, nor publish a
release. Subsequent changes need checks targeted to the behavior they affect.

Use one familiar song and, if available, a longer or higher-sample-rate track.
This is a focused follow-up to the September 5 session, not a repeat of the
full listening questionnaire. Allow roughly 10 minutes.

1. **Play and compare.** Play the familiar song at Universal / 50%. Switch
   Original/Mastered several times, first with Volume Match off, then on.
   The playhead should continue from the same place. Volume Match off can make
   Mastered louder; listen for unwanted clicks, silence or sudden blasts.
2. **Change settings while listening.** Leave Volume Match on. Move Intensity
   from about 20% to 80% and back, change a preset, and adjust Width modestly.
   Sound should respond without dropouts or the old settings-edit level jump.
   Restore your preferred settings afterward.
3. **Use the transport.** Pause/resume, seek, draw and enable a loop, then press
   Return to start. It should leave the loop and return to 0:00 while preserving
   playing/paused state. Switch tracks and confirm the intended track plays.
4. **Check a heavier import.** Import a few tracks including your largest normal
   source. Try playing a ready track while the rest finish. Individual tracks
   should become usable as ready; note any freezes, dropouts or error message.
   A 30–60 second playback check is enough unless a problem appears.
5. **Listen to saved files.** Export WAV and MP3 using the same chosen settings.
   Open both in your usual player and compare a familiar section with the app's
   Mastered playback with Volume Match off. Check for missing/truncated audio,
   obvious distortion or an unexpected level difference. For Album, also export
   two or three reordered tracks and confirm playback and filename order.

All five checks are complete. Retain the numbered procedure as the evidence
scope rather than treating it as another open questionnaire.

## Automated and installation evidence

- Built from a detached checkout of exact `main` commit
  `07021f1b1f90b00c24a7908e65886a639062e9af` with an isolated Cargo target.
  Tauri's initial manifest line-ending rewrite marked the first build dirty;
  restored the unchanged manifest, rebuilt with a clean stamp, and bundled NSIS.
- Silent installation returned 0. Prior executable and session were backed up
  locally; session bytes were unchanged by installation. Installed executable
  SHA-256: `2694e2902d7cd785d0271615ed2784dd35a59ec1f94721fd3dd5bc4015b9d2bc`.
  Installer SHA-256:
  `09070c807e7c062bc9ad793066c34195f6c9753ae01a60277317c68b77b7554a`.
  Installed bytes match the build after exactly the expected three-byte Tauri
  bundle-marker change (`UNK` to `NSS`); the first raw hash comparison correctly
  flagged that difference and it was explicitly investigated before acceptance.
- The installed `AppData/Local/YES Master/yes-master.exe` launched with version
  0.9.2, a responding YES Master window, and the correct startup log stamp. The
  restored track completed native analysis in 1.926 seconds. The updater logged
  its expected unavailable-release skip; no public update path was proved here.
- **240 focused automated tests passed:** 110 frontend transport/state tests,
  22 export-control/Standard/Album frontend tests, 96 audio-module tests,
  9 audio invariants, and 3 MP3/Album/incremental-readiness integration tests.
  Includes the regression that bounds the old warm-VM uncached-edit jump.
- **19 native-engine checks passed** through the real default Windows output,
  `Speakers (Realtek(R) Audio)`: Original/Mastered progress and finite non-silent
  engine metering, six A/B switches preserving position, VM enable/settings
  edits, pause/resume, two observed loop wraps, loop disarm/return to start,
  and 192 kHz Original/Mastered playback. Used the built-in demo and a separate
  12-second upsampled derivative; no owner performance fixture was regenerated.
- Normal engine export produced stereo 48 kHz / 24-bit WAV and 320 kbps MP3.
  Both decoded independently in FFmpeg. WAV: -19.2 LUFS / -2.8 dBTP / 0.9 LU LRA;
  MP3: -19.2 LUFS / -2.9 dBTP / 0.9 LU LRA, agreeing with receipts to displayed
  precision. WAV is 24 seconds; the MP3 container reports 24.024 seconds including
  frame padding. Gapless/processed-PCM contracts passed in the MP3 regressions.
  Source bytes remained unchanged.

**Limits:** the hardware probe calls the exact source revision's real audio
engine outside the installed GUI. It is not acoustic/loopback capture, hardware
callback-deadline certification, long-file stress proof, native mouse-gesture
coverage, or owner listening approval. Installed shell startup and restored-track
analysis were verified separately. The short owner checks above address the
remaining practical listening/interaction questions; Mac installer and release
integration remain separate. This baseline predates the broader format expansion.

Logs, probe source and temporary outputs are outside git under
`test-output/install-07021f1-20260909/`. The isolated installation checkout is
`../yes-master-install-07021f1`; generated builds/fixtures are local only.
