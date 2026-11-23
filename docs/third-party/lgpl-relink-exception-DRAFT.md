# LGPL relinking permission — APPROVED 2026-11-23 (now `LICENSE` section 7)

This draft is for the release owner's review. It has not changed `LICENSE`,
has not been granted to recipients, and must not be represented as an approved
distribution term. The current source-available license remains unchanged.

> **2026-11-23 — approved by the owner.** Dan Kinsner authorized merging branch
> `vera/launch-tie-up` into `main` ("push the new vera/tie up … to main"). The
> wording below is in force as `LICENSE` section 7 from commit `25a118d`
> (on `main` as of `ec8b56f`), unchanged except that "this exception" reads
> "this section" and the components are identified by `THIRD_PARTY_NOTICES.md`.
> Still required before any public release: regenerate the redistribution
> materials and release evidence for the changed terms. The scripts no longer
> list this permission as pending by constant: they read it from the archived
> `LICENSE` (`scripts/lib/redistribution-readiness.mjs`), so a release run on a
> revision that contains section 7 records it as present.
> The text beneath this note is the historical draft and is kept for the record.

Proposed narrowly scoped exception for the release notice:

> Notwithstanding the restrictions in the YES Master Source-Available License,
> recipients may modify the LGPL-covered library components, and modify and
> rebuild or relink the portions of YES Master necessary to use those modified
> components, for their own use. Recipients may reverse engineer YES Master
> to the extent necessary to debug those library modifications. The supplied
> source and build materials may be used for that purpose. This exception does
> not restrict any rights independently granted under the licenses of those
> third-party components. No permission to redistribute YES Master is otherwise
> granted by this exception.

Before approval, compare this wording and the actual source/build package to
the applicable LGPL versions, including the `mp3lame-encoder` and `mp3lame-sys`
LGPL v3 terms. This is a concrete release prerequisite proposal, not a legal
compliance certification. Packaging alone cannot grant the missing permission.
