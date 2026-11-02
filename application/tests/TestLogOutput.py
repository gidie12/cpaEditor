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

    def drag_sash(self, distance):
        """Drags the bar above the log up by this distance, as the mouse does."""
        sash = self.app.log_sash
        start = sash.winfo_rooty()
        sash.event_generate('<ButtonPress-1>', x=100, y=3, rootx=100, rooty=start)
        sash.event_generate('<B1-Motion>', x=100, y=3, rootx=100, rooty=start - distance)
        sash.event_generate('<ButtonRelease-1>', x=100, y=3, rootx=100, rooty=start - distance)
        self.app.update()

    def test_tabs_are_not_in_a_paned_window(self):
        """Test: tabs_are_not_in_a_paned_window"""
        # In a PanedWindow Windows draws a white bar over the tab titles
        self.assertEqual(self.app.tab_control.winfo_manager(), 'grid')

    def test_log_is_small_at_start(self):
        """Test: log_is_small_at_start"""
        self.assertEqual(int(self.app.log_output.cget('height')), 5)
        self.assertGreater(self.app.log_output.winfo_height(), 40)
        self.assertGreater(self.app.tab_control.winfo_height(), 500)

    def test_log_can_be_dragged_larger_and_smaller(self):
        """Test: log_can_be_dragged_larger_and_smaller"""
        self.assertEqual(self.app.log_sash.cget('cursor'), 'sb_v_double_arrow')
        height = self.app.log_output.winfo_height()
        tabs_height = self.app.tab_control.winfo_height()
        self.drag_sash(200)
        self.assertGreater(self.app.log_output.winfo_height(), height + 150)
        self.assertLess(self.app.tab_control.winfo_height(), tabs_height - 150)
        self.drag_sash(-100)
        self.assertLess(self.app.log_output.winfo_height(), height + 150)

    def test_log_stays_within_limits(self):
        """Test: log_stays_within_limits"""
        self.drag_sash(-1000)
        self.assertEqual(int(self.app.log_output.cget('height')), cpa_editor_app.LOG_MIN_LINES)
        self.drag_sash(5000)
        self.assertGreaterEqual(self.app.tab_control.winfo_height(), cpa_editor_app.LOG_MIN_TABS_HEIGHT - 30)

    def test_log_keeps_height_when_window_is_resized(self):
        """Test: log_keeps_height_when_window_is_resized"""
        self.drag_sash(100)
        height = self.app.log_output.winfo_height()
        self.app.geometry('1200x650')
        self.app.update()
        self.assertEqual(self.app.log_output.winfo_height(), height)

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
