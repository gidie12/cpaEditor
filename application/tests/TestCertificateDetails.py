import datetime
import os
import tempfile
import unittest
from unittest.mock import Mock, patch
import tkinter as tk

from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography import x509
from cryptography.x509.oid import NameOID

from application.app_classes.certificates_tab import Certificates
from application.helper_classes.certificateExpiry import CPA_NAMESPACE, DS_NAMESPACE, get_key_info_certificates
from application.tests.TestCertificateExpiry import encode, generate_certificate, generate_cpa

UTC = datetime.timezone.utc


class TestCertificateDetails(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.root_cert, root_key = generate_certificate('Root', datetime.datetime(2040, 1, 1, tzinfo=UTC), is_ca=True)
        cls.leaf_cert, _ = generate_certificate('Leaf', datetime.datetime(2035, 6, 15, 12, 30, 5, tzinfo=UTC),
                                                (cls.root_cert, root_key))
        cls.old_cert, _ = generate_certificate('Old', datetime.datetime(2021, 1, 1, tzinfo=UTC))

    def setUp(self):
        self.master = tk.Tk()
        self.master.host = 'mac'
        self.master.namespace_uri = CPA_NAMESPACE
        self.master.root = generate_cpa([{'A_cert': [encode(self.leaf_cert), encode(self.root_cert)]},
                                         {'B_old': [encode(self.old_cert)], 'B_empty': []}])
        self.logger = Mock()
        self.cert_tab = Certificates(self.master, self.logger)
        self.cert_tab.create()
        self.cert_tab.load()
        self.tree = self.cert_tab.tree_editor

    def tearDown(self):
        self.master.destroy()

    def cert_id_item(self, cert_id):
        for partner in self.tree.get_children():
            for item in self.tree.get_children(partner):
                if self.tree.item(item, 'values')[0] == cert_id:
                    return item
        raise KeyError(cert_id)

    def rows(self, item):
        return [(self.tree.item(child, 'text'), self.tree.item(child, 'values')[0])
                for child in self.tree.get_children(item)]

    def test_get_key_info_certificates_reads_details(self):
        """Test: get_key_info_certificates_reads_details"""
        key_info = self.master.root.find('.//{' + DS_NAMESPACE + '}KeyInfo')
        leaf, root = get_key_info_certificates(key_info)
        self.assertEqual((leaf['type'], leaf['common_name'], leaf['subject'], leaf['issuer']),
                         ('leaf', 'Leaf', 'CN=Leaf', 'CN=Root'))
        self.assertEqual(leaf['serial_number'], format(self.leaf_cert.serial_number, 'X'))
        self.assertEqual(leaf['not_before'], datetime.datetime(2020, 1, 1, tzinfo=UTC))
        self.assertEqual(leaf['not_after'], datetime.datetime(2035, 6, 15, 12, 30, 5, tzinfo=UTC))
        self.assertEqual((root['type'], root['common_name']), ('root', 'Root'))

    def test_get_key_info_certificates_marks_expired(self):
        """Test: get_key_info_certificates_marks_expired"""
        key_info = self.master.root.find('.//{' + DS_NAMESPACE + '}KeyInfo')
        expired = [certificate['expired'] for certificate in
                   get_key_info_certificates(key_info, now=datetime.datetime(2036, 1, 1, tzinfo=UTC))]
        self.assertEqual(expired, [True, False])

    def test_get_key_info_certificates_without_key_info(self):
        """Test: get_key_info_certificates_without_key_info"""
        self.assertEqual(get_key_info_certificates(None), [])

    def test_load_shows_details_of_certificates(self):
        """Test: load_shows_details_of_certificates"""
        item = self.cert_id_item('A_cert')
        rows = self.rows(item)
        self.assertEqual(rows[0][0], "KeyInfo")
        self.assertEqual(rows[1:], [("Certificate (leaf)", "Leaf - expires 2035-06-15T12:30:05Z"),
                                    ("Certificate (root)", "Root - expires 2040-01-01T00:00:00Z")])
        self.assertEqual(self.rows(self.tree.get_children(item)[1]),
                         [("Common name", "Leaf"),
                          ("Subject", "CN=Leaf"),
                          ("Issuer", "CN=Root"),
                          ("Serial number", format(self.leaf_cert.serial_number, 'X')),
                          ("Valid from", "2020-01-01T00:00:00Z"),
                          ("Valid until", "2035-06-15T12:30:05Z")])
        self.assertEqual([text for text, _ in self.rows(self.cert_id_item('B_empty'))], ["KeyInfo"])

    def test_load_marks_expired_certificate(self):
        """Test: load_marks_expired_certificate"""
        item = self.cert_id_item('B_old')
        certificate_item = self.tree.get_children(item)[1]
        self.assertEqual(self.tree.item(certificate_item, 'values')[0], "Old - expires 2021-01-01T00:00:00Z (EXPIRED)")

    @patch('application.app_classes.certificates_tab.filedialog.askopenfilename')
    def test_open_certificate_replaces_certificate_details(self, mock_askopenfilename):
        """Test: open_certificate_replaces_certificate_details"""
        key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
        name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, 'New')])
        new_cert = (x509.CertificateBuilder().subject_name(name).issuer_name(name).public_key(key.public_key())
                    .serial_number(x509.random_serial_number())
                    .not_valid_before(datetime.datetime(2020, 1, 1, tzinfo=UTC))
                    .not_valid_after(datetime.datetime(2045, 1, 1, tzinfo=UTC))
                    .sign(key, hashes.SHA256()))
        with tempfile.TemporaryDirectory() as directory:
            file_path = os.path.join(directory, 'new.pem')
            with open(file_path, 'wb') as cert_file:
                cert_file.write(new_cert.public_bytes(serialization.Encoding.PEM))
            mock_askopenfilename.return_value = file_path

            item = self.cert_id_item('B_old')
            self.tree.selection_set(self.tree.get_children(item)[0])
            self.cert_tab.open_certificate()

        self.assertEqual(self.rows(item)[1:], [("Certificate (leaf)", "New - expires 2045-01-01T00:00:00Z")])


if __name__ == '__main__':
    unittest.main()
