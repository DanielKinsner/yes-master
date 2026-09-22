# Final verification and corrections to both investigations

September 15 UTC / September 14 evening Pacific. Read with the
[authoritative synthesis](2026-09-15-mastering-quality-final-synthesis.md).
This addendum preserves the original reports and records what was actually
checked in the coordinator's final pass. It is not a production change.

## Fresh identity and witness reproduction

- All **18 original blind seal files** and **20 reconciliation seal files**
  match their recorded hashes. The 452 original source-snapshot files also
  match `SOURCE_SNAPSHOT.json`; the key DSP, engine, SRC wrapper and WAV writer
  are byte-identical to the current main checkout.
- `git diff a4fb621 -- src src-tauri` is empty. Root `AGENTS.md` and `CLAUDE.md`
  remain byte-identical. Existing production defaults and gates are unchanged.
- A fresh render through the retained Claude production-path executable again
  reproduces owner On as
  `639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce`.
  This is a new rendering check, not a new build or installed-app check. The
  original Codex fresh-build On/Off/257-frame proof remains separately recorded.
- Five of Claude's pruned critical WAVs were regenerated in a new coordinator
  directory. **All five match its deletion-ledger SHA-256 values**: Funk
  pre-SRC chain, production post-SRC float, no-target PCM24, no-target float,
  and 50 ms Streaming PCM24. No original/sealed file was edited.

Machine-readable results: [regeneration](evidence/2026-09-15-final-review/regeneration.json),
[reference verification](evidence/2026-09-15-final-review/verification.json),
[fixed-reference SRC check](evidence/2026-09-15-final-review/src-focused.json).

## Correction 1: FFT reference construction

Claude's `fft_exact_tp()` passes an even input's Nyquist bin unchanged into a
longer inverse real FFT. That bin must be split when it becomes a positive/
negative-frequency pair. A 0.5-amplitude alternating-sample signal becomes
amplitude 1.0 in the old upsampler, including at the original sample positions.
That is a reference implementation defect, not an audio-engine result.

