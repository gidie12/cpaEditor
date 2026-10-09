import base64
import datetime

from cryptography.x509 import load_der_x509_certificate, BasicConstraints, ExtensionNotFound
from cryptography.x509.oid import NameOID

CPA_NAMESPACE = 'http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd'
DS_NAMESPACE = 'http://www.w3.org/2000/09/xmldsig#'
CPA_DATE_FORMAT = '%Y-%m-%dT%H:%M:%SZ'


def certificate_type(cert):
    """
    Determines the position of a certificate in its chain.

    Args:
        cert (x509.Certificate): The certificate to classify.

    Returns:
        str: 'root' for a self-issued CA, 'intermediate' for any other CA, otherwise 'leaf'.

    Test Functions:
        - test_certificate_type_classifies_chain
    """
    try:
        is_ca = cert.extensions.get_extension_for_class(BasicConstraints).value.ca
    except ExtensionNotFound:
        is_ca = False
    if not is_ca:
        return 'leaf'
    return 'root' if cert.issuer == cert.subject else 'intermediate'


def certificate_common_name(cert):
    """
    Returns the common name of a certificate, or its full subject when it has no common name.

    Test Functions:
        - test_certificate_common_name
    """
    common_names = cert.subject.get_attributes_for_oid(NameOID.COMMON_NAME)
    return common_names[0].value if common_names else cert.subject.rfc4514_string()


def get_key_info_certificates(key_info, logger=None, now=None):
    """
    Reads the details of every X509Certificate of a KeyInfo element.

    Args:
        key_info (lxml.etree.Element): The KeyInfo element, or None.
        logger (logging.Logger): Optional logger for certificates that cannot be read.
        now (datetime.datetime): The moment to compare the expiry date with, the current time by default.

    Returns:
        list: One dict per certificate in the order of the KeyInfo with the keys 'type', 'common_name', 'subject',
              'issuer', 'serial_number', 'not_before' and 'not_after' (UTC datetimes) and 'expired' (bool).

    Test Functions:
        - test_get_key_info_certificates_reads_details
        - test_get_key_info_certificates_marks_expired
        - test_get_key_info_certificates_without_key_info
    """
    certificates = []
    if key_info is None:
        return certificates
    now = now or datetime.datetime.now(datetime.timezone.utc)
    for x509_certificate in key_info.iter('{' + DS_NAMESPACE + '}X509Certificate'):
        try:
            cert = load_der_x509_certificate(base64.b64decode(x509_certificate.text))
        except Exception as e:
            if logger:
                logger.error(f"Could not read a certificate: {e}")
            continue
        certificates.append({'type': certificate_type(cert),
                             'common_name': certificate_common_name(cert),
                             'subject': cert.subject.rfc4514_string(),
                             'issuer': cert.issuer.rfc4514_string(),
                             'serial_number': format(cert.serial_number, 'X'),
                             'not_before': cert.not_valid_before_utc,
                             'not_after': cert.not_valid_after_utc,
                             'expired': cert.not_valid_after_utc < now})
    return certificates


def get_certificate_expiries(root, logger=None):
    """
    Collects the expiry date of every X509Certificate (leaf, intermediate and root) in the CPA.

    Args:
        root (lxml.etree.Element): The root element of the CPA.
        logger (logging.Logger): Optional logger for certificates that cannot be read.

    Returns:
        list: One dict per certificate with the keys 'not_after' (UTC datetime), 'type', 'common_name', 'subject'
              and 'cert_id', sorted on 'not_after' so the certificate that expires first comes first.

    Test Functions:
        - test_get_certificate_expiries_reads_whole_chain
        - test_get_certificate_expiries_sorted_on_expiry
        - test_get_certificate_expiries_skips_unreadable_certificate
    """
    expiries = []
    for certificate_element in root.iter('{' + CPA_NAMESPACE + '}Certificate'):
        cert_id = certificate_element.get('{' + CPA_NAMESPACE + '}certId')
        for x509_certificate in certificate_element.iter('{' + DS_NAMESPACE + '}X509Certificate'):
            try:
                cert = load_der_x509_certificate(base64.b64decode(x509_certificate.text))
            except Exception as e:
                if logger:
                    logger.error(f"Could not read a certificate of certId '{cert_id}': {e}")
                continue
            expiries.append({'not_after': cert.not_valid_after_utc,
                             'type': certificate_type(cert),
                             'common_name': certificate_common_name(cert),
                             'subject': cert.subject.rfc4514_string(),
                             'cert_id': cert_id})
    return sorted(expiries, key=lambda expiry: expiry['not_after'])


def get_first_expiring_certificate(root, logger=None):
    """
    Finds the certificate in the CPA that expires first.

    Returns:
        dict: The entry of get_certificate_expiries with the earliest 'not_after', or None without certificates.

    Test Functions:
        - test_get_first_expiring_certificate_returns_earliest
        - test_get_first_expiring_certificate_without_certificates
    """
    expiries = get_certificate_expiries(root, logger)
    return expiries[0] if expiries else None
