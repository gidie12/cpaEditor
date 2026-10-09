import unittest
from unittest.mock import Mock
import xml.etree.ElementTree as Et
import tkinter as tk
from application.app_classes.certificates_tab import Certificates

class TestGeneratePartnerKeyinfoCertidList(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.addCleanup(self.master.destroy)
        self.logger = Mock()
        self.cert_tab = Certificates(self.master, self.logger)

    def test_generate_partner_keyinfo_certid_list_with_valid_data(self):
        cert_element = Et.Element('Certificate', {'{http://example.com}certId': '123'})
        key_info = Et.SubElement(cert_element, '{http://www.w3.org/2000/09/xmldsig#}KeyInfo')
        Et.SubElement(key_info, 'KeyValue')
        result = self.cert_tab.generate_partner_keyinfo_certid_list([cert_element], '{http://example.com}certId')
        self.assertIn('123', result)
        self.assertEqual(len(result['123']), 3)

    def test_generate_partner_keyinfo_certid_list_with_missing_cert_id(self):
        cert_element = Et.Element('Certificate')
        result = self.cert_tab.generate_partner_keyinfo_certid_list([cert_element], '{http://example.com}certId')
        self.assertEqual(result, {})

    def test_generate_partner_keyinfo_certid_list_with_no_key_info(self):
        cert_element = Et.Element('Certificate', {'{http://example.com}certId': '123'})
        result = self.cert_tab.generate_partner_keyinfo_certid_list([cert_element], '{http://example.com}certId')
        self.assertIn('123', result)
        self.assertEqual(result['123'][0], '')

if __name__ == '__main__':
    unittest.main()