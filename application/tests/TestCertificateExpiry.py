import base64
import datetime
import unittest
from unittest.mock import Mock
import tkinter as tk

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.x509.oid import NameOID
from lxml import etree as lxml_etree

from application.app_classes.general_tab import General
from application.helper_classes.certificateExpiry import (CPA_NAMESPACE, DS_NAMESPACE, certificate_common_name,
                                                          certificate_type, get_certificate_expiries,
                                                          get_first_expiring_certificate)

UTC = datetime.timezone.utc


def generate_certificate(common_name, not_after, issuer=None, is_ca=False):
    """Returns (certificate, key); self-signed when no (issuer certificate, issuer key) is given."""
    key = ec.generate_private_key(ec.SECP256R1())
    subject = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, common_name)])
    issuer_name, issuer_key = (issuer[0].subject, issuer[1]) if issuer else (subject, key)
    cert = (x509.CertificateBuilder()
            .subject_name(subject)
            .issuer_name(issuer_name)
            .public_key(key.public_key())
            .serial_number(x509.random_serial_number())
            .not_valid_before(datetime.datetime(2020, 1, 1, tzinfo=UTC))
            .not_valid_after(not_after)
            .add_extension(x509.BasicConstraints(ca=is_ca, path_length=None), critical=True)
            .sign(issuer_key, hashes.SHA256()))
    return cert, key


def generate_cpa(certificates_per_party):
    """Builds a minimal CPA; certificates_per_party holds per party a dict of certId -> list of X509Certificate texts."""
    root = lxml_etree.Element('{' + CPA_NAMESPACE + '}CollaborationProtocolAgreement')
    lxml_etree.SubElement(root, '{' + CPA_NAMESPACE + '}Start').text = '2020-01-01T00:00:00Z'
    lxml_etree.SubElement(root, '{' + CPA_NAMESPACE + '}End').text = '2099-01-01T00:00:00Z'
    for certificates in certificates_per_party:
        party_info = lxml_etree.SubElement(root, '{' + CPA_NAMESPACE + '}PartyInfo')
        for cert_id, chain in certificates.items():
            certificate = lxml_etree.SubElement(party_info, '{' + CPA_NAMESPACE + '}Certificate')
            certificate.set('{' + CPA_NAMESPACE + '}certId', cert_id)
            key_info = lxml_etree.SubElement(certificate, '{' + DS_NAMESPACE + '}KeyInfo')
            for text in chain:
                x509_data = lxml_etree.SubElement(key_info, '{' + DS_NAMESPACE + '}X509Data')
                lxml_etree.SubElement(x509_data, '{' + DS_NAMESPACE + '}X509Certificate').text = text
    return root


def encode(cert):
    return base64.b64encode(cert.public_bytes(serialization.Encoding.DER)).decode('utf-8')


