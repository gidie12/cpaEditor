import os
import unittest
from unittest.mock import patch
import tkinter as tk

from application import CpaEditorApp as cpa_editor_app
from application.helper_classes.directories import TESTS_DIR
from application.helper_classes.partnerColors import PARTNER_COLORS, PARTNER_TEXT_COLOR, partner_color

CPA_FILE = os.path.join(TESTS_DIR, 'resources', 'cpas', 'test_cpa.xml')


class TestPartnerColors(unittest.TestCase):

    def setUp(self):
        with patch.object(tk.Tk, 'mainloop', lambda self: None):
            self.app = cpa_editor_app.CpaEditorApp()
        self.app.load_cpa_data(CPA_FILE)

    def tearDown(self):
        # The app logs into its own Text widget, which no longer exists after destroy
        for handler in list(cpa_editor_app.logger.handlers):
            cpa_editor_app.logger.removeHandler(handler)
        self.app.destroy()

    def partner_row_colors(self, tree):
        """Returns per partner row (text starting with 'Partner ') its (background, foreground)."""
        colors = []
        for item in tree.get_children(''):
            if tree.item(item, 'text').startswith('Partner '):
                tag = tree.item(item, 'tags')[0]
                colors.append((str(tree.tag_configure(tag, 'background')), str(tree.tag_configure(tag, 'foreground'))))
        return colors

    def test_partner_color(self):
        """Test: partner_color"""
        self.assertEqual(partner_color(0), PARTNER_COLORS[0])
        self.assertEqual(partner_color(1), PARTNER_COLORS[1])
        self.assertEqual(partner_color(2), PARTNER_COLORS[0])

    def test_tabs_use_the_same_partner_colors(self):
        """Test: tabs_use_the_same_partner_colors"""
        expected = [(PARTNER_COLORS[0], PARTNER_TEXT_COLOR), (PARTNER_COLORS[1], PARTNER_TEXT_COLOR)]
        self.app.collaboration_role_tab.reload()
        self.assertEqual(self.partner_row_colors(self.app.collaboration_role_tab.tree_editor), expected)
        # Transport shows a partner row per transport, so a partner can occur more than once
        self.assertEqual(sorted(set(self.partner_row_colors(self.app.transport_tab.tree_editor))), sorted(expected))
        self.assertEqual(self.partner_row_colors(self.app.certificates_tab.tree_editor), expected)

    def test_general_tab_partner_labels(self):
        """Test: general_tab_partner_labels"""
        general = self.app.general_tab
        for label, index in ((general.label_partner_name_partner_a, 0), (general.label_party_id_partner_a, 0),
                             (general.label_party_id_type_partner_a, 0), (general.label_partner_name_partner_b, 1),
                             (general.label_party_id_partner_b, 1), (general.label_party_id_type_partner_b, 1)):
            self.assertEqual(label.cget('bg'), PARTNER_COLORS[index])
            self.assertEqual(label.cget('fg'), PARTNER_TEXT_COLOR)


if __name__ == '__main__':
    unittest.main()
