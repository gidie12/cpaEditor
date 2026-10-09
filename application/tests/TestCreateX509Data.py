import logging
import unittest
from unittest.mock import Mock
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization, hashes
from cryptography.hazmat.backends import default_backend
from cryptography import x509
from cryptography.x509.oid import NameOID
from lxml import etree as lxml_etree
import base64
import datetime
import tkinter as tk
from application.app_classes.certificates_tab import Certificates, create_x509_data
logger = logging.getLogger(__name__)

class TestCreateX509Data(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.addCleanup(self.master.destroy)
        # generate mock logger with a mock info method
        self.logger = logger
        self.cert_tab = Certificates(self.master, self.logger)

    def generate_test_certificate(self):
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048,
            backend=default_backend()
        )

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

    def test_create_x509_data_creates_correct_elements(self):
        cert = self.generate_test_certificate()
        key_info = lxml_etree.Element('{http://www.w3.org/2000/09/xmldsig#}KeyInfo')
        x509_data = create_x509_data(cert, key_info)

        self.assertIsNotNone(x509_data.find('{http://www.w3.org/2000/09/xmldsig#}X509SubjectName'))
        self.assertIsNotNone(x509_data.find('{http://www.w3.org/2000/09/xmldsig#}X509Certificate'))
        self.assertIsNotNone(x509_data.find('{http://www.w3.org/2000/09/xmldsig#}X509IssuerSerial'))

    def test_create_x509_data_adds_elements_to_key_info(self):
        cert = self.generate_test_certificate()
        key_info = lxml_etree.Element('{http://www.w3.org/2000/09/xmldsig#}KeyInfo')
        create_x509_data(cert, key_info)

        self.assertIsNotNone(key_info.find('{http://www.w3.org/2000/09/xmldsig#}X509Data'))

if __name__ == '__main__':
    unittest.main()