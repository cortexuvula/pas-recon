#!/usr/bin/env python3
"""VirusTotal scan for release installers (free-tier aware).

Never exits non-zero: every failure is captured into the report.
"""
from __future__ import annotations

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
    counted = ("malicious", "suspicious", "harmless", "undetected",
               "type-unsupported", "timeout", "confirmed-timeout", "failure")
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
    # Missing/empty stats means VT has no completed analysis for this file yet.
    # Reporting that as "clean" would be misleading — treat as not-yet-analyzed.
    if not stats or total == 0:
        status = "queued"
    elif malicious > 0:
        status = "detection"
    else:
        status = "clean"
    return VtResult(
        name="", sha256=sha, size=size, status=status,
        malicious=malicious, total=total,
        permalink=f"https://www.virustotal.com/gui/file/{sha}",
    )


def parse_analysis(payload):
    """Return (state, (malicious, total)|None) from a GET /analyses/{id} payload.

    `state` is "completed" (terminal success with stats), "queued" (keep
    polling), or "error-<raw>" for any other status VT reports (e.g. failed),
    so callers surface terminal failures instead of polling forever.
    """
    data = (payload or {}).get("data")
    if not data:
        return ("error", None)
    attrs = data.get("attributes", {}) or {}
    state = attrs.get("status", "queued")
    if state == "completed":
        stats = attrs.get("stats", {}) or {}
        malicious = int(stats.get("malicious", 0))
        return (state, (malicious, _count_engines(stats)))
    if state == "queued":
        return (state, None)
    # Any other status is a terminal failure we should surface, not poll past.
    return (f"error-{state}", None)


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
        safe_name = fname.replace('"', "").replace("\r", "").replace("\n", "")
        head = (
            f"--{boundary}\r\n"
            f'Content-Disposition: form-data; name="file"; filename="{safe_name}"\r\n'
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
                raise
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

    # 2. not known -> upload if within free-tier size cap
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
        if state != "queued":
            # Terminal non-success (e.g. VT reported failed/error) — surface it
            # instead of polling the remaining attempts and reporting "queued".
            return VtResult(name, sha, size, "error",
                            detail=f"analysis {state}", permalink=permalink)

    return VtResult(name, sha, size, "queued",
                    detail="analysis still running; see permalink", permalink=permalink)


def gh(args):
    """Run a gh CLI command; return stdout. Raises on non-zero exit."""
    import subprocess
    res = subprocess.run(["gh", *args], check=True, capture_output=True, text=True)
    return res.stdout


def main(argv=None):
    import argparse
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

    # Top-level guard: the script MUST never fail the release build, even on
    # unexpected exceptions (malformed VT payloads, missing dirs, etc.).
    try:
        return _run(args)
    except Exception as e:
        _sys.stderr.write(f"vt_scan: aborting but not failing build: {e}\n")
        return 0


def _run(args):
    import datetime as _dt
    import os as _os
    import pathlib
    import sys as _sys

    try:
        assets = filter_installers(pathlib.Path(args.assets_dir))
    except Exception as e:
        assets = []
        _sys.stderr.write(f"could not list assets dir: {e}\n")

    api_key = "" if args.dry_run else _os.environ.get("VIRUSTOTAL_API_KEY", "")

    if args.dry_run:
        placeholder = "0" * 64
        results = [
            VtResult(a.name, placeholder, a.stat().st_size, "clean", 0, 70,
                     permalink=f"https://www.virustotal.com/gui/file/{placeholder}")
            for a in assets
        ]
    elif not api_key:
        _sys.stderr.write("VIRUSTOTAL_API_KEY not set; emitting skipped report\n")
        results = [VtResult(a.name, "", a.stat().st_size, "skipped",
                            detail="API key unavailable") for a in assets]
    else:
        # jitter is non-negative so the floor stays at MIN_INTERVAL (>= 16 s)
        limiter = RateLimiter(MIN_INTERVAL + _random.uniform(0, 3))
        results = [scan_one(a, api_key, limiter) for a in assets]

    meta = {
        "tag": args.tag,
        "date": _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%d %H:%M"),
    }
    try:
        pathlib.Path(args.report).write_text(build_report_md(results, meta))
        _sys.stderr.write(f"wrote {args.report}\n")
    except Exception as e:
        _sys.stderr.write(f"could not write report {args.report}: {e}\n")

    # Notes: only rewrite the release body if we actually fetched it; otherwise
    # `gh release edit --notes-file` would replace the whole changelog with just
    # this section. Clear any stale notes file so the edit step fails cleanly.
    if args.dry_run:
        existing = ""
    else:
        existing = None
        try:
            data = _json.loads(gh(["release", "view", args.tag, "--json", "body"]))
            existing = data.get("body") or ""
        except Exception as e:
            _sys.stderr.write(f"could not fetch release body; skipping notes rewrite: {e}\n")
            try:
                pathlib.Path(args.notes_file).unlink(missing_ok=True)
            except OSError:
                pass

    if existing is not None:
        section = build_notes_section(args.report_asset, results)
        try:
            pathlib.Path(args.notes_file).write_text(append_notes_section(existing, section))
            _sys.stderr.write(f"wrote {args.notes_file}\n")
        except Exception as e:
            _sys.stderr.write(f"could not write notes {args.notes_file}: {e}\n")
            try:
                pathlib.Path(args.notes_file).unlink(missing_ok=True)
            except OSError:
                pass

    return 0


if __name__ == "__main__":
    import sys as _sys
    _sys.exit(main())
