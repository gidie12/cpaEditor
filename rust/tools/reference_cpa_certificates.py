"""Writes what the Python CPA Editor does with the general fields, comments, transports, the XML editor and
certificates, so the Rust tests (cpa-core/tests/cpa_parity.rs, certificates_parity.rs) can compare.

Run from the rust directory with a Python that has the packages of requirements.txt:
    PYTHONPATH=.. python tools/reference_cpa_certificates.py

Output in cpa-core/tests/reference/cpa_certificates/:
- certificates.json: per test CPA the details of every X509Certificate, the expiry list, the certificate usages,
  the general fields and the certificate entries per party; the details of the PEM files of the test resources and
  of generated fixture certificates; describe_cpa_date for a list of texts and offsets.
- fixtures.json, new_rsa.pem: generated certificates for the ported unit tests (Rust cannot make certificates).
- cases.json and one XML file per case: the CPA after operations done by the real Python code. Elements are
  addressed by a path of element-child indexes from the root, computed right before each operation.
- key_info_full_test_cpa.pem: key_info_to_pem of the first KeyInfo of full_test_cpa.xml.

Deviations applied to the Python side so the bytes are comparable (the Rust code fixes these on purpose):
- add_comment writes xml:lang with the chosen language instead of the attribute {xml}type="nl-NL".
- clone_element inserts copy.deepcopy(element) after the element; the Python tab inserted the element itself
  at its own position again, which changed nothing.
- the XML editor's add element uses lxml instead of xml.etree.ElementTree, which raised a TypeError.
"""
import base64
import contextlib
import copy
import datetime
import io
import json
import pathlib
from unittest import mock

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec, rsa
from cryptography.x509.name import _ASN1Type
from cryptography.x509.oid import NameOID, ObjectIdentifier
from lxml import etree

from application.app_classes import certificates_tab, xml_editor_tab
from application.app_classes.certificates_tab import Certificates, key_info_to_pem
from application.app_classes.transport_tab import Transport
from application.app_classes.xml_editor_tab import XMLEditor
from application.helper_classes.certificateExpiry import (certificate_common_name, certificate_type,
                                                          get_certificate_expiries, get_key_info_certificates)
from application.helper_classes.certificateUsage import get_certificate_usages
from application.helper_classes.cpaCreator import CPACreator
from application.helper_classes.cpaDates import describe_cpa_date
from application.helper_classes.cpaParser import CPAParser

CPAS = pathlib.Path('../application/tests/resources/cpas')
CERTS = pathlib.Path('../application/tests/resources/certs')
OUT = pathlib.Path('cpa-core/tests/reference/cpa_certificates')
PARSER = etree.XMLParser(resolve_entities=False, no_network=True, load_dtd=False)
CPA = 'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd'
DS = 'http://www.w3.org/2000/09/xmldsig#'
XML = 'http://www.w3.org/XML/1998/namespace'
XSI = 'http://www.w3.org/2001/XMLSchema-instance'
UTC = datetime.timezone.utc
FORMAT = '%Y-%m-%dT%H:%M:%SZ'


class FailingLogger:
    """A logger that records messages; error() is collected so callers can check them."""

    def __init__(self):
        self.errors = []
        self.infos = []

    def error(self, message):
        self.errors.append(message)

    def info(self, message):
        self.infos.append(message)

    def debug(self, message):
        pass

    def warning(self, message):
        pass


def quiet(function, *args):
    """Calls a function of the Python version that prints to stdout."""
    with contextlib.redirect_stdout(io.StringIO()):
        return function(*args)


def tostring(tree):
    return etree.tostring(tree, encoding='UTF-8', xml_declaration=True)


def elements(element):
    return [child for child in element if isinstance(child.tag, str)]


def path_of(element):
    path = []
    while element.getparent() is not None:
        parent = element.getparent()
        path.insert(0, elements(parent).index(element))
        element = parent
    return path


def details(cert):
    return {'type': certificate_type(cert),
            'common_name': certificate_common_name(cert),
            'subject': cert.subject.rfc4514_string(),
            'issuer': cert.issuer.rfc4514_string(),
            'serial': format(cert.serial_number, 'X'),
            'serial_decimal': str(cert.serial_number),
            'not_before': cert.not_valid_before_utc.strftime(FORMAT),
            'not_after': cert.not_valid_after_utc.strftime(FORMAT),
            'der_base64': base64.b64encode(cert.public_bytes(serialization.Encoding.DER)).decode()}


