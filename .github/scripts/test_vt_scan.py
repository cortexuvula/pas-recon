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


if __name__ == "__main__":
    unittest.main()