class TestCertificateExpiry(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.root_cert, root_key = generate_certificate('Root', datetime.datetime(2040, 1, 1, tzinfo=UTC), is_ca=True)
        cls.intermediate_cert, intermediate_key = generate_certificate(
            'Intermediate', datetime.datetime(2030, 6, 15, 12, 30, 5, tzinfo=UTC), (cls.root_cert, root_key), is_ca=True)
        cls.leaf_cert, _ = generate_certificate(
            'Leaf', datetime.datetime(2035, 1, 1, tzinfo=UTC), (cls.intermediate_cert, intermediate_key))
        cls.chain = [encode(cls.leaf_cert), encode(cls.intermediate_cert), encode(cls.root_cert)]

    def test_certificate_type_classifies_chain(self):
        """Test: certificate_type_classifies_chain"""
        self.assertEqual(certificate_type(self.leaf_cert), 'leaf')
        self.assertEqual(certificate_type(self.intermediate_cert), 'intermediate')
        self.assertEqual(certificate_type(self.root_cert), 'root')

    def test_get_certificate_expiries_reads_whole_chain(self):
        """Test: get_certificate_expiries_reads_whole_chain"""
        expiries = get_certificate_expiries(generate_cpa([{'A_cert': self.chain}, {'B_cert': self.chain[:1]}]))
        self.assertCountEqual([(e['cert_id'], e['type']) for e in expiries],
                              [('A_cert', 'leaf'), ('A_cert', 'intermediate'), ('A_cert', 'root'), ('B_cert', 'leaf')])

    def test_get_certificate_expiries_sorted_on_expiry(self):
        """Test: get_certificate_expiries_sorted_on_expiry"""
        expiries = get_certificate_expiries(generate_cpa([{'A_cert': self.chain}, {}]))
        self.assertEqual([e['common_name'] for e in expiries], ['Intermediate', 'Leaf', 'Root'])

    def test_certificate_common_name(self):
        """Test: certificate_common_name"""
        self.assertEqual(certificate_common_name(self.leaf_cert), 'Leaf')

    def test_get_certificate_expiries_skips_unreadable_certificate(self):
        """Test: get_certificate_expiries_skips_unreadable_certificate"""
        logger = Mock()
        expiries = get_certificate_expiries(generate_cpa([{'A_cert': ['bm90IGEgY2VydA==', self.chain[0]]}, {}]), logger)
        self.assertEqual(len(expiries), 1)
        logger.error.assert_called_once()

    def test_get_first_expiring_certificate_returns_earliest(self):
        """Test: get_first_expiring_certificate_returns_earliest"""
        first_expiring = get_first_expiring_certificate(generate_cpa([{'A_cert': self.chain[:1]}, {'B_cert': self.chain}]))
        self.assertEqual(first_expiring['type'], 'intermediate')
        self.assertEqual(first_expiring['cert_id'], 'B_cert')
        self.assertEqual(first_expiring['subject'], 'CN=Intermediate')
        self.assertEqual(first_expiring['not_after'], datetime.datetime(2030, 6, 15, 12, 30, 5, tzinfo=UTC))

    def test_get_first_expiring_certificate_without_certificates(self):
        """Test: get_first_expiring_certificate_without_certificates"""
        self.assertIsNone(get_first_expiring_certificate(generate_cpa([{}, {}])))


class TestSetEndDateFromCertificates(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.logger = Mock()
        self.master.root = generate_cpa([{'A_cert': TestCertificateExpiry.chain}, {}])
        self.general_tab = General(self.master, self.logger)
        self.general_tab.create()

    def tearDown(self):
        self.master.destroy()

    @classmethod
    def setUpClass(cls):
        TestCertificateExpiry.setUpClass()

    def test_set_end_date_from_certificates_uses_first_expiring(self):
        """Test: set_end_date_from_certificates_uses_first_expiring"""
        self.general_tab.button_end_date_from_certificates.invoke()
        self.assertEqual(self.general_tab.entry_end_date.get(), '2030-06-15T12:30:05Z')
        self.assertEqual(self.master.root.find('{' + CPA_NAMESPACE + '}End').text, '2030-06-15T12:30:05Z')
        self.assertIn('intermediate certificate', self.logger.info.call_args[0][0])

    def test_set_end_date_from_certificates_lists_certificates_in_debug(self):
        """Test: set_end_date_from_certificates_lists_certificates_in_debug"""
        self.general_tab.set_end_date_from_certificates()
        self.assertEqual([call.args[0] for call in self.logger.debug.call_args_list],
                         ["Certificate 1/3: Intermediate expires 2030-06-15T12:30:05Z (intermediate, certId A_cert)",
                          "Certificate 2/3: Leaf expires 2035-01-01T00:00:00Z (leaf, certId A_cert)",
                          "Certificate 3/3: Root expires 2040-01-01T00:00:00Z (root, certId A_cert)"])

    def test_set_start_date_to_now(self):
        """Test: set_start_date_to_now"""
        before = datetime.datetime.now(UTC).replace(microsecond=0)
        self.general_tab.button_start_date_now.invoke()
        after = datetime.datetime.now(UTC)
        start_date = self.general_tab.entry_start_date.get()
        self.assertEqual(self.master.root.find('{' + CPA_NAMESPACE + '}Start').text, start_date)
        parsed = datetime.datetime.strptime(start_date, '%Y-%m-%dT%H:%M:%SZ').replace(tzinfo=UTC)
        self.assertTrue(before <= parsed <= after)
        self.assertEqual(self.master.root.find('{' + CPA_NAMESPACE + '}End').text, '2099-01-01T00:00:00Z')

    def test_set_start_date_to_now_without_cpa(self):
        """Test: set_start_date_to_now_without_cpa"""
        self.master.root = None
        self.general_tab.set_start_date_to_now()
        self.assertEqual(self.general_tab.entry_start_date.get(), '')
        self.logger.error.assert_called_with("No CPA loaded")

    def test_set_end_date_from_certificates_without_cpa(self):
        """Test: set_end_date_from_certificates_without_cpa"""
        self.master.root = None
        self.general_tab.set_end_date_from_certificates()
        self.assertEqual(self.general_tab.entry_end_date.get(), '')
        self.logger.error.assert_called_with("No CPA loaded")

    def test_set_end_date_from_certificates_without_certificates(self):
        """Test: set_end_date_from_certificates_without_certificates"""
        self.master.root = generate_cpa([{}, {}])
        self.general_tab.set_end_date_from_certificates()
        self.assertEqual(self.master.root.find('{' + CPA_NAMESPACE + '}End').text, '2099-01-01T00:00:00Z')
        self.logger.error.assert_called_with("No certificates found in the CPA, end date not changed")


if __name__ == '__main__':
    unittest.main()
