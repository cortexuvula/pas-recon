# VirusTotal CI Scan Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a post-build VirusTotal scan of all release installers to the release workflow and publish results to the GitHub release (report asset + release-notes section), never failing the release.

**Architecture:** A new `virustotal-scan` job in `release.yml` runs after the build matrix, downloads release installers, and a stdlib-only Python script (`.github/scripts/vt_scan.py`) drives the VT v3 free-tier API with hash-first lookups, uploads-under-32 MB, rate limiting, and graceful oversized/skip handling. The script emits `VIRUSTOTAL-REPORT.md` and an updated `release-notes.md`; the workflow uploads the report and edits the release notes.

**Tech Stack:** Python 3 (stdlib only), GitHub Actions, `gh` CLI, VirusTotal v3 API (free public key).

**Spec:** `docs/superpowers/specs/2026-08-11-virustotal-ci-scan-design.md`

---

## File Structure

- **`.github/scripts/vt_scan.py`** (new) — single-module scan logic. Pure, unit-tested functions for filtering / parsing / report-building / notes / rate-limiting; thin HTTP client; `main()` orchestration with `--dry-run`.
- **`.github/scripts/test_vt_scan.py`** (new) — stdlib `unittest` suite for the pure functions and dry-run path.
- **`.github/workflows/release.yml`** (modify) — add `virustotal-scan` job.
- **`docs/release-setup.md`** (modify) — document the new `VIRUSTOTAL_API_KEY` secret and scan stage.

Tests run with **zero dependencies**: `cd .github/scripts && python3 -m unittest test_vt_scan -v`.

---

### Task 1: Branch + scaffold

**Files:**
- Create: `.github/scripts/vt_scan.py`
- Create: `.github/scripts/test_vt_scan.py`

- [ ] **Step 1: Create feature branch from `main`**

Run:
```bash
git checkout main
git pull --ff-only
git checkout -b ci/virustotal-scan
```

- [ ] **Step 2: Create the module scaffold**

Create `.github/scripts/vt_scan.py`:
```python
#!/usr/bin/env python3
"""VirusTotal scan for release installers (free-tier aware).

Never exits non-zero: every failure is captured into the report.
"""
from __future__ import annotations
```

- [ ] **Step 3: Write the smoke test**

Create `.github/scripts/test_vt_scan.py`:
```python
import unittest

import vt_scan  # noqa: F401  (smoke: module imports)


class TestSmoke(unittest.TestCase):
    def test_module_imports(self):
        self.assertTrue(hasattr(vt_scan, "__doc__"))


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 4: Run test, verify it passes**

Run:
```bash
cd .github/scripts && python3 -m unittest test_vt_scan -v
```
Expected: `OK` (1 test).

- [ ] **Step 5: Commit scaffold + spec**

```bash
git add docs/superpowers/specs/2026-08-11-virustotal-ci-scan-design.md \
        docs/superpowers/plans/2026-08-11-virustotal-ci-scan.md \
        .github/scripts/vt_scan.py .github/scripts/test_vt_scan.py
git commit -m "chore: scaffold VirusTotal scan script + tests [ci/virustotal-scan]"
```

---

### Task 2: Pure helpers (filtering, size, hash)

**Files:**
- Modify: `.github/scripts/vt_scan.py` (append config block + helpers)
- Modify: `.github/scripts/test_vt_scan.py` (append `TestHelpers`)

- [ ] **Step 1: Write the failing tests**

Append to `test_vt_scan.py` (before the `if __name__` guard):
```python
import pathlib
import tempfile

from vt_scan import (
    EXCLUDE_SUFFIXES,
    INCLUDE_EXTS,
    filter_installers,
    should_upload,
    short_sha,
)


class TestHelpers(unittest.TestCase):
    def test_filter_includes_installers_excludes_updater_bundles(self):
        with tempfile.TemporaryDirectory() as d:
            td = pathlib.Path(d)
            (td / "App_1.0_x64.dmg").write_bytes(b"x")
            (td / "App_1.0_x64-setup.exe").write_bytes(b"x")
            (td / "App_1.0_amd64.deb").write_bytes(b"x")
            (td / "App_1.0.AppImage").write_bytes(b"x")
            (td / "App_1.0_x64-setup.exe.tar.gz").write_bytes(b"x")  # updater
            (td / "App_1.0_x64-setup.exe.sig").write_bytes(b"x")     # signature
            (td / "latest.json").write_text("{}")                    # updater manifest
            (td / "RELEASE_NOTES.txt").write_text("n")               # notes
            got = [p.name for p in filter_installers(td)]
        self.assertEqual(
            sorted(got),
            sorted([
                "App_1.0_x64.dmg",
                "App_1.0_x64-setup.exe",
                "App_1.0_amd64.deb",
                "App_1.0.AppImage",
            ]),
        )

    def test_should_upload_threshold(self):
        self.assertTrue(should_upload(32 * 1024 * 1024))        # exactly 32 MB
        self.assertFalse(should_upload(32 * 1024 * 1024 + 1))   # one byte over
        self.assertTrue(should_upload(0))

    def test_short_sha(self):
        self.assertEqual(short_sha("abcdef0123456789"), "abcdef012345")
        self.assertEqual(short_sha("short", 12), "short")
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: FAIL — `ImportError: cannot import name 'EXCLUDE_SUFFIXES'` (or similar).

