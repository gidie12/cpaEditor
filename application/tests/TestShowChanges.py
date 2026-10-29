import os
import unittest
from unittest.mock import patch
import tkinter as tk

import lxml.etree as Et

from application import CpaEditorApp as cpa_editor_app
from application.app_classes.diff_window import changed_parts, cpa_diff, cpa_text
from application.helper_classes.directories import TESTS_DIR

CPA_FILE = os.path.join(TESTS_DIR, 'resources', 'cpas', 'test_cpa.xml')
CPA_NAMESPACE = 'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd'


class TestShowChanges(unittest.TestCase):

    def setUp(self):
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            self.app = cpa_editor_app.CpaEditorApp()

    def tearDown(self):
        # The app logs into its own Text widget, which no longer exists after destroy
        for handler in list(cpa_editor_app.logger.handlers):
            cpa_editor_app.logger.removeHandler(handler)
        self.app.destroy()

    def last_log_line(self):
        return self.app.log_output.get('1.0', tk.END).strip().splitlines()[-1]

    def diff_text(self):
        return self.app.diff_window.text.get('1.0', tk.END)

    def test_cpa_text_ignores_layout(self):
        """Test: cpa_text_ignores_layout"""
        compact = Et.fromstring('<a><b>1</b><c/></a>')
        spread = Et.fromstring('<a>\n      <b>1</b>\n  <c/>\n</a>')
        self.assertEqual(cpa_text(compact), cpa_text(spread))
        self.assertEqual(Et.tostring(compact), b'<a><b>1</b><c/></a>')

    def test_cpa_diff_shows_changed_line(self):
        """Test: cpa_diff_shows_changed_line"""
        diff_lines = cpa_diff('<a>\n  <b>1</b>\n</a>', '<a>\n  <b>2</b>\n</a>')
        self.assertIn('-  <b>1</b>', diff_lines)
        self.assertIn('+  <b>2</b>', diff_lines)

    def test_cpa_diff_empty_without_changes(self):
        """Test: cpa_diff_empty_without_changes"""
        self.assertEqual(cpa_diff('<a/>', '<a/>'), [])

    def test_changed_parts_marks_whole_words(self):
        """Test: changed_parts_marks_whole_words"""
        old_line = 'tns:cpaid="template_cpa_v01" tns:version="1.0"'
        new_line = 'tns:cpaid="new_cpa_v02" tns:version="1.0"'
        old_parts, new_parts = changed_parts(old_line, new_line)
        self.assertEqual([old_line[start:end] for start, end in old_parts], ['template_cpa_v01'])
        self.assertEqual([new_line[start:end] for start, end in new_parts], ['new_cpa_v02'])
        self.assertEqual(changed_parts('<a/>', '<a/>'), ([], []))

    def test_show_changes_without_cpa(self):
        """Test: show_changes_without_cpa"""
        self.app.changes_button.invoke()
        self.assertIsNone(self.app.diff_window)
        self.assertIn("ERROR - No CPA loaded, no changes to show", self.last_log_line())

    def test_show_changes_without_changes(self):
        """Test: show_changes_without_changes"""
        self.app.load_cpa_data(CPA_FILE)
        self.app.changes_button.invoke()
        self.assertIsNone(self.app.diff_window)
        self.assertIn("INFO - No changes since the CPA was loaded", self.last_log_line())

    def test_show_changes_shows_old_and_new_value(self):
        """Test: show_changes_shows_old_and_new_value"""
        self.app.load_cpa_data(CPA_FILE)
        end = self.app.root.find('{' + CPA_NAMESPACE + '}End')
        old_end = end.text
        end.text = '2031-01-01T00:00:00Z'
        self.app.changes_button.invoke()
        text = self.diff_text()
        self.assertIn(f'-   <tns:End>{old_end}</tns:End>', text)
        self.assertIn('+   <tns:End>2031-01-01T00:00:00Z</tns:End>', text)
        window = self.app.diff_window
        removed = window.text.tag_ranges('removed')
        self.assertIn(old_end, window.text.get(removed[0], removed[1]))
        added_part = window.text.tag_ranges('added_part')
        self.assertEqual(window.text.get(added_part[0], added_part[1]), '2031')
        self.assertTrue(text.startswith('1 line changed since the CPA was loaded.'))

    def test_show_changes_opens_one_window(self):
        """Test: show_changes_opens_one_window"""
        self.app.load_cpa_data(CPA_FILE)
        self.app.root.set('{' + CPA_NAMESPACE + '}cpaid', 'new_cpa_v02')
        self.app.changes_button.invoke()
        first = self.app.diff_window
        self.app.root.set('{' + CPA_NAMESPACE + '}cpaid', 'new_cpa_v03')
        self.app.changes_button.invoke()
        self.assertFalse(first.winfo_exists())
        self.assertIn('new_cpa_v03', self.diff_text())
        self.assertNotIn('new_cpa_v02', self.diff_text())

    def press_on_save_window(self, button_name):
        """Presses Save or Cancel of the changes window as soon as it is shown on save."""
        shown = []
        def press():
            window = self.app.diff_window
            shown.append(window.title())
            shown.append(window.text.get('1.0', tk.END))
            getattr(window, button_name).invoke()
        self.app.after(50, press)
        return shown

    def test_save_shows_changes_and_saves(self):
        """Test: save_shows_changes_and_saves"""
        self.app.load_cpa_data(CPA_FILE)
        self.app.root.set('{' + CPA_NAMESPACE + '}cpaid', 'new_cpa_v02')
        shown = self.press_on_save_window('save_button')
        with patch.object(self.app, 'save_cpa_data') as save, \
                patch.object(cpa_editor_app.filedialog, 'asksaveasfilename', return_value='saved.xml'):
            self.app.save_button.invoke()
        self.assertEqual(shown[0], "Save CPA with these changes?")
        self.assertIn('new_cpa_v02', shown[1])
        save.assert_called_once_with('saved.xml')
        self.assertFalse(self.app.diff_window.winfo_exists())

    def test_save_cancelled_in_changes_window(self):
        """Test: save_cancelled_in_changes_window"""
        self.app.load_cpa_data(CPA_FILE)
        self.app.root.set('{' + CPA_NAMESPACE + '}cpaid', 'new_cpa_v02')
        self.press_on_save_window('cancel_button')
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename') as dialog:
            self.app.save_button.invoke()
        dialog.assert_not_called()
        self.assertIn("INFO - Saving cancelled", self.last_log_line())

    def test_save_without_changes_skips_changes_window(self):
        """Test: save_without_changes_skips_changes_window"""
        self.app.load_cpa_data(CPA_FILE)
        self.assertTrue(self.app.confirm_changes())
        self.assertIsNone(self.app.diff_window)
        self.assertIn("INFO - No changes since the CPA was loaded", self.last_log_line())

    def test_show_changes_window_has_no_save_buttons(self):
        """Test: show_changes_window_has_no_save_buttons"""
        self.app.load_cpa_data(CPA_FILE)
        self.app.root.set('{' + CPA_NAMESPACE + '}cpaid', 'new_cpa_v02')
        self.app.changes_button.invoke()
        self.assertIsNone(self.app.diff_window.save_button)

    def test_saving_keeps_loaded_cpa_as_base(self):
        """Test: saving_keeps_loaded_cpa_as_base"""
        self.app.load_cpa_data(CPA_FILE)
        self.app.root.set('{' + CPA_NAMESPACE + '}cpaid', 'new_cpa_v02')
        with patch.object(self.app, 'save_cpa_data'), patch.object(self.app, 'confirm_changes', return_value=True):
            with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename', return_value='saved.xml'):
                self.app.save_button.invoke()
        self.app.changes_button.invoke()
        self.assertIn('template_cpa_v01', self.diff_text())


if __name__ == '__main__':
    unittest.main()
