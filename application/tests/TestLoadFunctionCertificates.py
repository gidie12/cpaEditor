import unittest
import tkinter as tk
from tkinter import ttk
from unittest.mock import Mock, patch
from lxml import etree

from application.app_classes.certificates_tab import Certificates
from application.helper_classes.directories import TESTS_DIR
import os

class TestLoadFunction(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.logger = Mock()
        self.master.namespace_uri = 'http://example.com'
        # load self.master.root from file
        self.master.root = etree.Element('root')
        self.cert_tab = Certificates(self.master, self.logger)  # Use the real logger instance
        self.cert_tab.tree_editor = ttk.Treeview(self.master)
        self.cert_tab.tree_editor.insert = Mock(wraps=self.cert_tab.tree_editor.insert)  # Record the inserted rows

    @patch('application.app_classes.certificates_tab.CPAParser')
    def test_load_populates_tree_with_certificates(self, MockCPAParser):
        mock_parser = MockCPAParser.return_value
        mock_parser.get_party_name_partner_a.return_value = 'PartnerA'
        mock_parser.get_party_name_partner_b.return_value = 'PartnerB'
        # <Certificate>
        #     <KeyInfo>
        #         <KeyValue>
        #             <RSAKeyValue>
        #                 <Modulus>...</Modulus>
        #                 <Exponent>...</Exponent>
        #             </RSAKeyValue>
        #         </KeyValue>
        #     </KeyInfo>
        # </Certificate>
        # generate a certificate element
        cert_element = etree.Element('Certificate', {'{http://example.com}certId': '123'})
        key_info = etree.SubElement(cert_element, '{http://www.w3.org/2000/09/xmldsig#}KeyInfo')

        mock_parser.get_certificate_party_a_elements.return_value = [cert_element]
        mock_parser.get_certificate_party_b_elements.return_value = [cert_element]

        self.cert_tab.load()

        self.assertTrue(self.cert_tab.tree_editor.insert.called)


    @patch('application.app_classes.certificates_tab.CPAParser')
    def test_load_logs_error_when_no_certificates_found(self, MockCPAParser):
        mock_parser = MockCPAParser.return_value
        mock_parser.get_certificate_party_a_elements.return_value = []
        mock_parser.get_certificate_party_b_elements.return_value = []

        self.cert_tab.load()

        self.assertTrue(self.logger.error.called)
        self.assertIn('No certificates found in the CPA', self.logger.error.call_args[0][0])

if __name__ == '__main__':
    unittest.main()