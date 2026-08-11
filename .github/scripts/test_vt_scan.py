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


if __name__ == "__main__":
    unittest.main()
