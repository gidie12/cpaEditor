import unittest
from unittest.mock import Mock, patch
import tkinter as tk

from application.helper_classes.resizeApplication import update_entry_width


class TestUpdateEntryWidth(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.addCleanup(self.master.destroy)
        self.logger = Mock()
        self.entry_fields = [tk.Entry(self.master) for _ in range(3)]

    @patch.object(tk.Tk, 'winfo_screenwidth', return_value=1920)
    def test_update_entry_width_sets_correct_width(self, mock_winfo_screenwidth):
        screen_width = self.master.winfo_screenwidth()
        update_entry_width(self.entry_fields, screen_width)
        for entry in self.entry_fields:
            self.assertEqual(entry.cget('width'), 19)  # Adjusted to match the minimum width logic

    def test_update_entry_width_handles_no_entry_fields(self):
        entry_fields = []
        try:
            update_entry_width(entry_fields, 800)
        except Exception as e:
            self.fail(f"update_entry_width raised an exception: {e}")

if __name__ == '__main__':
    unittest.main()