# ------------------------------------------------------------------------------------------------------------------
# Generated certificates
# ------------------------------------------------------------------------------------------------------------------

def generate(name, serial, not_after, issuer=None, is_ca=False, key=None):
    key = key or ec.generate_private_key(ec.SECP256R1())
    subject = name if isinstance(name, x509.Name) else x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, name)])
    issuer_name, issuer_key = (issuer[0].subject, issuer[1]) if issuer else (subject, key)
    builder = (x509.CertificateBuilder().subject_name(subject).issuer_name(issuer_name)
               .public_key(key.public_key()).serial_number(serial)
               .not_valid_before(datetime.datetime(2020, 1, 1, tzinfo=UTC)).not_valid_after(not_after))
    if is_ca is not None:
        builder = builder.add_extension(x509.BasicConstraints(ca=is_ca, path_length=None), critical=True)
    return builder.sign(issuer_key, hashes.SHA256()), key


def generate_fixtures():
    root, root_key = generate('Root', 0x1001, datetime.datetime(2040, 1, 1, tzinfo=UTC), is_ca=True)
    intermediate, intermediate_key = generate('Intermediate', 0x1002, datetime.datetime(2030, 6, 15, 12, 30, 5, tzinfo=UTC),
                                              (root, root_key), is_ca=True)
    leaf, _ = generate('Leaf', 0x1003, datetime.datetime(2035, 1, 1, tzinfo=UTC), (intermediate, intermediate_key))
    leaf_of_root, _ = generate('Leaf', 0xF0E1D2C3B4A5968778695A4B3C2D1E0F,
                               datetime.datetime(2035, 6, 15, 12, 30, 5, tzinfo=UTC), (root, root_key))
    old, _ = generate('Old', 0x80, datetime.datetime(2021, 1, 1, tzinfo=UTC))
    rsa_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    new, _ = generate('New', 0x7FFFFFFFFFFFFFFF, datetime.datetime(2045, 1, 1, tzinfo=UTC), is_ca=None, key=rsa_key)
    nasty_name = x509.Name([
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.COUNTRY_NAME, 'NL')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, 'Zuid-Holland',
                                                           _type=_ASN1Type.BMPString)]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.ORGANIZATION_NAME, 'Org, "Inc" + <x>;\\'),
                                        x509.NameAttribute(NameOID.ORGANIZATIONAL_UNIT_NAME, 'Unit')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.LOCALITY_NAME, ' leading and trailing ')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.STREET_ADDRESS, '#1 Street')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.DOMAIN_COMPONENT, 'example')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.USER_ID, 'uid1')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.EMAIL_ADDRESS, 'a@b.nl')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(ObjectIdentifier('1.2.3.4.5'), 'other oid')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.X500_UNIQUE_IDENTIFIER, b'\x00\x12\xab',
                                                           _type=_ASN1Type.BitString)]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.SERIAL_NUMBER, ' ')]),
        x509.RelativeDistinguishedName([x509.NameAttribute(NameOID.COMMON_NAME, 'Ünïcode CN')]),
    ])
    nasty, _ = generate(nasty_name, 0x00FF, datetime.datetime(2031, 3, 4, 5, 6, 7, tzinfo=UTC), (root, root_key),
                        is_ca=True)
    no_cn, _ = generate(x509.Name([x509.NameAttribute(NameOID.ORGANIZATION_NAME, 'Only Org')]), 1,
                        datetime.datetime(2032, 1, 1, tzinfo=UTC), is_ca=None)
    fixtures = {'root': root, 'intermediate': intermediate, 'leaf': leaf, 'leaf_of_root': leaf_of_root, 'old': old,
                'new': new, 'nasty': nasty, 'no_cn': no_cn}
    (OUT / 'new_rsa.pem').write_bytes(new.public_bytes(serialization.Encoding.PEM))
    return {name: details(cert) for name, cert in fixtures.items()}


# ------------------------------------------------------------------------------------------------------------------
# Reading
# ------------------------------------------------------------------------------------------------------------------

