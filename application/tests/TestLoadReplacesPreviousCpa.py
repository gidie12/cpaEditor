import os
import unittest
from unittest.mock import patch
import tkinter as tk

from application import CpaEditorApp as cpa_editor_app
from application.helper_classes.directories import TESTS_DIR

CPAS_DIR = os.path.join(TESTS_DIR, 'resources', 'cpas')
CPA_WITH_CERTIFICATES = os.path.join(CPAS_DIR, 'full_test_cpa.xml')
CPA_WITHOUT_CERTIFICATES = os.path.join(CPAS_DIR, 'example.xml')


class TestLoadReplacesPreviousCpa(unittest.TestCase):

    def setUp(self):
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            self.app = cpa_editor_app.CpaEditorApp()
        self.app.load_cpa_data(CPA_WITH_CERTIFICATES)

    def tearDown(self):
        # The app logs into its own Text widget, which no longer exists after destroy
        for handler in list(cpa_editor_app.logger.handlers):
            cpa_editor_app.logger.removeHandler(handler)
        self.app.destroy()

    def test_certificates_of_previous_cpa_are_removed(self):
        """Test: certificates_of_previous_cpa_are_removed"""
        self.assertEqual(len(self.app.certificates_tab.tree_editor.get_children()), 2)
        self.app.load_cpa_data(CPA_WITHOUT_CERTIFICATES)
        self.assertEqual(self.app.certificates_tab.tree_editor.get_children(), ())

    def test_xml_editor_shows_only_loaded_cpa(self):
        """Test: xml_editor_shows_only_loaded_cpa"""
        self.app.load_cpa_data(CPA_WITHOUT_CERTIFICATES)
        self.app.load_cpa_data(CPA_WITHOUT_CERTIFICATES)
        tree = self.app.xml_editor_tab.tree_editor
        self.assertEqual(len(tree.get_children()), 1)
        self.assertIs(self.app.xml_editor_tab.xml_element_mapping[tree.get_children()[0]], self.app.root)

    def test_validation_result_of_previous_cpa_is_removed(self):
        """Test: validation_result_of_previous_cpa_is_removed"""
        self.app.validator_tab.validate_schema()
        self.app.load_cpa_data(CPA_WITHOUT_CERTIFICATES)
        self.assertEqual(self.app.validator_tab.validation_errors.get('1.0', tk.END).strip(), 'Geen validatie uitgevoerd')

    def test_comment_selection_of_previous_cpa_is_removed(self):
        """Test: comment_selection_of_previous_cpa_is_removed"""
        self.app.comment_tab.listbox.selection_set(0)
        self.app.comment_tab.on_select(None)
        self.assertNotEqual(self.app.comment_tab.comment_entry.get(), '')
        self.app.load_cpa_data(CPA_WITHOUT_CERTIFICATES)
        self.assertIsNone(self.app.comment_tab.selected_index)
        self.assertEqual(self.app.comment_tab.comment_entry.get(), '')

    def test_transport_selection_of_previous_cpa_is_removed(self):
        """Test: transport_selection_of_previous_cpa_is_removed"""
        transport_tab = self.app.transport_tab
        transport_tab.selected_item = 'I001'
        transport_tab.field_label.config(text="Editing: uri")
        self.app.load_cpa_data(CPA_WITHOUT_CERTIFICATES)
        self.assertIsNone(transport_tab.selected_item)
        self.assertEqual(transport_tab.field_label.cget('text'), "Select a field to edit")


if __name__ == '__main__':
    unittest.main()
