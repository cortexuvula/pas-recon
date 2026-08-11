import unittest

import vt_scan  # noqa: F401  (smoke: module imports)


class TestSmoke(unittest.TestCase):
    def test_module_imports(self):
        self.assertTrue(hasattr(vt_scan, "__doc__"))


if __name__ == "__main__":
    unittest.main()
