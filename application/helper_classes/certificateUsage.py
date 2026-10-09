from lxml import etree as lxml_etree

from application.helper_classes.certificateExpiry import CPA_NAMESPACE

# Attributes that identify the element a certificate reference belongs to
CONTEXT_ID_ATTRIBUTES = ('transportId', 'docExchangeId', 'securityId', 'channelId')


def describe_reference(reference):
    """
    Describes where a certificate reference is located in the CPA.

    Args:
        reference (lxml.etree.Element): An element that refers to a certificate, for example ClientCertificateRef.

    Returns:
        str: The name of the reference and the nearest element with an id, for example
             "ClientCertificateRef in Transport 'PartnerA_Transport'". Without such an element the parent is named.

    Test Functions:
        - test_describe_reference_names_element_with_id
        - test_describe_reference_falls_back_to_parent
    """
    name = lxml_etree.QName(reference).localname
    parent = reference.getparent()
    ancestor = parent
    while ancestor is not None:
        for attribute in CONTEXT_ID_ATTRIBUTES:
            context_id = ancestor.get('{' + CPA_NAMESPACE + '}' + attribute)
            if context_id:
                return f"{name} in {lxml_etree.QName(ancestor).localname} '{context_id}'"
        ancestor = ancestor.getparent()
    if parent is None:
        return name
    return f"{name} in {lxml_etree.QName(parent).localname}"


def get_certificate_usages(root):
    """
    Collects per certId the elements in the CPA that refer to that certificate.

    Every element with a certId attribute other than Certificate itself is a reference: ClientCertificateRef,
    ServerCertificateRef, SigningCertificateRef, EncryptionCertificateRef, ApplicationCertificateRef and
    AnchorCertificateRef.

    Args:
        root (lxml.etree.Element): The root element of the CPA.

    Returns:
        dict: certId -> list of descriptions (see describe_reference) in the order of the CPA. A certId without
              references is not in the dict.

    Test Functions:
        - test_get_certificate_usages_collects_references
        - test_get_certificate_usages_ignores_certificate_element
        - test_get_certificate_usages_of_full_cpa
    """
    cert_id_attribute = '{' + CPA_NAMESPACE + '}certId'
    certificate_tag = '{' + CPA_NAMESPACE + '}Certificate'
    usages = {}
    for element in root.iter():
        if not isinstance(element.tag, str) or element.tag == certificate_tag:
            continue
        cert_id = element.get(cert_id_attribute)
        if cert_id:
            usages.setdefault(cert_id, []).append(describe_reference(element))
    return usages

