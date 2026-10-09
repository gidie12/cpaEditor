import base64
import re
import xml.etree.ElementTree as Et
import tkinter as tk
from functools import partial
from tkinter import ttk, filedialog
from cryptography.hazmat.primitives import serialization
from lxml import etree as lxml_etree
from cryptography.hazmat.backends import default_backend
from cryptography.x509 import load_der_x509_certificate, load_pem_x509_certificate, ExtensionNotFound, KeyUsage
from application.helper_classes.cpaParser import CPAParser


def create_rsa_key_value_elements(cert):
    """
    Creates RSA key value elements from a certificate.

    Args:
        cert (x509.Certificate): The certificate from which to extract the RSA key values.

    Returns:
        list: A list of tuples containing the element names and their corresponding base64-encoded values.

    Test Functions:
        - test_create_rsa_key_value_elements
    """

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
    """
    Creates the X509Data element for a given certificate and adds it to the KeyInfo element.

    Args:
        cert (x509.Certificate): The certificate from which to extract the X509 data.
        key_info (lxml.etree.Element): The KeyInfo element to which the X509Data will be added.

    Returns:
        lxml.etree.Element: The created X509Data element.

    Test Functions:
        - test_create_x509_data_creates_correct_elements
        - test_create_x509_data_adds_elements_to_key_info
    """

    x509_data = lxml_etree.SubElement(key_info, '{http://www.w3.org/2000/09/xmldsig#}X509Data')

    # X509DataType is a sequence in xmldsig.xsd: X509IssuerSerial must come before X509SubjectName and X509Certificate
    X509IssuerSerial = lxml_etree.SubElement(x509_data, '{http://www.w3.org/2000/09/xmldsig#}X509IssuerSerial')
    X509IssuerName = lxml_etree.SubElement(X509IssuerSerial, '{http://www.w3.org/2000/09/xmldsig#}X509IssuerName')
    X509IssuerName.text = cert.issuer.rfc4514_string()

    X509SerialNumber = lxml_etree.SubElement(X509IssuerSerial, '{http://www.w3.org/2000/09/xmldsig#}X509SerialNumber')
    X509SerialNumber.text = str(cert.serial_number)

    data_elements = [
        ('{http://www.w3.org/2000/09/xmldsig#}X509SubjectName', cert.subject.rfc4514_string()),
        ('{http://www.w3.org/2000/09/xmldsig#}X509Certificate', base64.b64encode(cert.public_bytes(encoding=serialization.Encoding.DER)).decode('utf-8'))
    ]

    for element_name, element_text in data_elements:
        element = lxml_etree.SubElement(x509_data, element_name)
        element.text = element_text

    return x509_data


def key_info_to_pem(key_info):
    """
    Converts the certificates of a KeyInfo element to PEM.

    Args:
        key_info (lxml.etree.Element): The KeyInfo element with one X509Certificate per certificate in the chain.

    Returns:
        str: The certificates in PEM format in the order of the KeyInfo (leaf first), empty without certificates.

    Test Functions:
        - test_key_info_to_pem_returns_chain_in_order
        - test_key_info_to_pem_without_certificates
    """
    pem = ''
    if key_info is None:
        return pem
    for x509_certificate in key_info.iter('{http://www.w3.org/2000/09/xmldsig#}X509Certificate'):
        cert = load_der_x509_certificate(base64.b64decode(x509_certificate.text))
        pem += cert.public_bytes(serialization.Encoding.PEM).decode('utf-8')
    return pem


def create_sub_elements(parent, elements):
    """
       Creates sub-elements for a given parent element.

       Args:
           parent (lxml.etree.Element): The parent element to which sub-elements will be added.
           elements (list): A list of tuples containing the element names and their corresponding text values.

       Returns:
           None

       Test Functions:
           - test_create_sub_elements_creates_correct_elements
           - test_create_sub_elements_adds_elements_to_parent
       """
    for element_name, element_text in elements:
        element = lxml_etree.SubElement(parent, element_name)
        element.text = str(element_text)


