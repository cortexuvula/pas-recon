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