- [ ] **Step 3: Implement the helpers**

Append to `vt_scan.py`:
```python
# ---- config -------------------------------------------------------------
INCLUDE_EXTS = {".dmg", ".exe", ".msi", ".deb", ".appimage", ".rpm"}
EXCLUDE_SUFFIXES = (".sig", ".tar.gz", ".json", ".txt", ".zip")
FREE_TIER_MAX_BYTES = 32 * 1024 * 1024  # 32 MB upload cap for the public API


# ---- pure: filtering / size / hash --------------------------------------
def filter_installers(asset_dir):
    """Return installer files matching INCLUDE_EXTS, excluding updater bundles."""
    out = []
    for p in sorted(asset_dir.iterdir()):
        if not p.is_file():
            continue
        name = p.name.lower()
        if name.endswith(EXCLUDE_SUFFIXES):
            continue
        if p.suffix.lower() in INCLUDE_EXTS:
            out.append(p)
    return out


def should_upload(size, max_bytes=FREE_TIER_MAX_BYTES):
    return size <= max_bytes


def short_sha(sha, n=12):
    return sha[:n]


def sha256_of(path, chunk=1 << 20):
    import hashlib
    h = hashlib.sha256()
    with path.open("rb") as f:
        for b in iter(lambda: f.read(chunk), b""):
            h.update(b)
    return h.hexdigest()
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: `OK` (4 tests).

- [ ] **Step 5: Commit**

```bash
git add .github/scripts/vt_scan.py .github/scripts/test_vt_scan.py
git commit -m "feat(vt-scan): installer filtering, size threshold, sha helpers"
```

---

### Task 3: VT response parsing

**Files:**
- Modify: `.github/scripts/vt_scan.py` (append `VtResult` + parsers)
- Modify: `.github/scripts/test_vt_scan.py` (append `TestParsing`)

- [ ] **Step 1: Write the failing tests**

Append to `test_vt_scan.py`:
```python
from vt_scan import VtResult, parse_hash_lookup, parse_analysis


class TestParsing(unittest.TestCase):
    def test_parse_hash_lookup_known_clean(self):
        payload = {
            "data": {
                "id": "abc",
                "attributes": {
                    "sha256": "abc123",
                    "size": 12345,
                    "last_analysis_stats": {
                        "malicious": 0, "harmless": 70, "undetected": 3,
                    },
                },
            }
        }
        r = parse_hash_lookup(payload)
        self.assertIsNotNone(r)
        r.name = "app.exe"
        self.assertEqual(r.sha256, "abc123")
        self.assertEqual(r.status, "clean")
        self.assertEqual(r.malicious, 0)
        self.assertEqual(r.total, 73)  # 0+70+3
        self.assertEqual(r.permalink, "https://www.virustotal.com/gui/file/abc123")
        self.assertEqual(r.detection_label, "clean")

    def test_parse_hash_lookup_known_detection(self):
        payload = {
            "data": {
                "id": "z",
                "attributes": {
                    "sha256": "z9", "size": 1,
                    "last_analysis_stats": {"malicious": 4, "harmless": 60},
                },
            }
        }
        r = parse_hash_lookup(payload)
        self.assertEqual(r.status, "detection")
        self.assertEqual(r.malicious, 4)
        self.assertEqual(r.total, 64)  # 4+60
        self.assertEqual(r.detection_label, "4/64")

    def test_parse_hash_lookup_not_found_returns_none(self):
        self.assertIsNone(parse_hash_lookup({}))
        self.assertIsNone(parse_hash_lookup({"data": None}))

    def test_parse_analysis_queued_then_completed(self):
        queued = {"data": {"id": "anid", "attributes": {"status": "queued"}}}
        self.assertEqual(parse_analysis(queued)[0], "queued")
        completed = {
            "data": {
                "id": "anid",
                "attributes": {
                    "status": "completed",
                    "stats": {"malicious": 2, "harmless": 50, "undetected": 10},
                },
            }
        }
        state, stats = parse_analysis(completed)
        self.assertEqual(state, "completed")
        self.assertEqual(stats, (2, 62))  # 2+50+10
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: FAIL — `ImportError: cannot import name 'VtResult'`.

- [ ] **Step 3: Implement parsers**

