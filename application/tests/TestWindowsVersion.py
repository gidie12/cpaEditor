import unittest

from application.helper_classes.windowsVersion import file_version
from application.version import __version__


class TestWindowsVersion(unittest.TestCase):

    def test_file_version_pads_to_four_numbers(self):
        """Test: file_version_pads_to_four_numbers"""
        self.assertEqual(file_version('1.0.2'), (1, 0, 2, 0))
        self.assertEqual(file_version('2'), (2, 0, 0, 0))
        self.assertEqual(len(file_version(__version__)), 4)

    def test_file_version_leaves_out_label(self):
        """Test: file_version_leaves_out_label"""
        self.assertEqual(file_version('1.2.3-rc.1'), (1, 2, 3, 0))
        self.assertEqual(file_version('1.0.5+nightly.20261029.abc1234'), (1, 0, 5, 0))

    def test_file_version_limits_numbers(self):
        """Test: file_version_limits_numbers"""
        self.assertEqual(file_version('1.2.3.4.5'), (1, 2, 3, 4))
        self.assertEqual(file_version('1.70000.0'), (1, 65535, 0, 0))


if __name__ == '__main__':
    unittest.main()
