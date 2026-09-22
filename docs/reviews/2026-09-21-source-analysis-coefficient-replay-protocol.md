# Source-analysis coefficient replay

Post-audit mechanism check, frozen before replay. The 24-analysis audit finds
two actual-copy coefficient changes and no normalized-copy coefficient changes.
Use only those retained analyses and the twelve verified headroom masters.
Resolve their existing Universal 50 coefficients; create no new audio or source
analysis and change no shipping code or acceptance criterion.

For all twelve normalized analyses, replace only source profile/source LUFS in
the matching headroom candidate's requested settings. Apply that candidate's
same f32 automatic drive and require exact complete coefficient debug identity
with its rendered recipe. Bind normalized PCM identity to the existing static
reference input. Record every mismatch. This checks recipe closure only; it is
not a new native application, listening or end-to-end render test.

For all eight actual quiet copies, replay their existing coefficient resolution
after restoring only their original actual source's LRA field in SourceProfile.
Compare against original coefficients exactly. This is a post-hoc causal
isolation of the observed coefficient changes, not a proposed alteration to
the actual-file LRA measurement. Preserve both original and substituted recipes.