Append to `vt_scan.py`:
```python
# ---- pure: VT response parsing ------------------------------------------
class VtResult:
    __slots__ = (
        "name", "sha256", "size", "status",
        "malicious", "total", "detail", "permalink",
    )

    def __init__(self, name, sha256, size, status, malicious=0, total=0,
                 detail="", permalink=""):
        self.name = name
        self.sha256 = sha256
        self.size = size
        self.status = status  # clean | detection | queued | oversized | error | skipped
        self.malicious = malicious
        self.total = total
        self.detail = detail
        self.permalink = permalink

    @property
    def detection_label(self):
        if self.status == "detection":
            return f"{self.malicious}/{self.total}"
        return self.status

    def __repr__(self):  # pragma: no cover - debugging aid
        return f"VtResult({self.name!r}, {self.status}, {self.detection_label})"


def _count_engines(stats):
    counted = ("malicious", "suspicious", "harmless", "undetected", "type-unsupported")
    return sum(int(stats.get(k, 0)) for k in counted)


def parse_hash_lookup(payload):
    """Return a VtResult from a GET /files/{id} 200 payload, or None if not present."""
    data = (payload or {}).get("data")
    if not data:
        return None
    attrs = data.get("attributes", {}) or {}
    sha = attrs.get("sha256") or data.get("id") or ""
    size = int(attrs.get("size", 0))
    stats = attrs.get("last_analysis_stats", {}) or {}
    malicious = int(stats.get("malicious", 0))
    total = _count_engines(stats)
    status = "detection" if malicious > 0 else "clean"
    return VtResult(
        name="", sha256=sha, size=size, status=status,
        malicious=malicious, total=total,
        permalink=f"https://www.virustotal.com/gui/file/{sha}",
    )


def parse_analysis(payload):
    """Return (state, (malicious, total)|None) from a GET /analyses/{id} payload."""
    data = (payload or {}).get("data")
    if not data:
        return ("error", None)
    attrs = data.get("attributes", {}) or {}
    state = attrs.get("status", "queued")
    if state != "completed":
        return (state, None)
    stats = attrs.get("stats", {}) or {}
    malicious = int(stats.get("malicious", 0))
    return (state, (malicious, _count_engines(stats)))
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: `OK` (8 tests).

- [ ] **Step 5: Commit**

```bash
git add .github/scripts/vt_scan.py .github/scripts/test_vt_scan.py
git commit -m "feat(vt-scan): VT v3 response parsing (hash lookup + analysis)"
```

---

### Task 4: Report markdown generation

**Files:**
- Modify: `.github/scripts/vt_scan.py` (append `platform_of`, `build_report_md`)
- Modify: `.github/scripts/test_vt_scan.py` (append `TestReport`)

- [ ] **Step 1: Write the failing tests**

Append to `test_vt_scan.py`:
```python
from vt_scan import build_report_md, platform_of


class TestReport(unittest.TestCase):
    def test_platform_of(self):
        self.assertEqual(platform_of("a.dmg"), "macOS")
        self.assertEqual(platform_of("setup.exe"), "Windows")
        self.assertEqual(platform_of("a.msi"), "Windows")
        self.assertEqual(platform_of("a.deb"), "Linux")
        self.assertEqual(platform_of("a.AppImage"), "Linux")
        self.assertEqual(platform_of("a.rpm"), "Linux")

    def test_build_report_md_summary_and_rows(self):
        results = [
            VtResult("app-1.0.dmg", "abcdef0123456789", 5000, "clean",
                     0, 70, permalink="https://vt/g/a"),
            VtResult("setup.exe", "ff", 70000000, "detection",
                     3, 70, permalink="https://vt/g/b"),
            VtResult("big.msi", "11", 40000000, "oversized",
                     detail=">32 MB"),
        ]
        md = build_report_md(results, {"tag": "v0.5.4", "date": "2026-08-11 12:00"})
        self.assertIn("`v0.5.4`", md)
        self.assertIn("2026-08-11 12:00 UTC", md)
        self.assertIn("Free public API (32 MB upload cap)", md)
        self.assertIn("Files scanned: **3**", md)
        self.assertIn("Files with detections: **1**", md)
        self.assertIn("Total engine detections: **3**", md)
        self.assertIn("| app-1.0.dmg | macOS | `abcdef012345` |", md)
        self.assertIn("| setup.exe | Windows |", md)
        self.assertIn("3/70", md)            # detection label inline
        self.assertIn("oversized", md)
        self.assertIn("https://vt/g/a", md)  # permalink
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: FAIL — `ImportError: cannot import name 'build_report_md'`.

- [ ] **Step 3: Implement report generation**

