import os
import tempfile
import unittest
from unittest.mock import Mock, patch
import tkinter as tk

from cryptography.x509 import load_pem_x509_certificates

from application.app_classes.certificates_tab import Certificates, key_info_to_pem
from application.helper_classes.certificateExpiry import CPA_NAMESPACE, DS_NAMESPACE
from application.tests.TestCertificateExpiry import encode, generate_certificate, generate_cpa

import datetime

UTC = datetime.timezone.utc


class TestDownloadCertificate(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.root_cert, root_key = generate_certificate('Root', datetime.datetime(2040, 1, 1, tzinfo=UTC), is_ca=True)
        cls.leaf_cert, _ = generate_certificate('Leaf', datetime.datetime(2035, 1, 1, tzinfo=UTC),
                                                (cls.root_cert, root_key))

    def setUp(self):
        self.master = tk.Tk()
        self.master.host = 'mac'
        self.master.namespace_uri = CPA_NAMESPACE
        self.master.root = generate_cpa([{'A/cert 1': [encode(self.leaf_cert), encode(self.root_cert)]},
                                         {'B_cert': []}])
        self.logger = Mock()
        self.cert_tab = Certificates(self.master, self.logger)
        self.cert_tab.create()
        self.cert_tab.load()
        self.directory = tempfile.TemporaryDirectory()
        self.file_path = os.path.join(self.directory.name, 'download.pem')

    def tearDown(self):
        self.directory.cleanup()
        self.master.destroy()

    def select_key_info(self, partner_index):
        tree = self.cert_tab.tree_editor
        cert_id_item = tree.get_children(tree.get_children()[partner_index])[0]
        tree.selection_set(tree.get_children(cert_id_item)[0])

    def test_key_info_to_pem_returns_chain_in_order(self):
        """Test: key_info_to_pem_returns_chain_in_order"""
        key_info = self.master.root.find('.//{' + DS_NAMESPACE + '}KeyInfo')
        certs = load_pem_x509_certificates(key_info_to_pem(key_info).encode('utf-8'))
        self.assertEqual(certs, [self.leaf_cert, self.root_cert])

    def test_key_info_to_pem_without_certificates(self):
        """Test: key_info_to_pem_without_certificates"""
        self.assertEqual(key_info_to_pem(None), '')
        self.assertEqual(key_info_to_pem(self.master.root.findall('.//{' + DS_NAMESPACE + '}KeyInfo')[1]), '')

    def test_save_certificate_writes_chain(self):
        """Test: save_certificate_writes_chain"""
        self.select_key_info(0)
        with patch('application.app_classes.certificates_tab.filedialog.asksaveasfilename',
                   return_value=self.file_path) as dialog:
            self.cert_tab.save_certificate()
        self.assertEqual(dialog.call_args.kwargs['initialfile'], 'A_cert_1.pem')
        with open(self.file_path, 'rb') as cert_file:
            self.assertEqual(load_pem_x509_certificates(cert_file.read()), [self.leaf_cert, self.root_cert])
        self.assertIn("2 certificate(s) of certId A/cert 1", self.logger.info.call_args[0][0])

    def test_save_certificate_no_file_selected(self):
        """Test: save_certificate_no_file_selected"""
        self.select_key_info(0)
        with patch('application.app_classes.certificates_tab.filedialog.asksaveasfilename', return_value=''):
            self.cert_tab.save_certificate()
        self.assertFalse(os.path.exists(self.file_path))
        self.logger.error.assert_not_called()

    def test_save_certificate_without_certificate(self):
        """Test: save_certificate_without_certificate"""
        self.select_key_info(1)
        with patch('application.app_classes.certificates_tab.filedialog.asksaveasfilename') as dialog:
            self.cert_tab.save_certificate()
        dialog.assert_not_called()
        self.logger.error.assert_called_with("No certificate found in this KeyInfo")


if __name__ == '__main__':
    unittest.main()