def replace_xml_object(parent_xml_element, xml_element, new_element):
    """
    Replaces an existing XML element with a new element or appends a new element if the existing element is None.

    Args:
        parent_xml_element (xml.etree.ElementTree.Element): The parent XML element that contains the element to be replaced or appended to.
        xml_element (xml.etree.ElementTree.Element or None): The existing XML element to be replaced. If None, the new element will be appended to the parent.
        new_element (xml.etree.ElementTree.Element or None): The new XML element to replace the existing element or to be appended.

    Returns:
        None

    Test Functions:
        - test_replace_xml_object_appends_new_element_when_xml_element_is_none
        - test_replace_xml_object_replaces_existing_element_with_new_element
        - test_replace_xml_object_does_nothing_when_new_element_is_none
        - test_replace_xml_object_does_nothing_when_parent_xml_element_is_none
    """

    if xml_element is None and parent_xml_element is not None and new_element is not None:
        parent_xml_element.append(new_element)

    elif xml_element and parent_xml_element and new_element:
        parent_xml_element.remove(xml_element)
        parent_xml_element.append(new_element)


class Certificates(tk.Frame):
    def __init__(self, master, logger, **kw):
        super().__init__(master, **kw)
        self.tree_editor = None
        self.entry_fields = []
        self.master = master
        self.xml_element_mapping = {}
        self.logger = logger

    def load(self):
        """
          Loads and processes certificate information from the CPA (Collaboration Protocol Agreement).

          This method performs the following steps:
          1. Retrieves the party names for Partner A and Partner B.
          2. Parses the certificate information for both parties.
          3. Generates a dictionary mapping certificate IDs to their KeyInfo and certificate elements.
          4. Populates the Treeview widget with the certificate and key info data.

          Returns:
              None

          Test Functions:
              - test_load_populates_tree_with_certificates
              - test_load_logs_error_when_no_certificates_found
          """
        cpa_parser_functions = CPAParser(self.master.root)
        partner_a = cpa_parser_functions.get_party_name_partner_a()
        partner_b = cpa_parser_functions.get_party_name_partner_b()
        cpa_parser_functions.certificate_info()
        certificate_party_a_elements = cpa_parser_functions.get_certificate_party_a_elements()
        certificate_party_b_elements = cpa_parser_functions.get_certificate_party_b_elements()
        self.xml_element_mapping = {}

        namespace_certificate_id = '{' + self.master.namespace_uri + '}' + 'certId'

        # Always drop the certificates of the previously loaded CPA, also when the new CPA has none
        self.clear_tree()

        if not certificate_party_a_elements and not certificate_party_b_elements:
            self.logger.error("No certificates found in the CPA")
            return

        for partner, certificate_party_elements in ((partner_a, certificate_party_a_elements),
                                                    (partner_b, certificate_party_b_elements)):
            if certificate_party_elements:
                partner_keyinfo_certid_list = self.generate_partner_keyinfo_certid_list(certificate_party_elements, namespace_certificate_id)
                self.populate_tree(partner, partner_keyinfo_certid_list, clear_tree=False)

    def generate_partner_keyinfo_certid_list(self, certificate_party_elements, namespace_certificate_id):
        """
            Generates a dictionary mapping certificate IDs to their KeyInfo and certificate elements.

            Args:
                certificate_party_elements (list): A list of certificate elements.
                namespace_certificate_id (str): The namespace URI for the certificate ID attribute.

            Returns:
                dict: A dictionary mapping certificate IDs to a list containing the KeyInfo XML string,
                      the KeyInfo element, and the certificate element.

            Test Functions:
                - test_generate_partner_keyinfo_certid_list_with_valid_data
                - test_generate_partner_keyinfo_certid_list_with_missing_cert_id
                - test_generate_partner_keyinfo_certid_list_with_no_key_info
            """
        partner_keyinfo_certid_list = {}
        for cert_item in certificate_party_elements:
            try:
                if cert_item.attrib[f'{namespace_certificate_id}']:
                    cert_id = cert_item.attrib[f'{namespace_certificate_id}']
                    # add xmlns:ds to the KeyInfo element
                    namespace_key_info = '{http://www.w3.org/2000/09/xmldsig#}' + 'KeyInfo'
                    if cert_item.findall(namespace_key_info) is not None:
                        try:
                            key_info = cert_item.findall(namespace_key_info)[0]
                            Et.register_namespace('ds', 'http://www.w3.org/2000/09/xmldsig#')
                            pretty_printed_key_info = Et.tostring(key_info).decode('utf-8')
                            partner_keyinfo_certid_list[f'{cert_id}'] = [pretty_printed_key_info, key_info, cert_item]
                        except IndexError:
                            self.logger.error(f"KeyInfo not found for certificate ID: {cert_id}")
                            partner_keyinfo_certid_list[f'{cert_id}'] = ['', None, cert_item]
            except Exception:
                self.logger.error("Certificate ID attribute not found for certificate element in CPA please check the CPA")
                continue
        return partner_keyinfo_certid_list

    def reload(self):
        """
        Reloads and processes certificate information from the CPA (Collaboration Protocol Agreement).

        This method performs the following steps:
        1. Clears the current Treeview widget.
        2. Calls the `load` method to reload and process the certificate information.

        Returns:
            None

        Test Functions:
            - test_reload_clears_tree_and_loads_certificates
        """
        self.load()

    def show_menu(self, event):
        if self.tree_editor.selection():
            selected_item = self.tree_editor.selection()[0]
            text_len = len(self.tree_editor.item(selected_item, 'text'))
            text = ''

            if text_len > 0:
                text = self.tree_editor.item(selected_item, 'text')

            if (selected_item and text == 'KeyInfo'):
                menu = tk.Menu(self, tearoff=0)
                menu.add_command(label="Copy KeyInfo", command=self.copy_item)
                menu.add_command(label="Upload Certificate", command=self.open_certificate)
                menu.add_command(label="Download Certificate", command=self.save_certificate)
                menu.post(event.x_root, event.y_root)

    def copy_item(self):
        selected_item = self.tree_editor.selection()
        if selected_item:
            # text = self.tree_editor.item(selected_item, 'text')
            text = self.tree_editor.item(selected_item, 'values')[0]
            self.clipboard_clear()
            self.clipboard_append(text)
            self.update()

    def populate_tree(self, partner, data, clear_tree=True):
        # Clear existing tree items
        if clear_tree:
            self.clear_tree()

        # Populate the tree with certificate and key info data
        partner = self.tree_editor.insert("", "end", text=f"Partner {partner}", values=(), open=True)
        for cert_id, items in data.items():
            cert_id_item = self.tree_editor.insert(partner, "end", text="CertId", values=(cert_id))
            key_info_item = self.tree_editor.insert(cert_id_item, "end", text="KeyInfo", values=(items[0],))
            self.xml_element_mapping[key_info_item] = [items[1], items[2]]  # Map Treeview item ID to XML element


            
    def clear_tree(self):
        for item in self.tree_editor.get_children():
            self.tree_editor.delete(item)

    def create(self):
        """
        Creates and configures a Treeview widget for displaying certificate and key info.

        This method performs the following steps:
        1. Creates a Treeview widget with columns for Partner/Certificate ID and Value/KeyInfo.
        2. Configures the style for the Treeview headings and attribute items.
        3. Binds the Treeview widget to a right-click menu for copying KeyInfo and uploading certificates.

        Returns:
            None

        Test Functions:
            - test_create_treeview_widget_created
            - test_create_treeview_widget_configured
            - test_create_treeview_widget_bound_to_menu
        """

        # Create a Treeview widget for displaying the certificate and key info
        self.tree_editor = ttk.Treeview(self, columns=("Value"), selectmode="browse")
        self.tree_editor.heading("#0", text="Partner/Certificate ID")
        self.tree_editor.heading("Value", text="Value/KeyInfo")
        self.tree_editor.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)

        # Define style for attribute tags
        style = ttk.Style()
        style.configure("Treeview.Heading", font=("Helvetica", 10, "bold"))
        style.configure("attribute.Treeview.Item", font=("Helvetica", 10, "normal"))

        # Create a tag for attribute items
        self.tree_editor.tag_configure("attribute", font=("Helvetica", 10, "normal"))

        # Bind the Treeview widget to the right-click menu
        if self.master.host == 'mac':
            self.tree_editor.bind("<Button-2>", self.show_menu)
        else:
            self.tree_editor.bind("<Button-3>", self.show_menu)

    def save_certificate(self):
        """
        Saves the certificate chain of the selected KeyInfo as a PEM file.

        The file holds every certificate of the KeyInfo, leaf first, and can be uploaded again with open_certificate.

        Returns:
            None

        Test Functions:
            - test_save_certificate_writes_chain
            - test_save_certificate_no_file_selected
            - test_save_certificate_without_certificate
        """
        try:
            key_info_element, cert_element = self.xml_element_mapping.get(self.tree_editor.selection()[0])
            pem = key_info_to_pem(key_info_element)
            if not pem:
                self.logger.error("No certificate found in this KeyInfo")
                return

            cert_id = cert_element.get('{' + self.master.namespace_uri + '}certId') or 'certificate'
            file_path = filedialog.asksaveasfilename(defaultextension=".pem",
                                                     initialfile=re.sub(r'[^\w.-]', '_', cert_id) + ".pem",
                                                     filetypes=[("Certificate files", "*.pem *.crt *.cer")])
            if not file_path:
                return

            with open(file_path, 'w', encoding='utf-8') as cert_file:
                cert_file.write(pem)
            self.logger.info(f"{pem.count('BEGIN CERTIFICATE')} certificate(s) of certId {cert_id} saved to {file_path}")
        except Exception as e:
            self.logger.error(f"Error while downloading certificate: {e}")

    def open_certificate(self):
        """
        Opens a file dialog to select a certificate file, processes the certificate chain, and updates the KeyInfo element.

        This method performs the following steps:
        1. Opens a file dialog to select a certificate file.
        2. Reads the selected certificate file and splits it into individual certificates.
        3. Creates a KeyInfo element and processes each certificate in the chain.
        4. For the leaf certificate, creates the RSAKeyValue element.
        5. Creates the X509Data element for each certificate and adds it to the KeyInfo.
        6. Updates the XML tree with the new KeyInfo element.

        Returns:
            None

        Test Functions:
            - test_open_certificate_no_file_selected
            - test_open_certificate_invalid_certificate
            - test_open_certificate_valid_file_updates_tree
        """
        try:

            file_path = filedialog.askopenfilename(filetypes=[("Certificate files", "*.cer *.crt *.pem")])
            if not file_path:
                return

            with open(file_path, 'rb') as cert_file:
                full_chain_data = cert_file.read().decode('utf-8')

            # Split the full chain data into individual certificates
            certs_data = full_chain_data.strip().split('-----END CERTIFICATE-----')

            # Remove any empty strings from the list
            certs_data = [cert_data for cert_data in certs_data if cert_data]
            nsmap = {'ds': 'http://www.w3.org/2000/09/xmldsig#'}
            # Add back the '-----END CERTIFICATE-----' to each certificate data
            certs_data = [f"{cert_data}-----END CERTIFICATE-----" for cert_data in certs_data]

            # Create the KeyInfo element
            new_key_info = lxml_etree.Element('{http://www.w3.org/2000/09/xmldsig#}KeyInfo', nsmap=nsmap)

            # Process each certificate separately
            for i, cert_data in enumerate(certs_data):
                cert = load_pem_x509_certificate(cert_data.encode('utf-8'), default_backend())

                # For the leaf certificate (the first one in the chain), create the rsa_key_value
                if i == 0:
                    key_value_element = lxml_etree.SubElement(new_key_info, '{http://www.w3.org/2000/09/xmldsig#}KeyValue')
                    rsa_key_value = lxml_etree.SubElement(key_value_element, '{http://www.w3.org/2000/09/xmldsig#}RSAKeyValue')
                    rsa_key_value_elements = create_rsa_key_value_elements(cert)
                    create_sub_elements(rsa_key_value, rsa_key_value_elements)

                # Create the X509Data for each certificate and add it to the key_info
                create_x509_data(cert, new_key_info)
            pretty_printed_key_info = lxml_etree.tostring(new_key_info, pretty_print=True).decode('utf-8')

            # Split the pretty printed key info into lines
            pretty_printed_key_info = pretty_printed_key_info.replace('><', '>\n<')

            key_info_element = self.xml_element_mapping.get(self.tree_editor.selection()[0])[0]
            cert_element = self.xml_element_mapping.get(self.tree_editor.selection()[0])[1]

            # if isinstance(new_key_info, lxml_etree._Element):
            #     new_key_info = Et.fromstring(lxml_etree.tostring(new_key_info))

            replace_xml_object(cert_element, key_info_element, new_key_info)
            self.xml_element_mapping[self.tree_editor.selection()[0]] = [new_key_info, cert_element]
            self.tree_editor.item(self.tree_editor.selection()[0], values=(pretty_printed_key_info,))
            self.logger.info(f"Certificate uploaded successfully and transformed into KeyInfo element")
        except Exception as e:
            self.logger.error(f"Error while uploading certificate: {e}")