Append to `vt_scan.py`:
```python
# ---- pure: report markdown ----------------------------------------------
def platform_of(name):
    n = name.lower()
    if n.endswith(".dmg"):
        return "macOS"
    if n.endswith((".exe", ".msi")):
        return "Windows"
    if n.endswith((".deb", ".appimage", ".rpm")):
        return "Linux"
    return "?"


def build_report_md(results, meta):
    lines = [
        "# VirusTotal Scan Report",
        "",
        f"- **Release:** `{meta['tag']}`",
        f"- **Scanned:** {meta['date']} UTC",
        "- **Tier:** Free public API (32 MB upload cap)",
        "",
        "## Summary",
        "",
        f"- Files scanned: **{len(results)}**",
        f"- Files with detections: **{sum(1 for r in results if r.status == 'detection')}**",
        f"- Total engine detections: **{sum(r.malicious for r in results)}**",
        "",
        "## Per-file results",
        "",
        "| File | Platform | SHA-256 (short) | Size | Detections | Status | Report |",
        "|---|---|---|---|---|---|---|",
    ]
    if not results:
        lines.append("| _no installers found_ |  |  |  |  |  |  |")
    for r in results:
        size_kb = f"{r.size / 1024:.0f} KB" if r.size else "?"
        link = f"[view]({r.permalink})" if r.permalink else "—"
        lines.append(
            f"| {r.name} | {platform_of(r.name)} | `{short_sha(r.sha256)}` | "
            f"{size_kb} | {r.detection_label} | {r.status} | {link} |"
        )
    lines.append("")
    return "\n".join(lines)
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: `OK` (10 tests).

- [ ] **Step 5: Commit**

```bash
git add .github/scripts/vt_scan.py .github/scripts/test_vt_scan.py
git commit -m "feat(vt-scan): markdown report generation"
```

---

### Task 5: Release-notes section + append

**Files:**
- Modify: `.github/scripts/vt_scan.py` (append `NOTES_HEADER`, `build_notes_section`, `append_notes_section`)
- Modify: `.github/scripts/test_vt_scan.py` (append `TestNotes`)

- [ ] **Step 1: Write the failing tests**

Append to `test_vt_scan.py`:
```python
from vt_scan import (
    NOTES_HEADER,
    append_notes_section,
    build_notes_section,
)


class TestNotes(unittest.TestCase):
    def test_section_lists_each_file(self):
        results = [
            VtResult("a.dmg", "a", 1, "clean", 0, 70, permalink="https://x/a"),
            VtResult("b.exe", "b", 2, "detection", 5, 70, permalink="https://x/b"),
        ]
        s = build_notes_section("VIRUSTOTAL-REPORT.md", results)
        self.assertIn(NOTES_HEADER, s)
        self.assertIn("Scanned 2 installer(s); 1 flagged.", s)
        self.assertIn("`VIRUSTOTAL-REPORT.md`", s)
        self.assertIn("🟢 `a.dmg`", s)
        self.assertIn("🔴 `b.exe`", s)
        self.assertIn("5/70", s)

    def test_append_preserves_existing_body(self):
        existing = "## What's new\n\n- feature\n"
        section = "## VirusTotal Scan\n\nstuff"
        out = append_notes_section(existing, section)
        self.assertIn("## What's new", out)
        self.assertIn("- feature", out)
        self.assertIn("## VirusTotal Scan", out)
        self.assertIn("stuff", out)

    def test_append_is_idempotent(self):
        section = "## VirusTotal Scan\n\nfirst"
        out1 = append_notes_section("body", section)
        out2 = append_notes_section(out1, "## VirusTotal Scan\n\nsecond")
        self.assertEqual(out1.count("VirusTotal Scan"), 1)
        self.assertEqual(out2.count("VirusTotal Scan"), 1)
        self.assertIn("second", out2)
        self.assertNotIn("first", out2)
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: FAIL — `ImportError: cannot import name 'NOTES_HEADER'`.

- [ ] **Step 3: Implement notes functions**

Append to `vt_scan.py`:
```python
# ---- pure: release notes ------------------------------------------------
NOTES_HEADER = "## VirusTotal Scan"


def build_notes_section(report_asset, results):
    flagged = sum(1 for r in results if r.status == "detection")
    lines = [NOTES_HEADER, ""]
    lines.append(
        f"Scanned {len(results)} installer(s); {flagged} flagged. "
        f"Full report: [`{report_asset}`]({report_asset})."
    )
    lines.append("")
    for r in results:
        mark = "🔴" if r.status == "detection" else "🟢"
        link = f"[{r.detection_label}]({r.permalink})" if r.permalink else r.detection_label
        lines.append(f"- {mark} `{r.name}` — {link}")
    return "\n".join(lines)


def append_notes_section(existing_body, section):
    """Append VT section, replacing any previously-added one (idempotent)."""
    parts = (existing_body or "").split("\n" + NOTES_HEADER)
    base = parts[0].rstrip()
    return f"{base}\n\n{section}\n"
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: `OK` (13 tests).

- [ ] **Step 5: Commit**

```bash
git add .github/scripts/vt_scan.py .github/scripts/test_vt_scan.py
git commit -m "feat(vt-scan): release-notes section builder + idempotent append"
```

---

### Task 6: Rate limiting & backoff

**Files:**
- Modify: `.github/scripts/vt_scan.py` (append `compute_backoff`, `RateLimiter`)
- Modify: `.github/scripts/test_vt_scan.py` (append `TestRateLimit`)

- [ ] **Step 1: Write the failing tests**

Append to `test_vt_scan.py`:
```python
from vt_scan import RateLimiter, compute_backoff


