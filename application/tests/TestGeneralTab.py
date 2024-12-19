import unittest
from unittest.mock import patch, Mock, MagicMock
import tkinter as tk
from application.app_classes.general_tab import General
import logging
logger = logging.getLogger(__name__)

class TestGeneralTab(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.logger = Mock()
        self.logger.debug = Mock()
        self.master.root = MagicMock()
        self.master.root.findall.return_value = [Mock(), Mock()]  # Mock the findall method to return a list of mock elements
        self.general_tab = General(self.master, self.logger)
        self.general_tab.create()

    @patch('application.app_classes.general_tab.CPAParser')
    def test_creates_all_fields_correctly(self, MockCPAParser):
        """Test: creates_all_fields_correctly"""
        mock_parser = MockCPAParser.return_value
        mock_parser.get_cpa_id.return_value = 'CPA123'
        mock_parser.get_cpa_status.return_value = 'proposed'
        mock_parser.get_party_id_partner_a.return_value = 'PartnerA_ID'
        mock_parser.get_party_id_partner_b.return_value = 'PartnerB_ID'
        mock_parser.get_party_name_partner_a.return_value = 'PartnerA'
        mock_parser.get_party_name_partner_b.return_value = 'PartnerB'
        mock_parser.get_cpa_start_date.return_value = '2023-01-01'
        mock_parser.get_cpa_end_date.return_value = '2023-12-31'
        mock_parser.get_party_ids_type_partner_a.return_value = 'TypeA'
        mock_parser.get_party_ids_type_partner_b.return_value = 'TypeB'

        self.general_tab.create()
        self.general_tab.load()

        self.assertEqual(self.general_tab.entry_cpa_id.get(), 'CPA123')
        self.assertEqual(self.general_tab.cpa_status_value.get(), 'proposed')
        self.assertEqual(self.general_tab.entry_party_id_partner_a.get(), 'PartnerA_ID')
        self.assertEqual(self.general_tab.entry_party_id_partner_b.get(), 'PartnerB_ID')
        self.assertEqual(self.general_tab.entry_partner_name_partner_a.get(), 'PartnerA')
        self.assertEqual(self.general_tab.entry_partner_name_partner_b.get(), 'PartnerB')
        self.assertEqual(self.general_tab.entry_start_date.get(), '2023-01-01')
        self.assertEqual(self.general_tab.entry_end_date.get(), '2023-12-31')
        self.assertEqual(self.general_tab.entry_party_id_type_partner_a.get(), 'TypeA')
        self.assertEqual(self.general_tab.entry_party_id_type_partner_b.get(), 'TypeB')

    @patch('application.app_classes.general_tab.CPAParser')
    def test_handles_empty_fields_gracefully(self, MockCPAParser):
        """Test: handles_empty_fields_gracefully"""
        mock_parser = MockCPAParser.return_value
        mock_parser.get_cpa_id.return_value = ''
        mock_parser.get_cpa_status.return_value = ''
        mock_parser.get_party_id_partner_a.return_value = ''
        mock_parser.get_party_id_partner_b.return_value = ''
        mock_parser.get_party_name_partner_a.return_value = ''
        mock_parser.get_party_name_partner_b.return_value = ''
        mock_parser.get_cpa_start_date.return_value = ''
        mock_parser.get_cpa_end_date.return_value = ''
        mock_parser.get_party_ids_type_partner_a.return_value = ''
        mock_parser.get_party_ids_type_partner_b.return_value = ''

        self.general_tab.create()

        self.assertEqual(self.general_tab.entry_cpa_id.get(), '')
        self.assertEqual(self.general_tab.cpa_status_value.get(), '')
        self.assertEqual(self.general_tab.entry_party_id_partner_a.get(), '')
        self.assertEqual(self.general_tab.entry_party_id_partner_b.get(), '')
        self.assertEqual(self.general_tab.entry_partner_name_partner_a.get(), '')
        self.assertEqual(self.general_tab.entry_partner_name_partner_b.get(), '')
        self.assertEqual(self.general_tab.entry_start_date.get(), '')
        self.assertEqual(self.general_tab.entry_end_date.get(), '')
        self.assertEqual(self.general_tab.entry_party_id_type_partner_a.get(), '')
        self.assertEqual(self.general_tab.entry_party_id_type_partner_b.get(), '')

    def test_bind_fields(self):
        """Test: bind_fields"""
        self.general_tab.bind_fields()
        self.assertIn('CPAId', self.general_tab.all_entry_fields)
        self.assertIn('cpaStatus', self.general_tab.all_dropdown_fields)

    def test_on_dropdown_change(self):
        """Test: on_dropdown_change"""
        self.general_tab.master.xml_tree = MagicMock()
        self.general_tab.on_dropdown_change('cpaStatus', self.general_tab.cpa_status_value, None)
        # check if logger was called
        self.logger.debug.assert_called_with("Dropdown changed cpaStatus New selected value: ")

        # self.general_tab.("Dropdown changed cpaStatus New selected value: ")

    def test_on_field_change(self):
        """Test: on_field_change"""
        self.general_tab.master.xml_tree = MagicMock()
        event = Mock()
        event.widget.get.return_value = 'new_value'
        self.general_tab.on_field_change('CPAId', event)
        self.logger.debug.assert_called_with("Field 'CPAId' changed: New value: new_value")

    def test_bind_event_to_entry(self):
        """Test: bind_event_to_entry"""
        entry = Mock()
        self.general_tab.bind_event_to_entry(entry, lambda x: x)
        entry.bind.assert_called_with("<FocusOut>", unittest.mock.ANY)

    def test_bind_event_to_widget(self):
        """Test: bind_event_to_widget"""
        widget = Mock()
        self.general_tab.bind_event_to_widget(widget, "<Configure>", lambda x: x)
        widget.bind.assert_called_with("<Configure>", unittest.mock.ANY)

    @patch('application.app_classes.general_tab.CPAParser')
    def test_load(self, MockCPAParser):
        """Test: load"""
        mock_parser = MockCPAParser.return_value
        mock_parser.get_cpa_id.return_value = 'CPA123'
        mock_parser.get_cpa_status.return_value = 'proposed'
        mock_parser.get_party_id_partner_a.return_value = 'PartnerA_ID'
        mock_parser.get_party_id_partner_b.return_value = 'PartnerB_ID'
        mock_parser.get_party_name_partner_a.return_value = 'PartnerA'
        mock_parser.get_party_name_partner_b.return_value = 'PartnerB'
        mock_parser.get_cpa_start_date.return_value = '2023-01-01'
        mock_parser.get_cpa_end_date.return_value = '2023-12-31'
        mock_parser.get_party_ids_type_partner_a.return_value = 'TypeA'
        mock_parser.get_party_ids_type_partner_b.return_value = 'TypeB'

        self.general_tab.load()

        self.assertEqual(self.general_tab.entry_cpa_id.get(), 'CPA123')
        self.assertEqual(self.general_tab.cpa_status_value.get(), 'proposed')
        self.assertEqual(self.general_tab.entry_party_id_partner_a.get(), 'PartnerA_ID')
        self.assertEqual(self.general_tab.entry_party_id_partner_b.get(), 'PartnerB_ID')
        self.assertEqual(self.general_tab.entry_partner_name_partner_a.get(), 'PartnerA')
        self.assertEqual(self.general_tab.entry_partner_name_partner_b.get(), 'PartnerB')
        self.assertEqual(self.general_tab.entry_start_date.get(), '2023-01-01')
        self.assertEqual(self.general_tab.entry_end_date.get(), '2023-12-31')
        self.assertEqual(self.general_tab.entry_party_id_type_partner_a.get(), 'TypeA')
        self.assertEqual(self.general_tab.entry_party_id_type_partner_b.get(), 'TypeB')

if __name__ == '__main__':
    unittest.main()