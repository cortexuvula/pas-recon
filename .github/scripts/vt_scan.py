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
