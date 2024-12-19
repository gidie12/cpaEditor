import unittest
from unittest.mock import Mock, patch

from cryptography.hazmat.backends import default_backend
from lxml import etree as lxml_etree
from application.app_classes.certificates_tab import Certificates, create_rsa_key_value_elements, replace_xml_object
import xml.etree.ElementTree as Et
import tkinter as tk  # Import tkinter as tk
from tkinter import ttk
import platform
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization, hashes
from cryptography.hazmat.backends import default_backend
from cryptography import x509
from cryptography.x509.oid import NameOID
import datetime


class TestCertificates(unittest.TestCase):

    @patch('application.app_classes.certificates_tab.filedialog.askopenfilename')
    @patch('application.app_classes.certificates_tab.open')
    @patch('application.app_classes.certificates_tab.load_pem_x509_certificate')
    def test_open_certificate_valid_file(self, mock_load_cert, mock_open, mock_askopenfilename):
        mock_askopenfilename.return_value = 'dummy_path'
        mock_open.return_value.__enter__.return_value.read.return_value = b'-----BEGIN CERTIFICATE-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA7\n-----END CERTIFICATE-----'
        mock_cert = Mock()
        mock_cert.public_key().public_numbers.return_value.n = 123456789
        mock_cert.public_key().public_numbers.return_value.e = 65537
        mock_cert.subject.rfc4514_string.return_value = 'CN=Test'
        mock_cert.issuer.rfc4514_string.return_value = 'CN=Issuer'
        mock_cert.serial_number = 123456789
        mock_cert.public_bytes.return_value = b'cert_bytes'
        mock_load_cert.return_value = mock_cert

        master = tk.Tk()  # Use tkinter.Tk as the master object
        master.namespace_uri = 'http://example.com'
        logger = Mock()
        cert_tab = Certificates(master, logger)
        cert_tab.tree_editor = Mock()
        cert_tab.tree_editor.selection.return_value = ['item']
        cert_tab.xml_element_mapping = {'item': [None, None]}

        cert_tab.open_certificate()
        # show log output

        self.assertTrue(logger.info.called)
        self.assertIn('Certificate uploaded successfully', logger.info.call_args[0][0])


    @patch('application.app_classes.certificates_tab.filedialog.askopenfilename')
    def test_open_certificate_no_file_selected(self, mock_askopenfilename):
        mock_askopenfilename.return_value = ''

        master = tk.Tk()  # Use tkinter.Tk as the master object
        master.namespace_uri = 'http://example.com'
        logger = Mock()
        cert_tab = Certificates(master, logger)
        cert_tab.tree_editor = Mock()
        cert_tab.tree_editor.selection.return_value = ['item']
        cert_tab.xml_element_mapping = {'item': [None, None]}

        cert_tab.open_certificate()

        self.assertFalse(logger.info.called)
        self.assertFalse(logger.error.called)

    @patch('application.app_classes.certificates_tab.filedialog.askopenfilename')
    @patch('application.app_classes.certificates_tab.open')
    def test_open_certificate_invalid_certificate(self, mock_open, mock_askopenfilename):
        mock_askopenfilename.return_value = 'dummy_path'
        mock_open.return_value.__enter__.return_value.read.return_value = 'invalid_certificate_data'

        master = tk.Tk()  # Use tkinter.Tk as the master object
        master.namespace_uri = 'http://example.com'
        logger = Mock()
        cert_tab = Certificates(master, logger)
        cert_tab.tree_editor = Mock()
        cert_tab.tree_editor.selection.return_value = ['item']
        cert_tab.xml_element_mapping = {'item': [None, None]}

        cert_tab.open_certificate()

        self.assertTrue(logger.error.called)
        self.assertIn('Error while uploading certificate', logger.error.call_args[0][0])

    @patch('application.app_classes.certificates_tab.filedialog.askopenfilename')
    @patch('application.app_classes.certificates_tab.open')
    @patch('application.app_classes.certificates_tab.load_pem_x509_certificate')
    def test_open_certificate_valid_file_updates_tree(self, mock_load_cert, mock_open, mock_askopenfilename):
        mock_askopenfilename.return_value = 'dummy_path'
        mock_open.return_value.__enter__.return_value.read.return_value = b'-----BEGIN CERTIFICATE-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA7\n-----END CERTIFICATE-----'
        mock_cert = Mock()
        mock_cert.public_key().public_numbers.return_value.n = 123456789
        mock_cert.public_key().public_numbers.return_value.e = 65537
        mock_cert.subject.rfc4514_string.return_value = 'CN=Test'
        mock_cert.issuer.rfc4514_string.return_value = 'CN=Issuer'
        mock_cert.serial_number = 123456789
        mock_cert.public_bytes.return_value = b'cert_bytes'
        mock_load_cert.return_value = mock_cert

        master = tk.Tk()
        master.namespace_uri = 'http://example.com'
        logger = Mock()
        cert_tab = Certificates(master, logger)
        cert_tab.tree_editor = Mock()
        cert_tab.tree_editor.selection.return_value = ['item']
        cert_tab.xml_element_mapping = {'item': [None, None]}

        cert_tab.open_certificate()

        self.assertTrue(cert_tab.tree_editor.item.called)
        self.assertIn('Certificate uploaded successfully', logger.info.call_args[0][0])