def cpa_report(path):
    root = etree.parse(str(path), PARSER).getroot()
    certificates = []
    for x509_certificate in root.iter('{' + DS + '}X509Certificate'):
        logger = FailingLogger()
        found = get_key_info_certificates(x509_certificate, logger)
        if found:
            entry = found[0]
            certificates.append({'type': entry['type'], 'common_name': entry['common_name'],
                                 'subject': entry['subject'], 'issuer': entry['issuer'],
                                 'serial': entry['serial_number'],
                                 'not_before': entry['not_before'].strftime(FORMAT),
                                 'not_after': entry['not_after'].strftime(FORMAT)})
        else:
            certificates.append({'error': True})
    logger = FailingLogger()
    expiries = [[e['cert_id'], e['type'], e['common_name'], e['subject'], e['not_after'].strftime(FORMAT)]
                for e in get_certificate_expiries(root, logger)]
    report = {'certificates': certificates, 'expiries': expiries, 'expiry_errors': len(logger.errors),
              'usages': get_certificate_usages(root)}
    try:
        parser = quiet(CPAParser, root)
        getters = ['get_cpa_id', 'get_cpa_status', 'get_party_name_partner_a', 'get_party_name_partner_b',
                   'get_party_id_partner_a', 'get_party_id_partner_b', 'get_party_ids_type_partner_a',
                   'get_party_ids_type_partner_b', 'get_cpa_start_date', 'get_cpa_end_date']
        report['general'] = [quiet(getattr(parser, getter)) or '' for getter in getters]
        report['parties'] = [
            {'transports': [path_of(t) for t in parser.get_transport_items_partner_a()],
             'cert_ids': parser.get_cert_ids_partner_a(), 'security_ids': parser.get_security_ids_partner_a()},
            {'transports': [path_of(t) for t in parser.get_transport_items_partner_b()],
             'cert_ids': parser.get_cert_ids_partner_b(), 'security_ids': parser.get_security_ids_partner_b()}]
    except Exception:
        report['general'] = None
        report['parties'] = None
    entries = []
    for party_info in root.findall('{' + CPA + '}PartyInfo'):
        logger = FailingLogger()
        fake = mock.Mock(logger=logger)
        result = Certificates.generate_partner_keyinfo_certid_list(
            fake, party_info.findall('{' + CPA + '}Certificate'), '{' + CPA + '}certId')
        entries.append({'cert_ids': list(result), 'has_key_info': [items[1] is not None for items in result.values()],
                        'errors': logger.errors})
    report['entries'] = entries
    return report


def pem_report():
    report = {}
    for path in sorted(CERTS.rglob('*')):
        if path.suffix not in ('.pem', '.cer', '.crt') or path.name.endswith('_key.pem'):
            continue
        certs = x509.load_pem_x509_certificates(path.read_bytes())
        report[str(path.relative_to(CERTS))] = [details(cert) for cert in certs]
    return report


DATE_TEXTS = ['2026-10-09T10:44:32Z', '2026-12-31T23:30:00Z', '2026-10-09T12:44:32+02:00', '2026-10-09T10:44:32',
              '09-10-2026', '', '   ', ' 2026-10-09T10:44:32Z ', '2026-10-09 10:44:32+02:00', '20261009T104432Z',
              '2026-10-09T10:44Z', '2026-10-09T10Z', '2026-10-09T10:44:32.5Z', '2026-10-09T10:44:32,123456789Z',
              '2026-10-09T10:44:32+0530', '2026-10-09T10:44:32-03', '2026-10-09T10:44:32+02:00:30', '2026-10-09',
              '2026-10-09Z', '2026-W41-5T10:00Z', '2026W415T10:00Z', '2026-W53-1T00:00Z', '2026-10-09x10:44:32Z',
              '2026-13-09T10:44:32Z', '2026-02-30T00:00:00Z', '2026-10-09T24:00:00Z', '2026-10-09T10:44:32 Z',
              '2026-10-09T104432Z', '2026-10-09T10:4432Z', '2026-10-09T10:44:32.Z', '2026-10-09T10:44:32+2:00',
              '2026-10-09T10:44:32+23:59', '2026-10-09T10:44:32+24:00', '2026-10-09T10:44:32+05:99',
              '2026-10-09T10:44:32:5Z', '2026-10-09T10:44:60Z', 'tomorrow', '2026-1-09T10:44:32Z',
              '9999-12-30T23:59:59Z', '2026-10-09T10:44:32.123+01:00', '2026-10-09T10:44:32Zx']
