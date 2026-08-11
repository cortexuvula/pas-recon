import unittest

import vt_scan  # noqa: F401  (smoke: module imports)


class TestSmoke(unittest.TestCase):
    def test_module_imports(self):
        self.assertTrue(hasattr(vt_scan, "__doc__"))


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
            (td / "App_1.0_x64-setup.exe.tar.gz").write_bytes(b"x")
            (td / "App_1.0_x64-setup.exe.sig").write_bytes(b"x")
            (td / "latest.json").write_text("{}")
            (td / "RELEASE_NOTES.txt").write_text("n")
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
        self.assertTrue(should_upload(32 * 1024 * 1024))
        self.assertFalse(should_upload(32 * 1024 * 1024 + 1))
        self.assertTrue(should_upload(0))

    def test_short_sha(self):
        self.assertEqual(short_sha("abcdef0123456789"), "abcdef012345")
        self.assertEqual(short_sha("short", 12), "short")


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
        self.assertEqual(r.total, 73)
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
        self.assertEqual(r.total, 64)
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
        self.assertEqual(stats, (2, 62))


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
        self.assertIn("3/70", md)
        self.assertIn("oversized", md)
        self.assertIn("https://vt/g/a", md)


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


if __name__ == "__main__":
    unittest.main()
