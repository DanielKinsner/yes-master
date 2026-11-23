# YES Master — Launch Tracker (public beta)

One page. What stands between `main` and an open public beta, who owns each
item, and its state. Started 2026-11-23 on branch `vera/launch-tie-up`; merged to `main` the same day on the owner's word.
The authoritative gate record is still
[plans/beta-go-no-go.md](plans/beta-go-no-go.md) and
[OPEN_THREADS_AND_DECISIONS.md](OPEN_THREADS_AND_DECISIONS.md); this page is the
short view. Nothing here publishes anything.

## The owner's five

| # | Item | State |
|---|---|---|
| 1 | **Listen.** One compressed export against the in-app master on the installed build. | open |
| 2 | **Approve the relinking permission.** `LICENSE` section 7 (`25a118d`), merged to `main` on the owner's word, 2026-11-23. Follow-up: regenerate the redistribution materials for the changed terms. | **approved** |
| 3 | **A contact address that receives mail.** `yes@yesdsp.com` forwards to the owner's inbox (ImprovMX, free plan; MX and SPF records added on Vercel DNS). A test message sent 2026-11-23 arrived. Replies still leave from the owner's own address. | **done** |
| 4 | **Updater key.** Independent backup, and a proven restore on the other machine. | open |
| 5 | **Publish.** Approve the final tag and site activation. | open |

## Done (2026-11-23)

| Commit | What |
|---|---|
| `25a118d` | `LICENSE` section 7: the relinking permission, wording from the reviewed draft. |
| `01f8a36` | The draft notes that it is prepared on a branch and still not approved. |
| `b7fa287` | `docs/legal/BETA-TERMS.md`: short beta terms built only from claims already published. Draft. |
| — | Verified: `YES.Master_0.9.2_x64-setup.exe` in draft `yes-master-v0.9.2-manual-9` matches its line in `SHA256SUMS.txt` (`964715dd…a695`). Only that one asset was checked. |
| — | Web search for another audio product named "YES Master": none found. This is not a trademark search (see C8). |

## Found today — each needs a decision or a fix

| # | Finding | Evidence | Next step |
|---|---|---|---|
| C1 | **The draft release is older than `main`.** Draft manual-9 is built from `e6db1f4`; `main` is 19 commits ahead, 8 of them touching app code. | `git rev-list --count e6db1f4..origin/main` | Build the final candidate from the final commit (after item 2 is settled) and regenerate its evidence. |
| C2 | **21 open Dependabot alerts on the default branch.** Runtime: `zip` (1 critical, 1 high, 1 medium), `glib` (5 medium), `serde_with` (4 medium), `rustls` (1 medium, 3 low). Development only: `vite`, `vitest`, `@vitest/mocker`, `source-map-js`, `rimraf` and three transitive packages. | GitHub Dependabot API and `npm audit`, 2026-11-23 | **`zip`:** it is pulled in only by `tauri-plugin-updater` (2.10.1 → `zip` 4.6.1). The first plugin release on `zip` 9 is 2.14.1. Draft PR [#58](https://github.com/DanielKinsner/yes-master/pull/58) carries that lockfile update so CI can test it. It is not a small change: `tauri` moves 2.11.1 → 2.12.7 with `wry`, `tao` and the Windows bindings, so September's installed-build evidence does not carry over. Not built locally. **Development-only alerts:** `npm audit fix` reports fixes for all eight but failed in the WSL clone with an npm internal error; not done. **`glib`, `serde_with`, `rustls`:** not investigated. |
| C3 | **The live site is a build from 2026-09-18** (`ab42065`). It still shows "Planned launch pricing: $29 founder / $49 standard". `main` removed those figures after pricing was reopened on September 14 (`src/landing/page-copy.json` now says pricing is announced when confirmed). Pushes to `main` since then appear in Vercel as preview deployments; none became production. | Vercel project `yes-master`, 2026-11-23 | No pricing decision is needed for the beta. The hold is deliberate: `vercel.json` sets `git.deploymentEnabled.main` to `false` (since `b69277b`, 2026-09-21), so production changes only when the owner publishes. The stale price line goes away with that deployment. No action before launch. |
| C4 | ~~`hello@yesmaster.app` cannot receive mail.~~ **Resolved 2026-11-23.** The owner chose `yes@yesdsp.com`; the legal drafts use it; forwarding is set up and a test message arrived. `yesmaster.app` still has no mail records and is no longer referenced. | DNS lookups and a delivered test, 2026-11-23 | None. |
| C5 | **The legal drafts are paid-product templates.** `EULA.md`, `PRIVACY.md`, `REFUND-POLICY.md` carry unfilled brackets and describe license keys and purchases the beta does not have; the EULA says not to publish it yet. | the files | Keep them unpublished for the beta; review `BETA-TERMS.md` instead. |
| C6 | **Windows publisher signing.** Not required for the beta; the install guide documents the SmartScreen notice. `release.yml` already has optional Azure signing inputs. | `release.yml`, `BETA_INSTALL.md` | Parked by the owner. No action. |
| C7 | **No cold install on record.** I found no note of the installer being run on a computer that never had the development tools. | search of `docs/` | Have someone else install it with no help, and watch. |
| C8 | **Trademark.** No search of the USPTO register is recorded. | search of `docs/` | Optional before a paid release: search the register for the name in software classes. |
| C9 | **Promo and tour videos are on one computer only.** Branch `feat/promo-video` (5 commits, 2026-11-14) exists only in the home Windows clone. | `git branch -r --contains` | Owner's verdict on the videos first; then push or drop. |
| C10 | Still open from the September record: the external-interface (Focusrite) check, the hiccup at the end of preview preparation, one virtual endpoint that fails to open. | `OPEN_THREADS_AND_DECISIONS.md` | Unchanged. |

## What this page does not claim

No listening verdict, no legal review, no installed-build evidence and no
release authorization. Checks marked "verified" say exactly what was compared.