DATE_OFFSETS = [7200, 3600, 0, -12600, 19800, 18030]


def date_report():
    return [[text, offset, describe_cpa_date(text, datetime.timezone(datetime.timedelta(seconds=offset)))]
            for text in DATE_TEXTS for offset in DATE_OFFSETS]


# ------------------------------------------------------------------------------------------------------------------
# Changes
# ------------------------------------------------------------------------------------------------------------------

GENERAL_KEYS = {('CpaId', None): 'CPAId', ('Status', None): 'cpaStatus', ('PartyName', 0): 'partyNamePartnerA',
                ('PartyName', 1): 'partyNamePartnerB', ('PartyId', 0): 'partyIdPartnerA',
                ('PartyId', 1): 'partyIdPartnerB', ('PartyIdType', 0): 'partyIdTypePartnerA',
                ('PartyIdType', 1): 'partyIdTypePartnerB', ('StartDate', None): 'cpaStartDate',
                ('EndDate', None): 'cpaEndDate'}


def run(root, op, locate):
    """Runs one operation with the real Python code; locate(root) finds the element of the operation."""
    logger = FailingLogger()
    element = locate(root) if locate else None
    record = dict(op)
    if element is not None:
        record['element'] = path_of(element)
    kind = op['op']
    if kind == 'set_general':
        key = GENERAL_KEYS[(op['field'], op.get('index'))]
        quiet(CPACreator(root, logger).apply_changes, {key: op['value']})
    elif kind == 'add_comment':
        # comment_tab.add_item, with xml:lang instead of {xml}type="nl-NL"
        comment_object = etree.Element('{' + CPA + '}Comment')
        comment_object.text = op['text']
        comment_object.set('{' + XML + '}lang', op['lang'])
        quiet(CPACreator(root, logger).apply_changes, {'addXMLElement': [comment_object, root]})
    elif kind == 'set_comment':
        comment = quiet(CPAParser, root).get_comments()[op['index']]
        quiet(CPACreator(root, logger).apply_changes, {'changeXMLElement': [op['text'], comment]})
    elif kind == 'delete_comment':
        comment = quiet(CPAParser, root).get_comments()[op['index']]
        quiet(CPACreator(root, logger).apply_changes, {'deleteXMLElement': [comment]})
    elif kind == 'replace_key_info':
        key_info = element.find('{' + DS + '}KeyInfo')
        fake = mock.Mock(logger=logger, xml_element_mapping={'item': [key_info, element]})
        fake.tree_editor.selection.return_value = ['item']
        with mock.patch.object(certificates_tab.filedialog, 'askopenfilename', return_value=op['pem']):
            Certificates.open_certificate(fake)
        assert any('successfully' in message for message in logger.infos), logger.errors
    elif kind == 'set_cpa_attribute':
        Transport.change_xml_object(None, 'Attribute', '{' + CPA + '}' + op['local'], element, op['value'])
    elif kind == 'set_element_text':
        Transport.change_xml_object(None, 'Element', None, element, op['value'])
    elif kind == 'clone_element':
        parent = element.getparent()
        parent.insert(parent.index(element) + 1, copy.deepcopy(element))
    elif kind == 'editor_set_attribute':
        XMLEditor.change_xml_object(None, 'Attribute', op['name'], element, op['value'])
    elif kind == 'editor_add_attribute':
        XMLEditor.add_xml_object(mock.Mock(), 'Attribute', op['name'], element, op['value'])
    elif kind == 'editor_add_element':
        fake = mock.Mock()
        fake.master.namespace_uri = etree.QName(root).namespace
        with mock.patch.object(xml_editor_tab, 'Et', etree):
            XMLEditor.add_xml_object(fake, 'Element', op['name'], element, op['value'])
    elif kind == 'editor_delete_element':
        fake = mock.Mock(xml_element_mapping={'parent': element.getparent(), 'item': element})
        fake.tree_editor.parent.return_value = 'parent'
        XMLEditor.delete_xml_object(fake, 'Element', etree.QName(element).localname, element, 'item')
    elif kind == 'editor_delete_attribute':
        fake = mock.Mock(xml_element_mapping={'item': element})
        XMLEditor.delete_xml_object(fake, 'Attribute', op['name'], element, 'item')
    else:
        raise ValueError(kind)
    assert not logger.errors, logger.errors
    return record


