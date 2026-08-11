# VirusTotal CI Scan — Design

**Date:** 2026-08-11
**Status:** Approved (Approach A)
**Scope:** Add a post-build VirusTotal scan to the release workflow and publish results to the GitHub release.

## Context

`release.yml` builds a Tauri desktop app (PAS Reconciliation) on a 4-platform matrix
(macOS aarch64, macOS x86_64, ubuntu-22.04, windows-latest) via `tauri-apps/tauri-action@v0.5.19`.
The action both builds installers and uploads them to the GitHub release in a single step,
triggered on `push: tags v*`.

We want every release installer scanned by VirusTotal (VT), with the results published on the
release — both as a downloadable report asset and as a section in the release notes.

## Decisions (confirmed with user)

| Decision | Choice |
|---|---|
| VT API tier | **Free / public API** (32 MB upload cap, ~4 req/min) |
| Scan scope | **All platform installers** (DMG, .exe, .msi, .deb, .AppImage, .rpm) |
| Publish format | **Both** — report file as release asset **and** links/summary in release notes |
| On detection | **Informational only — never fail the release** |

## Architecture

A new dedicated job, `virustotal-scan`, runs after the build matrix completes.

```
build-tauri  (4-platform matrix, unchanged)
        │   needs: build-tauri
        │   if: always()
        ▼
virustotal-scan  (ubuntu-latest)
   1. gh release download <tag>      (filtered to installers)
   2. python .github/scripts/vt_scan.py
   3. gh release upload   <tag>      VIRUSTOTAL-REPORT.md
   4. gh release edit      <tag>     (append VT section to release notes)
```

**Why a dedicated job (not inline per matrix leg):** the free VT key is rate-limited to ~4
requests/minute. Four parallel build jobs hitting VT simultaneously would trip 429s. A single
job with in-script rate limiting is the only layout that reliably stays under quota, and it
produces one consolidated report.

## Components

### 1. `virustotal-scan` job (in `.github/workflows/release.yml`)

- `runs-on: ubuntu-latest`
- `needs: build-tauri`
- `if: always()` — scan whatever was built even if one platform failed; missing assets are
  simply skipped. (A wholly failed matrix still skips the report gracefully.)
- `continue-on-error: true` — the job never fails the workflow.
- Env: `VIRUSTOTAL_API_KEY: ${{ secrets.VIRUSTOTAL_API_KEY }}`, `GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}`.
- Steps:
  1. `actions/checkout@v4` (to get the script).
  2. `gh release download "$TAG"` with `--pattern` filters for installer extensions,
     `--dir scan-assets`. (`TAG` = `${{ github.ref_name }}`.)
  3. `python .github/scripts/vt_scan.py --assets-dir scan-assets --tag "$TAG"`
     → writes `VIRUSTOTAL-REPORT.md`.
  4. `gh release upload "$TAG" VIRUSTOTAL-REPORT.md --clobber` (replaces any prior report).
  5. `gh release edit "$TAG" --notes-file <updated-notes>` after the script appends the VT
     section to the existing body (see Publishing).

### 2. `.github/scripts/vt_scan.py` (new)

Python 3 (preinstalled on ubuntu-latest), stdlib only (`urllib`, `hashlib`, `json`, `time`,
`argparse`, `pathlib`) — no dependencies to install.

Responsibilities:

- **Enumerate** installers in `--assets-dir` (filter by extension below).
- **Rate limiting:** a module-level `min_interval = 16 s` between any VT API call (global guard
  since this is the only job using the key), plus ±3 s jitter. Exponential backoff + retry on
  HTTP 429 (up to 3 retries).
- **Per-file logic:**
  1. Compute SHA-256.
  2. `GET /api/v3/files/{sha256}` (hash lookup — cheap, instant, no upload).
     - **Known** → use the cached analysis (`last_analysis_stats`, permalink).
     - **Unknown (404)** → if file size ≤ 32 MB: `POST /api/v3/files` (multipart upload) →
       capture `analysis_id`; poll `GET /api/v3/analyses/{id}` up to 8 × 30 s.
       If still queued when polling budget is exhausted, record the permalink only — VT
       completes asynchronously and the link resolves retroactively.
     - **Unknown and > 32 MB** → record as *"oversized for free tier — not uploaded"* with
       the SHA-256 so a human can look it up later. No upload attempted.
- **Report writing:** emit `VIRUSTOTAL-REPORT.md` with:
  - Header: release tag, scan date, VT tier note.
  - Overall summary line (files scanned, total detections across all engines).
  - Per-file table: Name | Platform | SHA-256 (short) | Size | Detections | Status | VT link.
- **Release-notes append:** fetch current body via `gh release view --json body`, append a
  `## VirusTotal Scan` section (summary line + link to `VIRUSTOTAL-REPORT.md` + per-file VT
  links), write to a temp file, and the workflow step calls `gh release edit --notes-file`.
  The script handles the append so the workflow step is a single `gh release edit`.
- **Never exits non-zero.** All errors (bad key, network, 429s exhausted, oversized) are
  captured into the report as warnings; the workflow's `continue-on-error` is a second line
  of defense.

### Artifact selection

Include (matches "all platform installers"): `.dmg`, `.exe`, `.msi`, `.deb`, `.AppImage`, `.rpm`.
Exclude: `.sig`, `.tar.gz` (updater bundles), `latest.json`, `.txt`, anything else.

## Data flow

```
release assets ──gh release download──> scan-assets/
                                              │
                                     vt_scan.py (hash-first,
                                     upload-if-needed, rate-limited)
                                              │
                                  ┌───────────┴────────────┐
                                  ▼                        ▼
                         VIRUSTOTAL-REPORT.md     updated release notes
                                  │                        │
                       gh release upload          gh release edit
```

## Error handling

- **Rate limit (429):** retry up to 3× with exponential backoff; if exhausted, log warning,
  continue. Job does not fail.
- **Oversized file (>32 MB) on free tier:** hash-only lookup; if unknown, record as oversized,
  never attempt upload. Job does not fail.
- **Missing assets (a build leg failed):** scan whatever is present; note missing platforms in
  the report from the release's asset list.
- **Bad / missing API key:** report header notes "scan skipped — API key unavailable"; job does
  not fail.
- **Detection found:** purely informational — recorded in report and notes; no workflow effect.

## Permissions & secrets

- New repo secret: `VIRUSTOTAL_API_KEY`.
- `permissions: contents: write` is already set at the workflow level and covers both asset
  upload and notes editing with the auto-provided `GITHUB_TOKEN`. No PAT needed.
- Trigger unchanged.

## Files touched

- `.github/workflows/release.yml` — add `virustotal-scan` job.
- `.github/scripts/vt_scan.py` — new.
- This spec.

## Out of scope (YAGNI)

- Per-engine breakdown tables (VT permalink already shows them).
- Badge images / shields.io integration.
- Scanning updater `.tar.gz` bundles or `.sig` files.
- Running scans on PRs or main pushes (release tags only).
- Caching VT hashes across releases.
- Premium-tier large-file upload URL handling.
