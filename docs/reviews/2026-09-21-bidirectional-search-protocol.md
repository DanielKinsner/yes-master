# Bounded bidirectional development search v1

Frozen before new candidate audio on `codex/mastering-bidirectional-research`,
from pushed main `b69277b8`. This is a known-source C1 experiment. It preserves
all prior reports and the `control-inclusive-preserving-v1` implementation.

## Rule and scope

New selector version: `control-inclusive-bidirectional-v1`. Use the same
source/settings/processing context, complete-file technical verification,
source anchors, C1 character limits and 0.2 LU target tolerance as before.
Current processing remains an eligible candidate. Start with current and the
normalized zero-offset single. If either qualifies for character and target,
stop. Otherwise evaluate exactly two additional offsets, -3 and +3 dB relative
to the single, then select from all four. Evaluate both directions before
selecting; never infer monotonicity or keep searching until a pass appears.

Among character-qualified target hits prefer current, then smallest absolute
offset, then negative before positive, then ID. With no qualified target hit,
choose smallest absolute target error among character-qualified rows with the
same tie order. With no character-qualified result, retain the technically
verified current control and explicit `character_fallback`. Invalid control
with no qualified alternative yields `no_verified_fallback`. Unavailable
character data fails qualification. Budget exhaustion is explicit and does
not promote a failed row to PASS. Duplicate identities, mixed intent and
incomplete directional pairs are errors.

Apply this checkpoint to **Piano, Imaginal, Metal and Rich**, Universal 50,
original source levels, **-14 LUFS**. These are exactly the four unresolved
groups in the existing eight-source -14 comparison. Reuse their eight complete
current/single outputs and Metal's already verified +3 output. Render only
the **seven missing complete outputs**: both directions for Piano/Imaginal/Rich
and -3 for Metal. No -9, extra offsets, intensity/preset changes or new sources.
Other four qualified groups retain their existing simple-rule results.

Bind retained native/character/independent reports, source WAVs, settings,
drive, coefficients, source metrics, copied executable and this protocol by
hash before rendering. Historical f32 outputs supply settings/anchors only;
candidate PCM and controls use the corrected production chain/SRC/finalizer.
Reject mismatches before audio. Do not replace any prior artifact.

## Verification and cost

Use the same 48 kHz stereo float, -1 dBTP complete-file delivery checks:
SOXR16/64 independent reconstructed peaks, independent LUFS, native qualified
ceiling, exact frame/rate/channel, finite/full-scale and hashes. Recompute
retained constraints from their saved source/metric facts, and rehash retained
WAVs. Test context/grid/order/rejection/fallback logic before audio.

Run native renders alone, followed by character measurement and independent
references. Decode/normalize once per new source batch. Retain each candidate,
not just the winner. Record new versus reused costs separately: source
preparation, chain, SRC, finalization, metrics, references, child wall/CPU/memory.
The conceptual four-evaluation cost includes the historical current/single and
Metal +3 costs; this is an observed-work comparison, not a paired benchmark.

Hard work bound: two extra evaluations per unresolved source and seven new
files total. Keep 25 GiB free plus estimated new WAV size before execution.
Provisional engineering qualification: each directional pair's summed
chain/SRC/finalization work must be <=1.5 times its retained current/single
pair (total <=2.5 times); native child peak working set <=2 GiB. Report a
budget failure without relaxing limits or discarding its audio evidence. No
further rendering is authorized by this protocol if the benefit is absent.

Processing selection and delivery gain remain separate: this checkpoint tests
search at one target, not -14 as a universal preparation reference. Prepared
buffer/measurement reuse stays a separately verified optimization. These are
offline costs, not import/settling/playback/session latency. Full quiet-copy
coverage, source/preset/manual/no-target/Off/Adapt/Album/save contracts, C2
licensed holdout and C3 native adoption remain open. No shipping sonic policy,
listening preference, preset retuning or gated-feature activation follows.