def first(tag, namespace=CPA, party=None):
    def locate(root):
        scope = root if party is None else root.findall('{' + CPA + '}PartyInfo')[party]
        return next(scope.iter('{' + namespace + '}' + tag))
    return locate


def nth(tag, index, namespace=CPA):
    return lambda root: list(root.iter('{' + namespace + '}' + tag))[index]


def root_element(root):
    return root


LEAF_CHAIN = '../application/tests/resources/certs/leaf/certs/Leaf Certificate 1_full_chain.pem'
LEAF_2 = '../application/tests/resources/certs/leaf/certs/Leaf Certificate 2.pem'

CASES = [
    ('general_test_cpa', 'test_cpa.xml', [
        ({'op': 'set_general', 'field': 'CpaId', 'value': 'cpa_rust_parity'}, None),
        ({'op': 'set_general', 'field': 'Status', 'value': 'signed'}, None),
        ({'op': 'set_general', 'field': 'PartyName', 'index': 0, 'value': 'Café Ünïcode'}, None),
        ({'op': 'set_general', 'field': 'PartyName', 'index': 1, 'value': 'Partner <B> & "co"\tline'}, None),
        ({'op': 'set_general', 'field': 'PartyId', 'index': 0, 'value': '123456789'}, None),
        ({'op': 'set_general', 'field': 'PartyId', 'index': 1, 'value': 'B&B <id>'}, None),
        ({'op': 'set_general', 'field': 'PartyIdType', 'index': 0,
          'value': 'urn:oasis:names:tc:ebcore:partyid-type:unregistered'}, None),
        ({'op': 'set_general', 'field': 'PartyIdType', 'index': 1, 'value': 'urn:x'}, None),
        ({'op': 'set_general', 'field': 'StartDate', 'value': '2026-01-01T00:00:00Z'}, None),
        ({'op': 'set_general', 'field': 'EndDate', 'value': '2027-12-31T23:59:59Z'}, None),
    ]),
    ('general_test', 'test.xml', [
        ({'op': 'set_general', 'field': 'Status', 'value': 'proposed'}, None),
        ({'op': 'set_general', 'field': 'PartyId', 'index': 1, 'value': ''}, None),
        ({'op': 'set_general', 'field': 'EndDate', 'value': '2030-06-15T12:30:05Z'}, None),
    ]),
    ('comments_full_cpa_p', 'full_cpa_p.xml', [
        ({'op': 'set_comment', 'index': 0, 'text': 'Changed & <checked>'}, None),
        ({'op': 'delete_comment', 'index': 1}, None),
        ({'op': 'add_comment', 'text': 'New comment', 'lang': 'nl-NL'}, None),
        ({'op': 'add_comment', 'text': 'Second comment', 'lang': 'en-US'}, None),
        ({'op': 'delete_comment', 'index': 3}, None),
    ]),
    ('comments_test_cpa', 'test_cpa.xml', [
        ({'op': 'delete_comment', 'index': 0}, None),
        ({'op': 'add_comment', 'text': 'Added', 'lang': 'nl-NL'}, None),
    ]),
    ('comments_without_comments', 'full_cpa_p_without_comments.xml', [
        ({'op': 'add_comment', 'text': 'Only comment', 'lang': 'nl-NL'}, None),
    ]),
    ('key_info_test_cpa', 'test_cpa.xml', [
        ({'op': 'replace_key_info', 'pem': LEAF_CHAIN}, first('Certificate', party=0)),
        ({'op': 'replace_key_info', 'pem': LEAF_2}, first('Certificate', party=1)),
    ]),
    ('key_info_full_test_cpa', 'full_test_cpa.xml', [
        ({'op': 'replace_key_info', 'pem': LEAF_CHAIN}, nth('Certificate', 2)),
        ({'op': 'editor_delete_element'}, nth('KeyInfo', 0, DS)),
        ({'op': 'replace_key_info', 'pem': LEAF_2}, nth('Certificate', 0)),
    ]),
    ('key_info_new_cpa123', 'new_cpa123.xml', [
        ({'op': 'replace_key_info', 'pem': LEAF_CHAIN}, nth('Certificate', 0)),
        ({'op': 'replace_key_info', 'pem': 'cpa-core/tests/reference/cpa_certificates/new_rsa.pem'},
         nth('Certificate', 1)),
    ]),
    ('transport_full_test_cpa', 'full_test_cpa.xml', [
        ({'op': 'set_cpa_attribute', 'local': 'uri', 'value': 'https://example.org/new?a=1&b="2"'},
         first('Endpoint', party=0)),
        ({'op': 'set_cpa_attribute', 'local': 'transportId', 'value': 'Renamed_Transport'},
         first('Transport', party=1)),
        ({'op': 'set_element_text', 'value': 'HTTPS & more'}, first('TransportProtocol', party=1)),
        ({'op': 'set_cpa_attribute', 'local': 'version', 'value': '1.1'}, first('TransportProtocol', party=0)),
        ({'op': 'clone_element'}, first('Transport', party=1)),
        ({'op': 'clone_element'}, first('Endpoint', party=0)),
    ]),
    ('clone_new_cpa123', 'new_cpa123.xml', [
        ({'op': 'clone_element'}, nth('KeyInfo', 0, DS)),
        ({'op': 'clone_element'}, nth('Certificate', 5)),
        ({'op': 'clone_element'}, first('Status')),
    ]),
    ('xml_editor_full_test_cpa', 'full_test_cpa.xml', [
        ({'op': 'editor_set_attribute', 'name': '{' + CPA + '}cpaid', 'value': 'edited'}, root_element),
        ({'op': 'editor_add_attribute', 'name': 'extra', 'value': 'x'}, first('Status')),
        ({'op': 'editor_add_attribute', 'name': '{' + CPA + '}foo', 'value': 'y<'}, first('Status')),
        ({'op': 'editor_add_element', 'name': 'Comment', 'value': 'added by editor'}, root_element),
        ({'op': 'editor_add_element', 'name': 'Start', 'value': '2020-01-01T00:00:00Z'}, root_element),
        ({'op': 'editor_add_element', 'name': 'Note', 'value': ''}, first('Status')),
        ({'op': 'editor_add_element', 'name': 'Certificate', 'value': 'new'}, first('PartyInfo')),
        ({'op': 'set_element_text', 'value': 'text before children'}, first('PartyInfo')),
        ({'op': 'editor_delete_element'}, first('SimplePart')),
        ({'op': 'editor_delete_element'}, nth('Comment', 0)),
        ({'op': 'editor_delete_attribute', 'name': '{' + XSI + '}schemaLocation'}, root_element),
        ({'op': 'editor_delete_attribute', 'name': 'extra'}, first('Status')),
    ]),
    ('xml_editor_example', 'example.xml', [
        ({'op': 'editor_add_element', 'name': 'Comment', 'value': 'x'}, root_element),
        ({'op': 'editor_set_attribute', 'name': 'plain', 'value': 'v'}, root_element),
    ]),
]


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    fixtures = generate_fixtures()
    (OUT / 'fixtures.json').write_text(json.dumps(fixtures, indent=1, ensure_ascii=False), encoding='utf-8')

    report = {'cpas': {path.name: cpa_report(path) for path in sorted(CPAS.glob('*.xml'))},
              'pem_files': pem_report(), 'dates': date_report()}
    (OUT / 'certificates.json').write_text(json.dumps(report, indent=1, ensure_ascii=False), encoding='utf-8')

    cases = []
    for name, source, operations in CASES:
        root = etree.parse(str(CPAS / source), PARSER).getroot()
        records = [run(root, op, locate) for op, locate in operations]
        expected = f'{name}.xml'
        (OUT / expected).write_bytes(tostring(root.getroottree()))
        cases.append({'name': name, 'source': source, 'ops': records, 'expected': expected})
        print(name)
    (OUT / 'cases.json').write_text(json.dumps(cases, indent=1, ensure_ascii=False), encoding='utf-8')

    root = etree.parse(str(CPAS / 'full_test_cpa.xml'), PARSER).getroot()
    key_info = root.find('.//{' + DS + '}KeyInfo')
    (OUT / 'key_info_full_test_cpa.pem').write_text(key_info_to_pem(key_info), encoding='utf-8')
    (OUT / 'key_info_full_test_cpa.json').write_text(json.dumps({'source': 'full_test_cpa.xml',
                                                                 'key_info': path_of(key_info)}), encoding='utf-8')


if __name__ == '__main__':
    main()
