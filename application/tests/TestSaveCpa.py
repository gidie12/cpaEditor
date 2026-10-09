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
        self.assertIn("CPA data loaded successfully.", self.last_log_line())

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
