import unittest
from unittest.mock import Mock
from zoneinfo import ZoneInfo
import tkinter as tk

from lxml import etree as lxml_etree

from application.app_classes.general_tab import General
from application.helper_classes.cpaDates import describe_cpa_date

AMSTERDAM = ZoneInfo('Europe/Amsterdam')
CPA_NAMESPACE = 'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd'


class TestDescribeCpaDate(unittest.TestCase):

    def test_describe_cpa_date_zulu_and_local(self):
        """Test: describe_cpa_date_zulu_and_local"""
        self.assertEqual(describe_cpa_date('2026-10-09T10:44:32Z', AMSTERDAM),
                         "Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 CEST (UTC+02:00)")
        self.assertEqual(describe_cpa_date('2026-12-31T23:30:00Z', AMSTERDAM),
                         "Zulu: 2026-12-31T23:30:00Z    Local: 2027-01-01 00:30:00 CET (UTC+01:00)")

    def test_describe_cpa_date_with_offset(self):
        """Test: describe_cpa_date_with_offset"""
        self.assertEqual(describe_cpa_date('2026-10-09T12:44:32+02:00', AMSTERDAM),
                         "Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 CEST (UTC+02:00)")

    def test_describe_cpa_date_without_timezone(self):
        """Test: describe_cpa_date_without_timezone"""
        self.assertEqual(describe_cpa_date('2026-10-09T10:44:32', AMSTERDAM),
                         "Date has no timezone: add Z for Zulu time (UTC)")

    def test_describe_cpa_date_invalid(self):
        """Test: describe_cpa_date_invalid"""
        self.assertIn("Not a valid date", describe_cpa_date('09-10-2026', AMSTERDAM))

    def test_describe_cpa_date_empty(self):
        """Test: describe_cpa_date_empty"""
        self.assertEqual(describe_cpa_date('', AMSTERDAM), "")
        self.assertEqual(describe_cpa_date(None, AMSTERDAM), "")


class TestDateLabels(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.master.root = lxml_etree.Element('{' + CPA_NAMESPACE + '}CollaborationProtocolAgreement')
        lxml_etree.SubElement(self.master.root, '{' + CPA_NAMESPACE + '}Start').text = '2020-01-01T00:00:00Z'
        for _ in range(2):
            lxml_etree.SubElement(self.master.root, '{' + CPA_NAMESPACE + '}PartyInfo')
        self.general_tab = General(self.master, Mock())
        self.general_tab.create()

    def tearDown(self):
        self.master.destroy()

    def test_update_date_labels_shows_zulu_and_local(self):
        """Test: update_date_labels_shows_zulu_and_local"""
        self.general_tab.entry_start_date.insert(0, '2026-10-09T10:44:32Z')
        self.general_tab.entry_end_date.insert(0, 'tomorrow')
        self.general_tab.update_date_labels()
        self.assertTrue(self.general_tab.label_start_date_times.cget('text').startswith(
            "Zulu: 2026-10-09T10:44:32Z    Local: "))
        self.assertIn("Not a valid date", self.general_tab.label_end_date_times.cget('text'))

    def test_set_start_date_to_now_updates_label(self):
        """Test: set_start_date_to_now_updates_label"""
        self.general_tab.set_start_date_to_now()
        self.assertIn(self.general_tab.entry_start_date.get(), self.general_tab.label_start_date_times.cget('text'))


if __name__ == '__main__':
    unittest.main()
