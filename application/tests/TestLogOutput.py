import unittest
from unittest.mock import patch
import tkinter as tk

from application import CpaEditorApp as cpa_editor_app


class TestLogOutput(unittest.TestCase):

    def setUp(self):
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            self.app = cpa_editor_app.CpaEditorApp()
        self.app.geometry('1200x800')
        self.app.update()

    def tearDown(self):
        # The app logs into its own Text widget, which no longer exists after destroy
        for handler in list(cpa_editor_app.logger.handlers):
            cpa_editor_app.logger.removeHandler(handler)
        self.app.destroy()

    def test_log_is_visible_at_start(self):
        """Test: log_is_visible_at_start"""
        self.app.show_log()
        self.app.update()
        # Room for the five lines of the log, although the tabs ask for more height than the window has
        self.assertGreater(self.app.log_output.winfo_height(), 40)

    def test_log_can_be_made_larger_and_smaller(self):
        """Test: log_can_be_made_larger_and_smaller"""
        pane_height = self.app.log_pane.winfo_height()
        self.app.log_pane.sash_place(0, 0, pane_height - 300)
        self.app.update()
        large = self.app.log_output.winfo_height()
        self.assertGreater(large, 250)
        self.app.log_pane.sash_place(0, 0, pane_height - 100)
        self.app.update()
        self.assertLess(self.app.log_output.winfo_height(), large)

    def test_log_keeps_height_when_window_is_resized(self):
        """Test: log_keeps_height_when_window_is_resized"""
        self.app.show_log()
        self.app.update()
        height = self.app.log_output.winfo_height()
        self.app.geometry('1200x600')
        self.app.update()
        self.assertEqual(self.app.log_output.winfo_height(), height)

    def test_log_can_be_dragged_with_the_mouse(self):
        """Test: log_can_be_dragged_with_the_mouse"""
        self.app.show_log()
        self.app.update()
        pane = self.app.log_pane
        sash_y = pane.sash_coord(0)[1]
        # The sash is visible and shows a resize cursor
        self.assertEqual(pane.cget('sashcursor'), 'sb_v_double_arrow')
        self.assertGreaterEqual(int(pane.cget('sashwidth')), 6)
        height = self.app.log_output.winfo_height()
        grab_y = sash_y + 3
        pane.event_generate('<ButtonPress-1>', x=100, y=grab_y)
        pane.event_generate('<B1-Motion>', x=100, y=grab_y - 200)
        pane.event_generate('<ButtonRelease-1>', x=100, y=grab_y - 200)
        self.app.update()
        self.assertGreater(self.app.log_output.winfo_height(), height + 150)

    def test_log_has_scrollbar(self):
        """Test: log_has_scrollbar"""
        scrollbars = [widget for widget in self.app.log_frame.winfo_children() if widget.winfo_class() == 'TScrollbar']
        self.assertEqual(len(scrollbars), 1)
        self.app.log_output.insert(tk.END, 'line\n' * 100)
        self.app.log_output.yview_moveto(1)
        self.app.update()
        # The scrollbar follows the log: at the end, with only part of the lines visible
        first, last = scrollbars[0].get()
        self.assertGreater(first, 0)
        self.assertEqual(last, 1)


if __name__ == '__main__':
    unittest.main()