class TestRateLimit(unittest.TestCase):
    def test_backoff_grows_and_caps(self):
        self.assertEqual(compute_backoff(0), 1)
        self.assertEqual(compute_backoff(1), 2)
        self.assertEqual(compute_backoff(2), 4)
        self.assertEqual(compute_backoff(10), 60)  # capped

    def test_rate_limiter_enforces_min_interval(self):
        sleeps = []
        times = [100.0]

        def fake_monotonic():
            return times[0]

        rl = RateLimiter(min_interval=16.0, sleep=sleeps.append, monotonic=fake_monotonic)
        # First call: no prior → no sleep.
        rl.wait()
        self.assertEqual(sleeps, [])
        # Second call immediately after: should sleep the full interval.
        rl.wait()
        self.assertEqual(sleeps, [16.0])
        # Advance the clock partway: should sleep only the remainder.
        times[0] = 110.0
        rl.wait()
        self.assertEqual(sleeps[-1], 6.0)
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: FAIL — `ImportError: cannot import name 'RateLimiter'`.

- [ ] **Step 3: Implement rate limiting**

Append to `vt_scan.py`:
```python
# ---- rate limiting ------------------------------------------------------
def compute_backoff(attempt):
    """Exponential backoff seconds for retry `attempt` (0-indexed), capped at 60s."""
    return min(2 ** attempt, 60)


class RateLimiter:
    def __init__(self, min_interval, sleep=None, monotonic=None):
        import time
        self.min_interval = min_interval
        self._sleep = sleep if sleep is not None else time.sleep
        self._monotonic = monotonic if monotonic is not None else time.monotonic
        self._last = float("-inf")

    def wait(self):
        elapsed = self._monotonic() - self._last
        delta = self.min_interval - elapsed
        if delta > 0:
            self._sleep(delta)
        self._last = self._monotonic()
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: `OK` (15 tests).

- [ ] **Step 5: Commit**

```bash
git add .github/scripts/vt_scan.py .github/scripts/test_vt_scan.py
git commit -m "feat(vt-scan): rate limiter + exponential backoff"
```

---

### Task 7: HTTP client (implementation, no network tests)

**Files:**
- Modify: `.github/scripts/vt_scan.py` (append HTTP config + `vt_http`)

Rationale: `vt_http` is a thin wrapper around `urllib` doing real network I/O and multipart
encoding. It is exercised by the real workflow run (Task 9/10) and the `--dry-run` path
explicitly avoids it. We add an import/parse smoke test only.

- [ ] **Step 1: Write the smoke test**

Append to `test_vt_scan.py`:
```python
import inspect

from vt_scan import vt_http


class TestHttpShape(unittest.TestCase):
    def test_vt_http_is_callable(self):
        self.assertTrue(callable(vt_http))

    def test_vt_http_accepts_limiter_kwarg(self):
        sig = inspect.signature(vt_http)
        self.assertIn("limiter", sig.parameters)
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: FAIL — `ImportError: cannot import name 'vt_http'`.

- [ ] **Step 3: Implement the HTTP client**

