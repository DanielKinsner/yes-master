# Install the YES Master Public Beta

YES Master is in a free public beta. The beta's update packages are
cryptographically signed and each release includes `SHA256SUMS.txt`, but the
Mac and Windows installers do not yet use paid platform certificates. Your OS
will therefore show an unfamiliar-developer warning.

Only download YES Master from the official repository:

<https://github.com/DanielKinsner/yes-master/releases/latest>

## Windows

1. Download the `.exe` installer (recommended) or `.msi`.
2. If SmartScreen says **Windows protected your PC**, choose **More info**.
3. Confirm the app name is **YES Master**, then choose **Run anyway**.
4. Complete the installer and launch YES Master.

## macOS

1. Download the `.dmg` marked for the universal Mac build.
2. Drag YES Master to Applications.
3. Try to open it once. If macOS blocks it, open **System Settings → Privacy &
   Security**.
4. Find the YES Master message, choose **Open Anyway**, then confirm **Open**.

## Optional checksum verification

Download `SHA256SUMS.txt` from the same release. Compare the installer hash:

Windows PowerShell:

```powershell
Get-FileHash -Algorithm SHA256 -LiteralPath '.\YES Master installer.exe'
```

macOS Terminal:

```bash
shasum -a 256 'YES Master installer.dmg'
```

The printed hash must match the line for that filename in `SHA256SUMS.txt`.
If it does not match, do not run the file; report it through the repository's
Issues page.

## Next

Installed and running? **[BETA_TESTING.md](BETA_TESTING.md)** covers what is
most useful to test, the limitations already known about (so you do not spend
time on them), how to save a diagnostics report, and what happens to your
install when the beta ends.

Reporting uses two structured forms on the repository's Issues page — one for
bugs, one for workflow and sound feedback. Both are public and need a free
GitHub account; there is no YES Master account. Please describe your audio
rather than uploading it.
