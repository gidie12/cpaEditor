import os
import unittest
from unittest.mock import Mock, patch
import tkinter as tk

from lxml import etree

from application.app_classes.certificates_tab import Certificates
from application.helper_classes.certificateExpiry import CPA_NAMESPACE
from application.helper_classes.certificateUsage import describe_reference, get_certificate_usages
from application.helper_classes.directories import TESTS_DIR

FULL_TEST_CPA = os.path.join(TESTS_DIR, 'resources', 'cpas', 'full_test_cpa.xml')

CPA = f"""<tns:CollaborationProtocolAgreement xmlns:tns="{CPA_NAMESPACE}">
  <tns:PartyInfo tns:partyName="A">
    <tns:CollaborationRole>
      <tns:ApplicationCertificateRef tns:certId="A_app"/>
    </tns:CollaborationRole>
    <tns:Certificate tns:certId="A_client"/>
    <tns:Certificate tns:certId="A_app"/>
    <tns:Certificate tns:certId="A_unused"/>
    <tns:SecurityDetails tns:securityId="A_security">
      <tns:TrustAnchors>
        <tns:AnchorCertificateRef tns:certId="A_client"/>
      </tns:TrustAnchors>
    </tns:SecurityDetails>
    <tns:Transport tns:transportId="A_transport">
      <tns:TransportSender>
        <!-- comment between the elements -->
        <tns:TransportClientSecurity>
          <tns:ClientCertificateRef tns:certId="A_client"/>
        </tns:TransportClientSecurity>
      </tns:TransportSender>
    </tns:Transport>
  </tns:PartyInfo>
</tns:CollaborationProtocolAgreement>"""


class TestCertificateUsage(unittest.TestCase):

    def setUp(self):
        self.root = etree.fromstring(CPA)

    def find(self, name):
        return self.root.find('.//{' + CPA_NAMESPACE + '}' + name)

    def test_describe_reference_names_element_with_id(self):
        """Test: describe_reference_names_element_with_id"""
        self.assertEqual(describe_reference(self.find('ClientCertificateRef')),
                         "ClientCertificateRef in Transport 'A_transport'")
        self.assertEqual(describe_reference(self.find('AnchorCertificateRef')),
                         "AnchorCertificateRef in SecurityDetails 'A_security'")

    def test_describe_reference_falls_back_to_parent(self):
        """Test: describe_reference_falls_back_to_parent"""
        self.assertEqual(describe_reference(self.find('ApplicationCertificateRef')),
                         "ApplicationCertificateRef in CollaborationRole")

    def test_get_certificate_usages_collects_references(self):
        """Test: get_certificate_usages_collects_references"""
        self.assertEqual(get_certificate_usages(self.root), {
            'A_app': ["ApplicationCertificateRef in CollaborationRole"],
            'A_client': ["AnchorCertificateRef in SecurityDetails 'A_security'",
                         "ClientCertificateRef in Transport 'A_transport'"],
        })

    def test_get_certificate_usages_ignores_certificate_element(self):
        """Test: get_certificate_usages_ignores_certificate_element"""
        self.assertNotIn('A_unused', get_certificate_usages(self.root))

    def test_get_certificate_usages_of_full_cpa(self):
        """Test: get_certificate_usages_of_full_cpa"""
        usages = get_certificate_usages(etree.parse(FULL_TEST_CPA).getroot())
        self.assertEqual(usages['BROKER_ClientCert'],
                         ["AnchorCertificateRef in SecurityDetails 'BROKER_TransportSecurity'",
                          "ClientCertificateRef in Transport 'BROKER_T_HTTPS'"])


class TestLoadShowsUsage(unittest.TestCase):

    def setUp(self):
        self.master = tk.Tk()
        self.master.namespace_uri = CPA_NAMESPACE
        self.master.host = 'mac'
        self.master.root = etree.fromstring(CPA)
        self.cert_tab = Certificates(self.master, Mock())
        self.cert_tab.create()

    def tearDown(self):
        self.master.destroy()

    @patch('application.app_classes.certificates_tab.CPAParser')
    def test_load_shows_unused_certificates_in_grey(self, MockCPAParser):
        """Test: load_shows_unused_certificates_in_grey"""
        mock_parser = MockCPAParser.return_value
        mock_parser.get_party_name_partner_a.return_value = 'A'
        mock_parser.get_certificate_party_a_elements.return_value = self.master.root.findall(
            './/{' + CPA_NAMESPACE + '}Certificate')
        mock_parser.get_certificate_party_b_elements.return_value = []

        self.cert_tab.load()

        tree = self.cert_tab.tree_editor
        rows = {tree.item(item, 'values')[0]: item for item in tree.get_children(tree.get_children()[0])}

        self.assertEqual(tree.item(rows['A_client'], 'tags'), '')
        self.assertEqual(tree.item(rows['A_app'], 'tags'), '')
        self.assertEqual(tree.item(rows['A_unused'], 'tags'), ('unused',))
        self.assertEqual([tree.item(child, 'text') for child in tree.get_children(rows['A_client'])], ["KeyInfo"])


if __name__ == '__main__':
    unittest.main()
