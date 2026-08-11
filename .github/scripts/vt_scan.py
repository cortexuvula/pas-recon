#!/usr/bin/env python3
"""VirusTotal scan for release installers (free-tier aware).

Never exits non-zero: every failure is captured into the report.
"""
from __future__ import annotations
