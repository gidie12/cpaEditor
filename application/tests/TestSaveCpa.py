import os
import tempfile
import unittest
from unittest.mock import patch
import tkinter as tk

import lxml.etree as Et

from application import CpaEditorApp as cpa_editor_app
from application.helper_classes.directories import TESTS_DIR

CPA_FILE = os.path.join(TESTS_DIR, 'resources', 'cpas', 'test_cpa.xml')
CPA_NAMESPACE = 'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd'


class TestSaveCpa(unittest.TestCase):

    def setUp(self):
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            self.app = cpa_editor_app.CpaEditorApp()
        self.directory = tempfile.TemporaryDirectory()
        self.file_path = os.path.join(self.directory.name, 'saved.xml')
        # The popup about an unchanged CPAId is answered with Yes unless a test says otherwise
        patcher = patch.object(cpa_editor_app.messagebox, 'askyesno', return_value=True)
        self.ask_unchanged_cpa_id = patcher.start()
        self.addCleanup(patcher.stop)
        # The changes are confirmed with Save; TestShowChanges tests the window itself
        patcher = patch.object(self.app, 'confirm_changes', return_value=True)
        self.confirm_changes = patcher.start()
        self.addCleanup(patcher.stop)

    def tearDown(self):
        self.directory.cleanup()
        # The app logs into its own Text widget, which no longer exists after destroy
        for handler in list(cpa_editor_app.logger.handlers):
            cpa_editor_app.logger.removeHandler(handler)
        self.app.destroy()

    def last_log_line(self):
        return self.app.log_output.get('1.0', tk.END).strip().splitlines()[-1]

    def test_save_button_writes_edited_cpa(self):
        """Test: save_button_writes_edited_cpa"""
        self.app.load_cpa_data(CPA_FILE)
        self.app.root.find('{' + CPA_NAMESPACE + '}End').text = '2031-01-01T00:00:00Z'
        self.app.root.find('{' + CPA_NAMESPACE + '}PartyInfo').set('{' + CPA_NAMESPACE + '}partyName', 'Café Ünïcode')
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename', return_value=self.file_path) as dialog:
            self.app.save_button.invoke()
        self.assertEqual(dialog.call_args.kwargs['defaultextension'], '.xml')
        with open(self.file_path, 'rb') as file:
            content = file.read()
        self.assertTrue(content.startswith(b"<?xml version='1.0' encoding='UTF-8'?>"))
        saved_root = Et.fromstring(content)
        self.assertEqual(saved_root.find('{' + CPA_NAMESPACE + '}End').text, '2031-01-01T00:00:00Z')
        self.assertEqual(saved_root.find('{' + CPA_NAMESPACE + '}PartyInfo').get('{' + CPA_NAMESPACE + '}partyName'),
                         'Café Ünïcode')
        self.assertEqual(len(list(saved_root.iter())), len(list(self.app.root.iter())))
        self.assertIn(f"INFO - CPA saved to {self.file_path}", self.last_log_line())

    def log_text(self):
        return self.app.log_output.get('1.0', tk.END)

    def test_save_dialog_proposes_cpa_id_as_file_name(self):
        """Test: save_dialog_proposes_cpa_id_as_file_name"""
        self.app.load_cpa_data(CPA_FILE)
        self.app.root.set('{' + CPA_NAMESPACE + '}cpaid', 'partner_a/partner_b:v2')
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename', return_value='') as dialog:
            self.app.save_button.invoke()
        self.assertEqual(dialog.call_args.kwargs['initialfile'], 'partner_a_partner_b_v2.xml')
        self.assertNotIn("has not been changed", self.log_text())
        self.ask_unchanged_cpa_id.assert_not_called()

    def test_save_warns_when_cpa_id_not_changed(self):
        """Test: save_warns_when_cpa_id_not_changed"""
        self.app.load_cpa_data(CPA_FILE)
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename', return_value='') as dialog:
            self.app.save_button.invoke()
        self.assertEqual(dialog.call_args.kwargs['initialfile'], 'template_cpa_v01.xml')
        self.assertIn("WARNING - CPAId 'template_cpa_v01' has not been changed since the CPA was loaded",
                      self.last_log_line())
        self.ask_unchanged_cpa_id.assert_called_once()
        self.assertEqual(self.ask_unchanged_cpa_id.call_args.args[0], "CPAId not changed")
        self.assertIn("'template_cpa_v01'", self.ask_unchanged_cpa_id.call_args.args[1])

    def test_save_stops_when_unchanged_cpa_id_is_not_confirmed(self):
        """Test: save_stops_when_unchanged_cpa_id_is_not_confirmed"""
        self.app.load_cpa_data(CPA_FILE)
        self.ask_unchanged_cpa_id.return_value = False
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename') as dialog:
            self.app.save_button.invoke()
        dialog.assert_not_called()
        self.assertFalse(os.path.exists(self.file_path))
        self.assertIn("INFO - Saving cancelled: change the CPAId on the General tab", self.last_log_line())

    def test_save_warns_after_cpa_id_changed_in_general_tab(self):
        """Test: save_warns_after_cpa_id_changed_in_general_tab"""
        self.app.load_cpa_data(CPA_FILE)
        entry = self.app.general_tab.entry_cpa_id
        entry.delete(0, tk.END)
        entry.insert(0, 'new_cpa_v02')
        self.app.general_tab.on_field_change('CPAId', type('Event', (), {'widget': entry}))
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename', return_value='') as dialog:
            self.app.save_button.invoke()
        self.assertEqual(dialog.call_args.kwargs['initialfile'], 'new_cpa_v02.xml')
        self.assertNotIn("has not been changed", self.log_text())
        self.ask_unchanged_cpa_id.assert_not_called()

    def test_save_stops_when_changes_are_not_confirmed(self):
        """Test: save_stops_when_changes_are_not_confirmed"""
        self.app.load_cpa_data(CPA_FILE)
        self.confirm_changes.return_value = False
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename') as dialog:
            self.app.save_button.invoke()
        dialog.assert_not_called()
        self.assertIn("INFO - Saving cancelled", self.last_log_line())

    def test_saved_cpa_can_be_loaded_again(self):
        """Test: saved_cpa_can_be_loaded_again"""
        self.app.load_cpa_data(CPA_FILE)
        cpa_id = self.app.general_tab.entry_cpa_id.get()
        self.app.save_cpa_data(self.file_path)
        self.app.load_cpa_data(self.file_path)
        self.assertIn("CPA data loaded successfully.", self.last_log_line())
        self.assertEqual(self.app.general_tab.entry_cpa_id.get(), cpa_id)

    def test_save_keeps_comment_before_root(self):
        """Test: save_keeps_comment_before_root"""
        source = os.path.join(self.directory.name, 'source.xml')
        with open(CPA_FILE, 'rb') as file:
            cpa = file.read()
        with open(source, 'wb') as file:
            file.write(b'<!-- agreed with partner -->\n' + cpa[cpa.index(b'<tns:CollaborationProtocolAgreement'):])
        self.app.load_cpa_data(source)
        self.app.save_cpa_data(self.file_path)
        with open(self.file_path, 'rb') as file:
            self.assertIn(b'<!-- agreed with partner -->', file.read())

    def test_save_without_cpa(self):
        """Test: save_without_cpa"""
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename') as dialog:
            self.app.save_button.invoke()
        dialog.assert_not_called()
        self.assertIn("ERROR - No CPA loaded, nothing to save", self.last_log_line())

    def test_save_cancelled(self):
        """Test: save_cancelled"""
        self.app.load_cpa_data(CPA_FILE)
        with patch.object(cpa_editor_app.filedialog, 'asksaveasfilename', return_value=''):
            self.app.save_button.invoke()
        self.assertFalse(os.path.exists(self.file_path))
        self.assertNotIn("CPA saved to", self.log_text())

    def test_failed_save_keeps_existing_file(self):
        """Test: failed_save_keeps_existing_file"""
        self.app.load_cpa_data(CPA_FILE)
        with open(self.file_path, 'w') as file:
            file.write('original')
        with patch.object(cpa_editor_app.Et, 'tostring', side_effect=ValueError("cannot serialise")):
            self.app.save_cpa_data(self.file_path)
        with open(self.file_path) as file:
            self.assertEqual(file.read(), 'original')
        self.assertIn("ERROR - Error while saving CPA: cannot serialise", self.last_log_line())

    def test_buttons_open_one_dialog(self):
        """Test: buttons_open_one_dialog"""
        self.app.load_cpa_data(CPA_FILE)
        # The dialogs are opened by the button command only, not also by a mouse binding
        self.assertEqual(self.app.save_button.bind("<Button-1>"), '')
        self.assertEqual(self.app.load_button.bind("<Button-1>"), '')
        with patch.object(cpa_editor_app.filedialog, 'askopenfilename', return_value='') as dialog:
            self.app.load_button.invoke()
        dialog.assert_called_once()


if __name__ == '__main__':
    unittest.main()