The final tool halves that bin, explicitly retains original sample peaks,
uses overlap and zero-extends file boundaries. Twelve controls (random signal,
DC and alternating signal at four odd/even lengths) compare original sample
positions and [SciPy's resample](https://docs.scipy.org/doc/scipy/reference/generated/scipy.signal.resample.html)
to a tolerance of 1e-12. They pass. The critical music peaks survive the change.
Finite-block periodic interpolation still depends on boundaries and bandwidth;
neither implementation deserves the unqualified label “exact true peak.”

The final pass remeasured **44 WAVs**, including all 25 original pilot failures,
five newly regenerated witnesses, eight additional pilot controls/candidates
and six synthetic reference files. The corrected reference still places all
25 original failures above −0.9 dB, from about −0.616 to −0.380. Subtracting a
fixed 0.5 dB would leave only 9/25 at or below −1.0, and 21/25 at or below −0.9.
It remains insufficient. Another checked Funk `drive_t14` reads −0.889 dB on
this reference while FFmpeg rounds to −0.9; it was not one of the original
25 failures under the frozen FFmpeg criterion. Do not retroactively change
that criterion/count or claim the 25 files exhaust all reconstruction failures.

Selected new measurements in dB, using the identical regenerated files:

| Witness | Sample peak | Corrected FFT 16× | FFT 32× / doubled overlap | Float-forced SOXR 16× |
| --- | ---: | ---: | ---: | ---: |
| Funk Universal 75 chain at 44.1 kHz, before SRC | −1.104 | +1.662 | +1.681 | −0.043 |
| Same chain after production 48 kHz SRC | +1.043 | +1.091 | +1.104 | +1.091 |
| Funk Universal 50 no-target PCM24 | 0.000 | +0.773 | +0.780 | +0.773 |

The 16→32 comparison changes both oversampling factor and overlap; it is a
combined sensitivity check, not an isolation of either variable. SOXR's much
lower pre-SRC estimate reflects its different near-Nyquist reconstruction
bandwidth. A full-scale-safe sample grid can still contain reconstructed peaks
above full scale. The post-SRC file has actual samples above full scale as well.
The findings support final output protection and detector qualification; they
do not establish what every DAC would reproduce or what an owner would hear.

## Correction 2: the SOXR reference command clipped its own result

Both investigations used a FFmpeg resampling command without pinning floating
sample formats. Format negotiation could clamp above-full-scale oversampled
values. Claude's report correctly noticed a 0 dBFS result on a synthesized
+3 dBTP control, but attributed it too broadly to FFmpeg/SOXR.

The corrected command is:

```text
ffmpeg -hide_banner -nostats -i INPUT.wav -af "aformat=sample_fmts=dbl,aresample=OUTPUT_RATE_TIMES_16:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0" -c:a pcm_f64le -f null -
```

For example, the post-SRC Funk reference changes from approximately 0.000 to
+1.091 dB; the no-target PCM24 reconstruction changes from approximately 0.000
to +0.773. Those files did not change. Negative-valued old SOXR results are not
therefore all invalid, but the reference command was unsuitable as an unrestricted
peak oracle. Complete measured values, including the synthesized controls, are
in `verification.json`. These controls are not the official full EBU test suite.

## Correction 3: SRC prose overclaims versus raw data

The recorded 1,224 outputs represent **408 conditions**, each rendered through
production, Claude's correction and Codex's correction. The raw result count
confirms 368/408 exact production lengths and 408/408 for each prototype.

Claude's report claims all corrected long residuals are −128…−137 dBc. Its own
`src_probe_summary.txt` reports worst cases about −61…−73 dBc with a 2 ms guard;
the lower figures describe best cases. A finite tone's filter-edge transient,
fitting window and fitted amplitude/phase matter. Do not summarize the entire
matrix with its best residual or count outputs as independent conditions.

Likewise, “corrected short RMS/reference 0.94–1.00” omits other cases. A
single-frame sine at time zero contains zero signal; comparing it with a longer
ideal output sine does not demonstrate lost input energy. Some nondegenerate
short responses also fall below 0.94. The buffer/short-input defects still stand;
the claim that every corrected response matches an infinite sine does not.

The final independent remeasurement uses fixed amplitude/frequency and an
explicit **delay** of half an output sample for the odd-block pairs, with no
amplitude/phase fitting. On two-second 100 Hz inputs:

| Conversion | Production error at the stale frame | Corrected error there (both prototypes) |
| --- | ---: | ---: |
| 48→44.1 kHz, frame 955 | −0.429982 | +0.000001394 |
| 96→44.1 kHz, frame 514 | −0.429715 | +0.000000321 |

The clean-pair 44.1→48 kHz control returns 96,001 frames in production instead
of 96,000; both prototypes return exactly 96,000. A 20 ms interior guard hides
the 96→44.1 stale frame because frame 514 occurs earlier, so the tool checks
that location explicitly. These nine existing waveform checks validate the
mechanism; they do not rerun the complete native matrix or prove every edge.
During construction, this check's initial assertions exposed a reversed delay
sign and the already-known extra production frame; both were corrected in the
test logic without weakening the corrected-output contract.

## Other reconciled limits

- The corpus contains four finished MP3 sources, two MP3-stem sums and one
  FLAC-stem sum in addition to the owner source. Claude's “two lossy sources”
  summary was incorrect. This matters to high-frequency stress interpretation.
- A 0.5 dB fixed internal margin is not demonstrated as a ceiling solution.
  Claude withdrew it after broader replication. A nominal 8×/16× interpolation
  factor does not establish a universal FIR or transient-error bound.
- The proposed +12 dB drive cap and 3–5-point runtime claim are untested. An
  absolute input-trim cap would also compromise the level-invariance objective.
- The idle compression-readout pass in the blind phase covered pre-adaptation
  formulas. The matched follow-up confirms the actual resolved coefficient
  mismatch. Gated diagnostic adaptive plans must not be described as active DSP.
- The old policy summary contains NaN M:S entries; Claude separately computed
  `policy_ms_check.json`. Use the latter for that check, not a blanket stereo
  validity inference from missing summary values.
- Current primary standard status was checked: [BS.1770-5 is in force](https://www.itu.int/rec/R-REC-BS.1770/en)
  and −4 is superseded. [EBU Tech 3341](https://tech.ebu.ch/docs/tech/tech3341.pdf)
  includes additional transient/phase cases 20–23 beyond the synthesized 15–19
examples. Compliance, ceiling enforcement and preferred mastering sound are
separate questions. No full EBU compliance claim is made.

## Transfer and integration checks

The three-part supplement has been fully restored locally to a fresh temporary
two-root workspace: **3,543 unique objects verified, 4,507 paths written**, all
with matching SHA-256. See [local restore proof](evidence/2026-09-15-final-review/local-full-restore.json).
GitHub transport/download-back verification is recorded separately alongside
the supplement manifest: all three downloaded assets and all 3,543 payloads
match, and all 4,507 previously restored paths are identical. See
[GitHub assets](evidence/2026-09-15-final-review/github-assets.json) and
[download-back/restore verification](evidence/2026-09-15-final-review/github-download-restore.json).
The original 147-file September 14 archive remains
unchanged. The [transfer guide](2026-09-15-mastering-quality-transfer.md) gives
the exact commands and selected-bundle limits.

An attempted cleanup of this pass's two temporary verification directories was
rejected by automatic approval review with `blocked by policy`; no deletion ran.
They remain under the original machine's OS temporary directory as
`yes-master-research-restore-20260915` (about 6 GB) and
`yes-master-research-download-20260915` (about 3 GB). They are verified duplicates,
not required on the receiving machine. Claude's original workspace and all
pre-existing research remain intact. No cleanup workaround was attempted.

The restore tool's seven focused checks pass: two-root round trip and identical
resume; verify-only; differing-file protection before writes; corrupt archive;
unsafe path; duplicate destination; wrong member size. Documentation validation
checks links, original seals, source identity, Python syntax, transport manifest
consistency and changed-file scope before pushing.

No production application code was changed or built in this final pass. No
frontend/native gesture, audio deadline, installed product, Mac, lossy output,
listening or release gate is newly passed. The work prepares implementation;
it does not claim that the proposed production fixes have shipped.
