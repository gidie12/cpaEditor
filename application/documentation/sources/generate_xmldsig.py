import os
import base64
from lxml import etree as lxml_etree
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.backends import default_backend
from cryptography.x509 import load_pem_x509_certificate

from application.helper_classes.directories import TESTS_DIR


def create_rsa_key_value_elements(cert):
    public_numbers = cert.public_key().public_numbers()
    modulus_int = public_numbers.n
    modulus_bytes = modulus_int.to_bytes((modulus_int.bit_length() + 7) // 8, byteorder='big')
    modulus_base64 = base64.b64encode(modulus_bytes).decode('utf-8')
    exponent_int = public_numbers.e
    exponent_bytes = exponent_int.to_bytes((exponent_int.bit_length() + 7) // 8, byteorder='big')
    exponent_base64 = base64.b64encode(exponent_bytes).decode('utf-8')

    rsa_key_value_elements = [
        ('{http://www.w3.org/2000/09/xmldsig#}Modulus', modulus_base64),
        ('{http://www.w3.org/2000/09/xmldsig#}Exponent', exponent_base64)
    ]
    return rsa_key_value_elements

def create_x509_data(cert, key_info):
    x509_data = lxml_etree.SubElement(key_info, '{http://www.w3.org/2000/09/xmldsig#}X509Data')
    data_elements = [
        ('{http://www.w3.org/2000/09/xmldsig#}X509SubjectName', cert.subject.rfc4514_string()),
        ('{http://www.w3.org/2000/09/xmldsig#}X509Certificate', base64.b64encode(cert.public_bytes(encoding=serialization.Encoding.DER)).decode('utf-8'))
    ]
    for element_name, element_text in data_elements:
        element = lxml_etree.SubElement(x509_data, element_name)
        element.text = element_text

    X509IssuerSerial = lxml_etree.SubElement(x509_data, '{http://www.w3.org/2000/09/xmldsig#}X509IssuerSerial')
    X509IssuerName = lxml_etree.SubElement(X509IssuerSerial, '{http://www.w3.org/2000/09/xmldsig#}X509IssuerName')
    X509IssuerName.text = cert.issuer.rfc4514_string()

    X509SerialNumber = lxml_etree.SubElement(X509IssuerSerial, '{http://www.w3.org/2000/09/xmldsig#}X509SerialNumber')
    X509SerialNumber.text = str(cert.serial_number)

    return x509_data

def parse_example_data(cert_file):
    with open(cert_file, 'rb') as cert_file:
        full_chain_data = cert_file.read().decode('utf-8')

    certs_data = full_chain_data.strip().split('-----END CERTIFICATE-----')
    certs_data = [cert_data for cert_data in certs_data if cert_data]
    certs_data = [f"{cert_data}-----END CERTIFICATE-----" for cert_data in certs_data]

    nsmap = {'ds': 'http://www.w3.org/2000/09/xmldsig#'}
    new_key_info = lxml_etree.Element('{http://www.w3.org/2000/09/xmldsig#}KeyInfo', nsmap=nsmap)

    for i, cert_data in enumerate(certs_data):
        cert = load_pem_x509_certificate(cert_data.encode('utf-8'), default_backend())
        if i == 0:
            key_value_element = lxml_etree.SubElement(new_key_info, '{http://www.w3.org/2000/09/xmldsig#}KeyValue')
            rsa_key_value = lxml_etree.SubElement(key_value_element, '{http://www.w3.org/2000/09/xmldsig#}RSAKeyValue')
            rsa_key_value_elements = create_rsa_key_value_elements(cert)
            for element_name, element_text in rsa_key_value_elements:
                element = lxml_etree.SubElement(rsa_key_value, element_name)
                element.text = element_text

        create_x509_data(cert, new_key_info)

    examples = {
        'KeyInfo': lxml_etree.tostring(new_key_info, pretty_print=True).decode('utf-8'),
        'KeyValue': lxml_etree.tostring(new_key_info.find('{http://www.w3.org/2000/09/xmldsig#}KeyValue'), pretty_print=True).decode('utf-8'),
        'RSAKeyValue': lxml_etree.tostring(new_key_info.find('.//{http://www.w3.org/2000/09/xmldsig#}RSAKeyValue'), pretty_print=True).decode('utf-8'),
        'X509Data': lxml_etree.tostring(new_key_info.find('{http://www.w3.org/2000/09/xmldsig#}X509Data'), pretty_print=True).decode('utf-8'),
        'X509IssuerSerial': lxml_etree.tostring(new_key_info.find('.//{http://www.w3.org/2000/09/xmldsig#}X509IssuerSerial'), pretty_print=True).decode('utf-8'),
        'X509IssuerName': lxml_etree.tostring(new_key_info.find('.//{http://www.w3.org/2000/09/xmldsig#}X509IssuerName'), pretty_print=True).decode('utf-8'),
        'X509SerialNumber': lxml_etree.tostring(new_key_info.find('.//{http://www.w3.org/2000/09/xmldsig#}X509SerialNumber'), pretty_print=True).decode('utf-8'),
        'X509SubjectName': lxml_etree.tostring(new_key_info.find('.//{http://www.w3.org/2000/09/xmldsig#}X509SubjectName'), pretty_print=True).decode('utf-8'),
        'X509Certificate': lxml_etree.tostring(new_key_info.find('.//{http://www.w3.org/2000/09/xmldsig#}X509Certificate'), pretty_print=True).decode('utf-8'),
        'Modulus': lxml_etree.tostring(new_key_info.find('.//{http://www.w3.org/2000/09/xmldsig#}Modulus'), pretty_print=True).decode('utf-8'),
        'Exponent': lxml_etree.tostring(new_key_info.find('.//{http://www.w3.org/2000/09/xmldsig#}Exponent'), pretty_print=True).decode('utf-8')
    }
    return examples

def create_index_file(output_dir):
    index_file_path = os.path.join(output_dir, 'index.md')
    with open(index_file_path, 'w') as file:
        file.write("# Index\n\n")
        file.write("This is the index page.\n")

def create_element_docs(elements, output_dir, examples):
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    create_index_file(output_dir)

    def write_element_docs(element_name, details, parent_dir):
        file_path = os.path.join(parent_dir, f"{element_name}.md")
        with open(file_path, 'w') as file:
            file.write(f"# {element_name}\n\n")
            file.write("## Attributes\n\n")
            for attribute, description in details.get('attributes', {}).items():
                file.write(f"- **{attribute}**: {description}\n")
            file.write("\n## Sub-elements\n\n")
            for sub_element, description in details.get('sub_elements', {}).items():
                if sub_element in examples:
                    file.write(f"- **[{sub_element}]({sub_element}.md)**: {description}\n")
            file.write("\n## Examples\n\n")
            file.write(f"```xml\n{examples.get(element_name, f'<{element_name}>Example content for {element_name}</{element_name}>')}\n```\n")

    for element_name in examples.keys():
        if element_name in elements:
            write_element_docs(element_name, elements[element_name], output_dir)

if __name__ == "__main__":
    elements = {
        "KeyInfo": {
            "sub_elements": {
                "KeyValue": "Contains the actual key value.",
                "X509Data": "Contains X.509 certificate information."
            },
            "attributes": {}
        },
        "KeyValue": {
            "sub_elements": {
                "RSAKeyValue": "Contains RSA key value."
            },
            "attributes": {}
        },
        "RSAKeyValue": {
            "sub_elements": {
                "Modulus": "The modulus of the RSA key.",
                "Exponent": "The exponent of the RSA key."
            },
            "attributes": {}
        },
        "X509Data": {
            "sub_elements": {
                "X509IssuerSerial": "Contains issuer and serial number.",
                "X509SubjectName": "Contains the subject name.",
                "X509Certificate": "Contains the X.509 certificate."
            },
            "attributes": {}
        },
        "X509IssuerSerial": {
            "sub_elements": {
                "X509IssuerName": "The name of the issuer.",
                "X509SerialNumber": "The serial number of the certificate."
            },
            "attributes": {}
        },
        "Modulus": {
            "sub_elements": {},
            "attributes": {}
        },
        "Exponent": {
            "sub_elements": {},
            "attributes": {}
        },
        "X509IssuerName": {
            "sub_elements": {},
            "attributes": {}
        },
        "X509SerialNumber": {
            "sub_elements": {},
            "attributes": {}
        },
        "X509SubjectName": {
            "sub_elements": {},
            "attributes": {}
        },
        "X509Certificate": {
            "sub_elements": {},
            "attributes": {}
        }
    }
    output_dir = './/markdown/xmldsig'
    examples = parse_example_data(f'{TESTS_DIR}/resources/certs/leaf/certs/Leaf Certificate 1.pem')
    create_element_docs(elements, output_dir, examples)