Append to `vt_scan.py`:
```python
# ---- http client (free-tier aware) --------------------------------------
import json as _json
import random as _random
import urllib.error as _urlerr
import urllib.request as _urlreq

VT_BASE = "https://www.virustotal.com/api/v3"
HTTP_TIMEOUT = 60


def vt_http(method, path, api_key, *, body=None, multipart=None, limiter):
    """Call a VT v3 endpoint and return parsed JSON.

    On HTTP 404 raises urllib.error.HTTPError (callers treat as 'not found').
    On 429 / transient errors, retries with exponential backoff up to 3 times.
    On terminal failure raises RuntimeError.
    """
    url = f"{VT_BASE}{path}"
    headers = {"x-apikey": api_key, "Accept": "application/json"}
    data = None
    if multipart is not None:
        boundary = "----vtscan" + format(_random.randint(0, 1 << 32), "x")
        fname, content = multipart
        head = (
            f"--{boundary}\r\n"
            f'Content-Disposition: form-data; name="file"; filename="{fname}"\r\n'
            f"Content-Type: application/octet-stream\r\n\r\n"
        ).encode()
        data = head + content + f"\r\n--{boundary}--\r\n".encode()
        headers["Content-Type"] = f"multipart/form-data; boundary={boundary}"
    elif body is not None:
        data = body if isinstance(body, bytes) else body.encode()
        headers["Content-Type"] = "application/json"

    last_err = ""
    for attempt in range(4):
        limiter.wait()
        req = _urlreq.Request(url, data=data, headers=headers, method=method)
        try:
            with _urlreq.urlopen(req, timeout=HTTP_TIMEOUT) as resp:
                raw = resp.read().decode() or "{}"
                return _json.loads(raw)
        except _urlerr.HTTPError as e:
            payload = e.read().decode(errors="replace")
            last_err = f"HTTP {e.code}: {payload[:200]}"
            if e.code == 404:
                raise  # caller interprets as 'not found'
            if e.code == 429 and attempt < 3:
                import time
                time.sleep(compute_backoff(attempt))
                continue
            raise RuntimeError(last_err)
        except (_urlerr.URLError, TimeoutError, ConnectionError) as e:
            last_err = str(e)
            if attempt < 3:
                import time
                time.sleep(compute_backoff(attempt))
                continue
            raise RuntimeError(last_err)
    raise RuntimeError(last_err or "http failed")
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: `OK` (17 tests).

- [ ] **Step 5: Commit**

```bash
git add .github/scripts/vt_scan.py .github/scripts/test_vt_scan.py
git commit -m "feat(vt-scan): VT v3 HTTP client with retry + multipart upload"
```

---

### Task 8: Orchestration (`scan_one`, `main`, dry-run)

**Files:**
- Modify: `.github/scripts/vt_scan.py` (append config consts + `scan_one`, `gh`, `main`)
- Modify: `.github/scripts/test_vt_scan.py` (append `TestDryRun`)

- [ ] **Step 1: Write the failing test (dry-run end-to-end, no network)**

Append to `test_vt_scan.py`:
```python
import os
import pathlib
import tempfile

import vt_scan


class TestDryRun(unittest.TestCase):
    def test_dry_run_writes_report_and_notes(self):
        with tempfile.TemporaryDirectory() as d:
            td = pathlib.Path(d)
            (td / "app.dmg").write_bytes(b"hello")
            (td / "app.exe").write_bytes(b"world")
            (td / "latest.json").write_text("{}")  # ignored
            report = td / "REPORT.md"
            notes = td / "NOTES.md"
            rc = vt_scan.main([
                "--assets-dir", str(td),
                "--tag", "v9.9.9",
                "--report", str(report),
                "--notes-file", str(notes),
                "--dry-run",
            ])
            self.assertEqual(rc, 0)
            r = report.read_text()
            self.assertIn("`v9.9.9`", r)
            self.assertIn("app.dmg", r)
            self.assertIn("app.exe", r)
            self.assertNotIn("latest.json", r)
            n = notes.read_text()
            self.assertIn("## VirusTotal Scan", n)
            self.assertIn("`app.dmg`", n)
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: FAIL — `AttributeError: module 'vt_scan' has no attribute 'main'`.

- [ ] **Step 3: Implement orchestration**

