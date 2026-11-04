import unittest
from unittest.mock import MagicMock, patch
import tkinter as tk

from application import CpaEditorApp as cpa_editor_app


class TestStartupRedraw(unittest.TestCase):

    def setUp(self):
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            self.app = cpa_editor_app.CpaEditorApp()

    def tearDown(self):
        # The app logs into its own Text widget, which no longer exists after destroy
        for handler in list(cpa_editor_app.logger.handlers):
            cpa_editor_app.logger.removeHandler(handler)
        self.app.destroy()

    def test_redraw_is_planned_at_startup(self):
        """Test: redraw_is_planned_at_startup"""
        self.assertIsNotNone(self.app.startup_redraw)

    def test_redraw_window_repaints_on_windows(self):
        """Test: redraw_window_repaints_on_windows"""
        self.app.host = 'windows'
        user32 = MagicMock()
        user32.GetParent.return_value = 1234
        with patch('ctypes.windll', MagicMock(user32=user32), create=True):
            self.app.redraw_window()
        user32.GetParent.assert_called_once_with(self.app.winfo_id())
        # The frame window and all windows in it are painted again, background included
        user32.RedrawWindow.assert_called_once_with(1234, None, None, 0x0001 | 0x0004 | 0x0080)
        self.assertIsNone(self.app.startup_redraw)

    def test_redraw_window_does_nothing_elsewhere(self):
        """Test: redraw_window_does_nothing_elsewhere"""
        self.app.host = 'mac'
        with patch('ctypes.windll', create=True) as windll:
            self.app.redraw_window()
        windll.user32.RedrawWindow.assert_not_called()


if __name__ == '__main__':
    unittest.main()
