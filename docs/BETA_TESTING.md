# YES Master — Public Beta Testing Guide

Thanks for testing YES Master. This page covers what the beta is, what is worth
your attention, what is known to be rough, and how to send something back that
we can act on.

Installing is covered separately in **[BETA_INSTALL.md](BETA_INSTALL.md)** —
including the OS warnings you will see, and how to verify a download's checksum.

## What this beta is

YES Master is a **local-first desktop mastering app**. It analyses your track,
runs it through a real-time chain you can audition against the original, and
exports a technically checked master. It never modifies your source file.

- **Free, and time-boxed.** The beta runs for a limited period; the end date is
  announced on the landing page when the beta opens.
- **Mac and Windows.** Both ship together. Mobile is not part of this beta and
  there is no mobile download.
- **Nothing leaves your machine.** No telemetry, no accounts, no upload. The
  only data that ever moves is a diagnostics report you choose to save and
  attach yourself.

## What happens when the beta ends

Said plainly, because "beta over" usually means something worse than this:

- New beta downloads stop being offered, and beta support ends.
- **Your installed build keeps working.** Nothing is deactivated remotely and
  nothing stops opening. There is no kill switch.
- A paid 1.0 may follow. If it does, upgrading is optional, and beta testers
  get a founder price. The terms are announced ahead of time — never as a
  countdown in the app.
- You will not be pressured for feedback, and no feature is withheld to
  encourage it.

## What is most useful to test

You do not need to work through this in order. In rough priority:

1. **Audition your own material.** Import a track you know well and switch
   Original ↔ Mastered while it plays. Does it stay in time? Does the level
   comparison feel fair? Turn **Volume Match** on and off and see whether your
   judgment of the tone changes.
2. **Push a preset at a source it does not suit.** Try a dense, already-loud
   master, or something very dynamic. The app is allowed to let you overcook
   your own track — but it should be honest about what it did.
3. **Export, then listen to the file.** Not just the in-app audition. Does the
   exported WAV match what you heard?
4. **Album mode with a real record.** Ordering, per-track Follow/Override, and
   whether the sequence reads at a glance.
5. **The awkward paths.** Cancel an export halfway. Export twice to the same
   folder. Save a project, reopen it. Unplug your interface mid-playback.

Reports about **sound** are as valuable as reports about crashes. "Punch at 60%
sounds hollow on acoustic material" is a genuinely useful bug.

## Known limitations

Not bugs. Please do not spend time reporting these — but do tell us if one of
them is worse than described.

- **The installers are not signed with paid OS certificates.** Windows
  SmartScreen and macOS Gatekeeper will both warn about an unidentified
  developer. Update packages *are* cryptographically signed, and every release
  ships `SHA256SUMS.txt`. See the install guide.
- **Cloud-placeholder files are untested.** A OneDrive Files-On-Demand or
  Dropbox online-only file that is not downloaded locally may stall while the
  OS fetches it. Keep source files local for now. If you hit this, we want to
  hear about it.
- **Projects store absolute paths.** Moving a `.ams.json` to another machine, or
  moving the audio afterwards, restores the project with per-track "source
  missing" errors rather than relinking. Use **Re-analyze** after putting the
  files back. A relink affordance is not built yet.
- **Preset voicing is settled** — the current sound has been listened to and
  signed off. Character notes are still welcome: a future tuning pass only
  opens on new listening notes, and yours are exactly what would justify one.
- **Minimum window size is 1360×740.** Smaller is not supported yet.
- **High display scaling is tight.** The minimum window is 1360×740 logical
  pixels, so on a 1080p display at 150 % scaling the app opens maximized and
  zooms slightly to fit; at 200 % or above the right rail may not fit. Lower
  the display scale for that session if you hit it.

## Diagnostics

**Help → Save diagnostics report** writes a plain-text file containing the app
version, a recent log tail, and a summary of the current session. It is
assembled locally and saved wherever you choose.

The app keeps a small rotating log (about 2 MiB, error paths and lifecycle
events only) in its app-data folder. **There is no telemetry.** Nothing is
transmitted unless you save that report and attach it yourself.

Please attach a diagnostics report to any crash or "it stopped responding"
report — it is usually the difference between a fixable report and a guess.

## Reporting a bug or sending feedback

Both go through this repository's **Issues** page, using one of two forms:

- **Beta bug report** — something is broken, wrong, or crashed.
- **Beta workflow feedback** — something works but is confusing, sounds wrong,
  or gets in your way.

> **Two things to know before you post.**
>
> **GitHub issues are public**, and posting requires a **free GitHub account**.
> There is no YES Master account — we do not have a login system, and you will
> never be asked to make one.
>
> **Do not attach or upload private audio.** Not to an issue, not to a comment.
> If a problem only happens with material you cannot share, describe the file
> instead: sample rate, bit depth, channel count, length, roughly how loud and
> how dynamic it is, and whether it was already mastered. That is almost always
> enough.

If you would rather not use GitHub, that is fine — the beta does not depend on
it, and you are not obliged to report anything at all.

## The optional mailing list

There is an optional email list for beta milestones, focused "we'd like eyes on
this" invitations, and the announcement of the founder-price window.

- **It is entirely optional and sits beside the download, never in front of
  it.** Downloads are never gated on an email address.
- Subscribing changes nothing about how the app behaves.
- Unsubscribe is one click, in every message.
- No urgency tricks, no "give feedback or lose access", no selling addresses.

> **Status: not yet open.** The provider has not been chosen, so the sign-up
> form is present but deliberately inactive — no address is collected and
> nothing is stored. It will be switched on only once the provider, retention
> period, sender identity, and unsubscribe handling are settled and published
> here.

## Updates

YES Master checks for updates in the background and offers a one-click install
when one is available. Update packages are cryptographically signed and the
signature is verified before anything is applied. If you are offline the check
fails silently — it will not interrupt you.

If an update fails, the build you already have keeps working. Reinstalling over
the top from the releases page is always a safe fallback.