Append to `vt_scan.py`:
```python
# ---- orchestration ------------------------------------------------------
POLL_ATTEMPTS = 8
POLL_DELAY = 30.0
MIN_INTERVAL = 16.0


def scan_one(path, api_key, limiter):
    """Scan a single file: hash-first lookup, then upload+poll if needed."""
    name = path.name
    try:
        size = path.stat().st_size
        sha = sha256_of(path)
    except OSError as e:
        return VtResult(name, "", 0, "error", detail=str(e))

    permalink = f"https://www.virustotal.com/gui/file/{sha}"

    # 1. hash-first lookup (cheap, instant)
    try:
        payload = vt_http("GET", f"/files/{sha}", api_key, limiter=limiter)
        hit = parse_hash_lookup(payload)
        if hit is not None:
            hit.name = name
            hit.size = size  # show local size for display
            return hit
    except _urlerr.HTTPError as e:
        if e.code != 404:
            return VtResult(name, sha, size, "error",
                            detail=f"lookup HTTP {e.code}", permalink=permalink)
    except RuntimeError as e:
        return VtResult(name, sha, size, "error", detail=str(e), permalink=permalink)

    # 2. not known → upload if within free-tier size cap
    if not should_upload(size):
        return VtResult(name, sha, size, "oversized",
                        detail=">32 MB; free-tier upload not attempted",
                        permalink=permalink)

    try:
        content = path.read_bytes()
        up = vt_http("POST", "/files", api_key, multipart=(name, content), limiter=limiter)
        analysis_id = ((up.get("data") or {}).get("id") or "")
        if not analysis_id:
            return VtResult(name, sha, size, "error", detail="no analysis id",
                            permalink=permalink)
    except (RuntimeError, _urlerr.HTTPError) as e:
        return VtResult(name, sha, size, "error", detail=str(e), permalink=permalink)

    # 3. poll analysis
    import time as _time
    for _ in range(POLL_ATTEMPTS):
        _time.sleep(POLL_DELAY)
        try:
            p = vt_http("GET", f"/analyses/{analysis_id}", api_key, limiter=limiter)
        except (RuntimeError, _urlerr.HTTPError) as e:
            return VtResult(name, sha, size, "error", detail=str(e), permalink=permalink)
        state, stats = parse_analysis(p)
        if state == "completed" and stats is not None:
            mal, total = stats
            return VtResult(name, sha, size,
                            "detection" if mal else "clean",
                            malicious=mal, total=total, permalink=permalink)

    return VtResult(name, sha, size, "queued",
                    detail="analysis still running; see permalink", permalink=permalink)


def gh(args):
    """Run a gh CLI command; return stdout. Raises on non-zero exit."""
    import subprocess
    res = subprocess.run(["gh", *args], check=True, capture_output=True, text=True)
    return res.stdout


def main(argv=None):
    import argparse
    import datetime as _dt
    import os as _os
    import pathlib
    import random as _random
    import sys as _sys

    ap = argparse.ArgumentParser(description="VirusTotal scan for release installers")
    ap.add_argument("--assets-dir", required=True)
    ap.add_argument("--tag", required=True)
    ap.add_argument("--report", default="VIRUSTOTAL-REPORT.md")
    ap.add_argument("--notes-file", default="release-notes.md")
    ap.add_argument("--report-asset", default="VIRUSTOTAL-REPORT.md")
    ap.add_argument("--dry-run", action="store_true",
                    help="skip network + gh; produce report/notes from local files only")
    args = ap.parse_args(argv)

    assets = filter_installers(pathlib.Path(args.assets_dir))
    api_key = "" if args.dry_run else _os.environ.get("VIRUSTOTAL_API_KEY", "")

    if args.dry_run:
        placeholder = "0" * 64
        results = [
            VtResult(a.name, placeholder, a.stat().st_size, "clean",
                     0, 70,
                     permalink=f"https://www.virustotal.com/gui/file/{placeholder}")
            for a in assets
        ]
    elif not api_key:
        _sys.stderr.write("VIRUSTOTAL_API_KEY not set; emitting skipped report\n")
        results = [VtResult(a.name, "", a.stat().st_size, "skipped",
                            detail="API key unavailable") for a in assets]
    else:
        limiter = RateLimiter(MIN_INTERVAL + _random.uniform(-2, 3))
        results = [scan_one(a, api_key, limiter) for a in assets]

    meta = {
        "tag": args.tag,
        "date": _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%d %H:%M"),
    }
    pathlib.Path(args.report).write_text(build_report_md(results, meta))
    _sys.stderr.write(f"wrote {args.report}\n")

    if args.dry_run:
        existing = ""
    else:
        try:
            data = _json.loads(gh(["release", "view", args.tag, "--json", "body"]))
            existing = (data.get("body") or "")
        except Exception as e:
            _sys.stderr.write(f"could not fetch release body: {e}\n")
            existing = ""

    section = build_notes_section(args.report_asset, results)
    pathlib.Path(args.notes_file).write_text(append_notes_section(existing, section))
    _sys.stderr.write(f"wrote {args.notes_file}\n")
    return 0


if __name__ == "__main__":
    import sys as _sys
    _sys.exit(main())
```

Note: `pathlib` and `_json` are already imported by Task 7's HTTP block; if you prefer, move
all stdlib imports to the top of the file. Local imports inside functions keep each task's
addition self-contained and do not change behavior.

- [ ] **Step 4: Run tests, verify they pass**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: `OK` (18 tests).

- [ ] **Step 5: Commit**

```bash
git add .github/scripts/vt_scan.py .github/scripts/test_vt_scan.py
git commit -m "feat(vt-scan): orchestration (scan_one + main + dry-run)"
```

---

### Task 9: Add `virustotal-scan` workflow job

**Files:**
- Modify: `.github/workflows/release.yml` (append job)

- [ ] **Step 1: Append the job**

