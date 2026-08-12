# VirusTotal False-Positive Runbook

Every release's `setup.exe` is scanned and published in `VIRUSTOTAL-REPORT.md`. Unsigned
Tauri/NSIS/Rust Windows installers routinely get flagged by a handful of ML/heuristic
engines — this is the expected FP profile, not evidence of malware.

**How to tell a FP from a real detection:** look at the detection *type*. ML/heuristic/PUA
verdicts from a small subset of engines (e.g. Microsoft `!ml`, Sophos `Generic ML PUA`,
SecureAge `Malicious`, Arctic Wolf `Unsafe`) on an unsigned installer, with **zero
signature-based detections** from major engines (Kaspersky, ESET, Bitdefender, Avast,
etc.) = classic false positive.

## Durable fix (do once)

**Code-sign the Windows build.** Defender's ML model weights signing heavily; an
Authenticode certificate (OV or EV) typically drops these flags to zero. This is the only
fix that doesn't have to be repeated each release.

## Per-release remediation (until signed)

Submit the flagged `setup.exe` to the engines with a public FP process. Prioritize
**Microsoft** (largest user base) and **Sophos**. The detection is hash-specific, so submit
each release's binary; unflagging one sample also helps the ML model generalize.

### Current release details (fill in per release)

| Field | v0.5.5 |
|---|---|
| File | `PAS.Reconciliation_0.5.5_x64-setup.exe` |
| SHA-256 | `247568d3de97f98c629e4bddfa72aa6272accf27681dce78e256aa85217236b3` |
| Detection (Microsoft) | `Trojan:Win32/Wacatac.B!ml` |
| Detection (Sophos) | `Generic ML PUA` |
| VT permalink | https://www.virustotal.com/gui/file/247568d3de97f98c629e4bddfa72aa6272accf27681dce78e256aa85217236b3 |
| Direct download | https://github.com/cortexuvula/pas-recon/releases/download/v0.5.5/PAS.Reconciliation_0.5.5_x64-setup.exe |

### 1. Microsoft (highest priority)

1. Go to **https://www.microsoft.com/en-us/wdsi/filesubmission** (shortcut: https://aka.ms/wdsi).
2. Sign in with any Microsoft account.
3. Choose the false-positive path: **"I believe this file should not be detected as malware."**
4. Upload the `setup.exe` (3 MB — under the limit). Microsoft re-analyzes internally; the
   actual file is preferred over just the hash.
5. Fill in:
   - **Detection name:** `Trojan:Win32/Wacatac.B!ml`
   - **SHA-256:** (from the table above)
   - **What it is:** "Unsigned Tauri (Rust + NSIS) desktop installer for an open-source
     application; the `!ml` heuristic flags the NSIS auto-updater/bundler structure."
   - **Verifiable source:** link the GitHub release and repo so the analyst can confirm
     provenance.
6. Submit. You get a tracking ID by email; typical turnaround is a few business days.
7. Reference: [Address false positives/negatives in Microsoft Defender for Endpoint](https://learn.microsoft.com/en-us/defender-endpoint/defender-endpoint-false-positives-negatives).

> Enterprise tip: if you use Microsoft Defender for Endpoint, submit from the
> **Submissions** page in the Defender portal for better tracking and priority.

### 2. Sophos

Sophos [explicitly states](https://support.sophos.com/support/s/article/KBA-000005162) that
"Generic ML PUA — a false positive is possible."

1. Start at [Investigate and resolve a potential false positive (KBA-000005162)](https://support.sophos.com/support/s/article/KBA-000005162).
2. Use [Submit a Sample](https://support.home.sophos.com/hc/en-us/articles/360041664851)
   to send the file as a suspected false positive.
3. Cite detection **`Generic ML PUA`** and the SHA-256.

### After Microsoft/Sophos unflag

- The VirusTotal report keeps the **old** scan results until someone re-scans. With a free
  VT account, open the file's VT page and click **"Re-scan"** so the public report reflects
  the unflagging.
- Unflagging helps the ML model, but the **next release has a different hash** and may be
  flagged again — re-submit per release until the build is signed.

### Skip

SecureAge (`Malicious`) and Arctic Wolf (`Unsafe`) have no robust public FP portal and a
small user base. They clear up as a side effect of signing and/or reputation lift from the
Microsoft/Sophos unflags. Not worth the time.
