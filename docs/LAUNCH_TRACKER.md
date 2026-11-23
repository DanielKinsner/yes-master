# YES Master — Launch Tracker (public beta)

One page. What stands between `main` and an open public beta, who owns each
item, and its state. Started 2026-11-23 on branch `vera/launch-tie-up`.
The authoritative gate record is still
[plans/beta-go-no-go.md](plans/beta-go-no-go.md) and
[OPEN_THREADS_AND_DECISIONS.md](OPEN_THREADS_AND_DECISIONS.md); this page is the
short view. Nothing here publishes, merges or approves anything.

## The owner's five

| # | Item | State |
|---|---|---|
| 1 | **Listen.** One compressed export against the in-app master on the installed build. | open |
| 2 | **Approve the relinking permission.** Prepared as `LICENSE` section 7 on this branch (`25a118d`). Merging is the approval. | prepared, not approved |
| 3 | **A contact address that receives mail.** See finding C4. | open |
| 4 | **Updater key.** Independent backup, and a proven restore on the other machine. | open |
| 5 | **Publish.** Approve the final tag and site activation. | open |

## Done on this branch (2026-11-23)

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
| C2 | **21 open Dependabot alerts on the default branch.** Runtime: `zip` (1 critical, 1 high, 1 medium), `glib` (5 medium), `serde_with` (4 medium), `rustls` (1 medium, 3 low). Development only: `vite`, `@vitest/mocker`, `source-map-js`. | GitHub Dependabot API, 2026-11-23 | Start with `zip`: find which crate pulls it in and whether the updater's archive handling uses the affected code; update and rerun the suites. |
| C3 | **Price on the live site vs the record.** yesdsp.com shows "Planned launch pricing: $29 founder / $49 standard". `beta-go-no-go.md` says pricing was reopened on September 14 and those figures are "historical, not approved purchase terms". | live page, 2026-11-23 | Owner: confirm the figures or take the line off the page. |
| C4 | **`hello@yesmaster.app` cannot receive mail.** The domain is registered (Cloudflare name servers) with no MX record. `yesdsp.com` (Vercel DNS) has none either. The address appears in the three legal drafts. | DNS lookups, 2026-11-23 | Owner's choice (2026-11-23): use `yes@yesdsp.com`. `yesdsp.com` DNS is at Vercel, which does not forward mail, so it needs MX records for a forwarding service. When the address receives mail, replace the old one in the legal drafts. |
| C5 | **The legal drafts are paid-product templates.** `EULA.md`, `PRIVACY.md`, `REFUND-POLICY.md` carry unfilled brackets and describe license keys and purchases the beta does not have; the EULA says not to publish it yet. | the files | Keep them unpublished for the beta; review `BETA-TERMS.md` instead. |
| C6 | **Windows publisher signing.** Not required for the beta; the install guide documents the SmartScreen notice. `release.yml` already has optional Azure signing inputs. | `release.yml`, `BETA_INSTALL.md` | Parked by the owner. No action. |
| C7 | **No cold install on record.** I found no note of the installer being run on a computer that never had the development tools. | search of `docs/` | Have someone else install it with no help, and watch. |
| C8 | **Trademark.** No search of the USPTO register is recorded. | search of `docs/` | Optional before a paid release: search the register for the name in software classes. |
| C9 | **Promo and tour videos are on one computer only.** Branch `feat/promo-video` (5 commits, 2026-11-14) exists only in the home Windows clone. | `git branch -r --contains` | Owner's verdict on the videos first; then push or drop. |
| C10 | Still open from the September record: the external-interface (Focusrite) check, the hiccup at the end of preview preparation, one virtual endpoint that fails to open. | `OPEN_THREADS_AND_DECISIONS.md` | Unchanged. |

## What this page does not claim

No listening verdict, no legal review, no installed-build evidence and no
release authorization. Checks marked "verified" say exactly what was compared.