Append this job at the top level of `.github/workflows/release.yml` (sibling of `build-tauri`,
indented two spaces, after the `build-tauri` job's closing `args:` line):

```yaml

  virustotal-scan:
    needs: build-tauri
    if: always()
    runs-on: ubuntu-latest
    timeout-minutes: 20
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4

      - name: Download release installers
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          mkdir -p scan-assets
          gh release download "${{ github.ref_name }}" \
            --dir scan-assets \
            --pattern '*.dmg' \
            --pattern '*.exe' \
            --pattern '*.msi' \
            --pattern '*.deb' \
            --pattern '*.AppImage' \
            --pattern '*.rpm' \
            || echo "no matching assets"

      - name: Scan with VirusTotal
        env:
          VIRUSTOTAL_API_KEY: ${{ secrets.VIRUSTOTAL_API_KEY }}
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          # Requires the VIRUSTOTAL_API_KEY repository secret (free public key).
          python .github/scripts/vt_scan.py \
            --assets-dir scan-assets \
            --tag "${{ github.ref_name }}"

      - name: Publish report + release notes
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          gh release upload "${{ github.ref_name }}" VIRUSTOTAL-REPORT.md --clobber
          gh release edit "${{ github.ref_name }}" --notes-file release-notes.md
```

- [ ] **Step 2: Validate YAML parses**

Run:
```bash
python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml')); print('YAML OK')" \
  || (pip3 install --quiet pyyaml && python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml')); print('YAML OK')")
```
Expected: prints `YAML OK`. (If `actionlint` is installed, prefer: `actionlint .github/workflows/release.yml`.)

- [ ] **Step 3: Verify both jobs present**

Run:
```bash
python3 -c "import yaml; d=yaml.safe_load(open('.github/workflows/release.yml')); print(sorted(d['jobs']))"
```
Expected: `['build-tauri', 'virustotal-scan']`.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add VirusTotal scan job to release workflow"
```

---

### Task 10: Document the secret + scan stage

**Files:**
- Modify: `docs/release-setup.md`

- [ ] **Step 1: Read current doc**

Run: `cat docs/release-setup.md`

- [ ] **Step 2: Append a VirusTotal section**

Append to `docs/release-setup.md`:
```markdown

## VirusTotal scan

After the build matrix uploads installers to the release, a `virustotal-scan` job
downloads them, scans each on the **free public VirusTotal API**, and publishes:

- `VIRUSTOTAL-REPORT.md` as a release asset.
- A `## VirusTotal Scan` section appended to the release notes.

The job is `continue-on-error` and the script never exits non-zero — detections are
informational and never block a release.

### Required secret

Add a repository secret named **`VIRUSTOTAL_API_KEY`** with a free public API key from
<https://www.virustotal.com/gui/my-apikey>.

### Free-tier limits

- 32 MB upload cap. Larger installers fall back to a hash-only lookup; if the hash is
  unknown to VT, the file is recorded as "oversized" in the report.
- ~4 requests/minute. The scan job is single and rate-limits itself (≥16 s between calls),
  with exponential backoff on HTTP 429.
```

- [ ] **Step 3: Commit**

```bash
git add docs/release-setup.md
git commit -m "docs: document VirusTotal scan stage + VIRUSTOTAL_API_KEY secret"
```

---

### Task 11: Final verification

- [ ] **Step 1: Full test suite passes**

Run: `cd .github/scripts && python3 -m unittest test_vt_scan -v`
Expected: `OK` (18 tests).

- [ ] **Step 2: Script `--help` works (CLI wired)**

Run: `python3 .github/scripts/vt_scan.py --help`
Expected: usage text listing `--assets-dir`, `--tag`, `--report`, `--notes-file`, `--dry-run`.

- [ ] **Step 3: Local dry-run against a fake assets dir**

Run:
```bash
mkdir -p /tmp/vt-dry && \
  printf 'x' > /tmp/vt-dry/app.dmg && \
  printf 'y' > /tmp/vt-dry/setup.exe && \
  python3 .github/scripts/vt_scan.py --assets-dir /tmp/vt-dry --tag v0.0.0 --dry-run && \
  echo "--- REPORT ---" && cat VIRUSTOTAL-REPORT.md && \
  echo "--- NOTES ---" && cat release-notes.md && \
  rm -rf /tmp/vt-dry VIRUSTOTAL-REPORT.md release-notes.md
```
Expected: both files printed; `app.dmg` and `setup.exe` appear; macOS/Windows platforms labeled.

- [ ] **Step 4: Confirm workflow YAML still valid**

Run:
```bash
python3 -c "import yaml; d=yaml.safe_load(open('.github/workflows/release.yml')); assert 'virustotal-scan' in d['jobs']; print('OK')"
```
Expected: `OK`.

- [ ] **Step 5: Push the branch and open a PR**

Run:
```bash
git push -u origin ci/virustotal-scan
gh pr create --title "ci: VirusTotal scan on release" \
  --body "Adds a post-build VirusTotal scan that publishes VIRUSTOTAL-REPORT.md and a release-notes section. See docs/superpowers/specs/2026-08-11-virustotal-ci-scan-design.md. Never fails the release."
```

- [ ] **Step 6: After merge, validate on the next tag**

Tag a release as usual (`v0.5.5` or similar). On the Actions run, confirm:

- `virustotal-scan` runs after `build-tauri`, is green (or grey/skipped-on-failure) but never red-fails the workflow.
- The release gains `VIRUSTOTAL-REPORT.md` as an asset.
- Release notes contain a `## VirusTotal Scan` section with per-file links.
```