class TestCreateTreeWidget(unittest.TestCase):
    def setUp(self):
        self.master = tk.Tk()
        self.master.host = 'Darwin'
        self.logger = Mock()
        self.cert_tab = Certificates(self.master, self.logger)

    def test_create_treeview_widget_created(self):
        """
        Test that the Treeview widget is created.
        """
        self.cert_tab.create()
        self.assertIsNotNone(self.cert_tab.tree_editor)

    def test_create_treeview_widget_configured(self):
        """
        Test that the Treeview widget is configured with the correct columns and styles.
        """
        self.cert_tab.create()
        self.assertEqual(self.cert_tab.tree_editor["columns"], ("Value",))
        self.assertEqual(self.cert_tab.tree_editor.heading("#0")["text"], "Partner/Certificate ID")
        self.assertEqual(self.cert_tab.tree_editor.heading("Value")["text"], "Value/KeyInfo")

    def test_create_treeview_widget_bound_to_menu(self):
        """
        Test that the Treeview widget is bound to the right-click menu.
        """
        self.master.host = 'Darwin'
        self.cert_tab.create()
        self.assertIsNotNone(self.cert_tab.tree_editor.bind("<Button-2>"))

        self.master.host = 'Windows'
        self.cert_tab.create()
        self.assertIsNotNone(self.cert_tab.tree_editor.bind("<Button-3>"))


class TestCreateRSAKeyValueElements(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.logger = Mock()
        self.cert_tab = Certificates(self.master, self.logger)

    def generate_test_certificate(self):
        # Generate a private key for the certificate
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048,
            backend=default_backend()
        )

        # Generate a self-signed certificate
        subject = issuer = x509.Name([
            x509.NameAttribute(NameOID.COUNTRY_NAME, u"US"),
            x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, u"California"),
            x509.NameAttribute(NameOID.LOCALITY_NAME, u"San Francisco"),
            x509.NameAttribute(NameOID.ORGANIZATION_NAME, u"My Company"),
            x509.NameAttribute(NameOID.COMMON_NAME, u"mycompany.com"),
        ])
        cert = x509.CertificateBuilder().subject_name(
            subject
        ).issuer_name(
            issuer
        ).public_key(
            private_key.public_key()
        ).serial_number(
            x509.random_serial_number()
        ).not_valid_before(
            datetime.datetime.utcnow()
        ).not_valid_after(
            datetime.datetime.utcnow() + datetime.timedelta(days=10)
        ).sign(private_key, hashes.SHA256(), default_backend())

        return cert

    def test_create_rsa_key_value_elements(self):
        """
        Test that the RSA key value elements are created correctly from a certificate.
        """
        cert = self.generate_test_certificate()
        rsa_key_value_elements = create_rsa_key_value_elements(cert)

        self.assertEqual(len(rsa_key_value_elements), 2)
        self.assertEqual(rsa_key_value_elements[0][0], '{http://www.w3.org/2000/09/xmldsig#}Modulus')
        self.assertEqual(rsa_key_value_elements[1][0], '{http://www.w3.org/2000/09/xmldsig#}Exponent')
        self.assertTrue(isinstance(rsa_key_value_elements[0][1], str))
        self.assertTrue(isinstance(rsa_key_value_elements[1][1], str))

if __name__ == '__main__':
    unittest.main()
