# Review inputs

This folder supplies a production source snapshot, eight audio inputs and one
recorded production export. Source hashes and the baseline revision are in
SOURCE_SNAPSHOT.json. The supplied source has no intentional implementation edits.
The repository history, project reports and earlier experiment tooling are absent.

The fixtures manifest records source identity, licenses, preparation and baseline
export settings. It contains no expected audio measurements or preferred candidate.
The baseline export exists to check faithful engine reproduction, not to dictate
the audit conclusion. Do not infer current defect status from fixture selection.

Four public inputs are finished MP3 downloads decoded to floating-point WAV;
three are constructed sums of supplied stems. The latter are not artist-approved
premaster balances and may contain previously processed individual stems. One
input is owner-supplied, with earlier processing history unspecified. This is a
convenience set with limited creator coverage, not a representative population.
Original creator credit and license links are in fixtures/manifest.json. Sources
were decoded without clipping float headroom; the stem sums use the documented
scalar preparation. No creator endorsement of YES or experimental outputs is implied.

The supplied baseline was generated on Windows. Use existing local Rust, Python,
FFmpeg/FFprobe and other standard tools as needed, recording exact versions.
The application dependency locks are included. Measure with independent tools
where useful and investigate disagreements instead of assuming either tool is right.

Public licensing permits the documented local analysis/adaptations with credit;
the owner source/export is authorized only for this private investigation. No
public distribution, third-party audio upload or paid service is authorized.

Create audit-output/ for your own protocol, experiments and sealed report.
Follow START_REVIEW.md. Do not consult another YES workspace for context.
