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
