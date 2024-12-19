import os
import unittest

from application.helper_classes.directories import TESTS_DIR

if __name__ == '__main__':
    loader = unittest.TestLoader()
    suite = loader.discover(f'{TESTS_DIR}', pattern='Test*.py')
    runner = unittest.TextTestRunner()
    runner.run(suite)