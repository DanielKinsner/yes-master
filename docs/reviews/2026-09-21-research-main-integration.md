# Completed research integration

Dan explicitly requested committing and pushing the completed work to main.
This supersedes the earlier local-only push restriction for the completed
September 18 checkpoint, not for subsequent experiments or release activity.

Preflight on September 21 finds clean `codex/mastering-dynamics-research` at
`4c139b4e`, with freshly fetched `origin/main` and local main at `ab420654`.
The research head is 16 commits ahead with no incoming divergence. The original
research branch and unrelated worktrees remain intact.

The 55-file research delta contains tools, internal docs and numerical evidence.
Normal production source is unchanged apart from a four-line `cfg(test)` module
hook in `output_protection.rs`; its benchmark is explicitly ignored. No private
audio is included. The retained 26 September 18 WAVs are not regenerated.

Integration checks: unchanged selector's 11 regressions, Python compilation,
Rust formatting and strict all-target Clippy pass. The three focused output
protection tests pass; the explicitly ignored private benchmark stays ignored.
All saved checkpoint artifact hashes and all 26 whole WAV hashes match.
Unaffected production DSP,
fixture, frontend, bridge, native and package evidence is reused from the
[verified fixes integration](2026-09-16-mastering-fixes-integration.md), under
the scope matrix in `../TESTING.md`. This is not new listening or installer proof.

GitHub CI runs on main pushes; Release runs only on version tags or manual
dispatch. No tag or release dispatch is part of this work. The previous main
commit has a successful Vercel deployment status, so `vercel.json` now sets
`git.deploymentEnabled.main` to false to honor the explicit no-deployment scope.
Other branches retain their existing deployment behavior. Future deployment
work must account for this guard and requires its own authorization. See
[Vercel's branch configuration](https://vercel.com/docs/project-configuration/git-configuration).

Pre-existing remote CI at `ab420654`, run `35362291265`, is not green: Android
SDK setup requests an unavailable `tools` package, RustSec reports a vulnerability,
and encoder lanes fail an Album export assertion. Those files are unchanged by
the research integration. Exact-commit CI for the new push must be reported
separately; passing local research checks do not clear these existing failures.

After the push, continue new research on a separate local `codex/` branch.
The bounded bidirectional selector needs a new frozen version; it must preserve
the old selector, failures, limits, controls and private evidence. No production
sonic policy, preset retuning, gated calibration or holdout result is adopted.
