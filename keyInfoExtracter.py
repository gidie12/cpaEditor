import tkinter as tk
from tkinter import filedialog
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives.serialization.pkcs7 import load_der_pkcs7_certificates, load_pem_pkcs7_certificates
from cryptography.x509 import load_pem_x509_certificate
from lxml import etree
from cryptography.x509.extensions import KeyUsage, ExtensionNotFound, serialization
import base64

def add_spaces(s, n):
    return ' '.join(s[i:i+n] for i in range(0, len(s), n))
class KeyInfoExtractor:
    """
    A class used to extract key information from a certificate.

    ...

    Attributes
    ----------
    cert_data : str
        a string representing the certificate data

    Methods
    -------
    create_rsa_key_value_elements(cert)
        Returns a list of tuples representing the RSA key value elements.
    create_x509_data(cert, key_info)
        Creates the X509 data and returns it.
    create_sub_elements(parent, elements)
        Creates sub elements for a given parent element.
    create_key_info(cert)
        Creates the key info and returns it.
    minimum_bytes_needed(n)
        Returns the minimum number of bytes needed to represent a number.
    open_certificate()
        Opens a certificate file and extracts the key info.
    delete_item()
        Deletes the current item from the text field.
    export_keyinfo()
        Exports the key info to a file.
    format_certificate(certificate_data)
        Formats the certificate data and returns it.
    extract_certificate(key_info)
        Extracts the certificate from the key info and returns it.
    create_gui()
        Creates the GUI for the application.
    """

    def __init__(self, cert_data):
        """
        Constructs all the necessary attributes for the KeyInfoExtractor object.

        Parameters
        ----------
            cert_data : str
                a string representing the certificate data
        """
        self.cert_data = cert_data
        self.create_gui()

    def create_rsa_key_value_elements(self, cert):
        """
        Returns a list of tuples representing the RSA key value elements.

        Parameters
        ----------
            cert : Certificate
                the certificate object
        """
        public_numbers = cert.public_key().public_numbers()
        namespace_element_modulus = '{http://www.w3.org/2000/09/xmldsig#}' + 'Modulus'
        namespace_element_exponent = '{http://www.w3.org/2000/09/xmldsig#}' + 'Exponent'
        rsa_key_value_elements = [
            (namespace_element_modulus, public_numbers.n),
            (namespace_element_exponent, public_numbers.e)
        ]
        return rsa_key_value_elements

    def create_x509_data(self, cert, key_info):
        """
        Creates the X509 data and returns it.

        Parameters
        ----------
            cert : Certificate
                the certificate object
            key_info : Element
                the key info element
        """

        x509_data = etree.SubElement(key_info, '{http://www.w3.org/2000/09/xmldsig#}X509Data')

        issuer_serial = etree.SubElement(x509_data, '{http://www.w3.org/2000/09/xmldsig#}X509IssuerSerial')
        etree.SubElement(issuer_serial,
                         '{http://www.w3.org/2000/09/xmldsig#}X509IssuerName').text = cert.issuer.rfc4514_string()
        etree.SubElement(issuer_serial, '{http://www.w3.org/2000/09/xmldsig#}X509SerialNumber').text = str(
            cert.serial_number)

        data_elements = [
            ('{http://www.w3.org/2000/09/xmldsig#}X509SubjectName', cert.subject.rfc4514_string()),
            ('{http://www.w3.org/2000/09/xmldsig#}X509Certificate',
             base64.b64encode(cert.public_bytes(encoding=serialization.Encoding.DER)).decode('utf-8')),
        ]

        for element_name, element_text in data_elements:
            etree.SubElement(x509_data, element_name).text = element_text

        return x509_data

    def create_sub_elements(self, parent, elements):
        """
        Creates sub elements for a given parent element.

        Parameters
        ----------
            parent : Element
                the parent element
            elements : list
                a list of tuples representing the elements
        """
        for element_name, element_text in elements:
            element = etree.SubElement(parent, element_name)
            element.text = str(element_text)

    def minimum_bytes_needed(self,n):
        """
        Returns the minimum number of bytes needed to represent a number.

        Parameters
        ----------
            n : int
                the number
        """
        if n == 0:
            return 1
        return (n.bit_length() + 7) // 8

    def get_modulus_in_base64(self, cert):
        modulus = cert.public_key().public_numbers().n
        modulus_bytes = modulus.to_bytes((modulus.bit_length() + 7) // 8, 'big')
        modulus_base64 = base64.b64encode(modulus_bytes).decode('utf-8')
        modulus_base64 = add_spaces(modulus_base64, 76)
        return modulus_base64
    def open_certificate(self):
        """
        Opens a certificate file and extracts the key info.
        """
        # support also .p7b
        file_path = filedialog.askopenfilename(filetypes=[("Certificate files", "*.cer *.crt *.pem, *.p7b")])
        if not file_path:
            return

        with open(file_path, 'rb') as cert_file:
            cert_data = cert_file.read()
            if file_path.endswith('.p7b'):
                try:
                    certs = load_der_pkcs7_certificates(cert_data)
                except ValueError:
                    certs = load_pem_pkcs7_certificates(cert_data)
                self.certs_data = [cert.public_bytes(serialization.Encoding.PEM).decode('utf-8') for cert in certs]
            else:
                self.certs_data = cert_data.decode('utf-8').strip().split('-----END CERTIFICATE-----')
                self.certs_data = [f"{cert_data}-----END CERTIFICATE-----" for cert_data in self.certs_data if cert_data]

        key_info = etree.Element('{http://www.w3.org/2000/09/xmldsig#}KeyInfo', nsmap={'ds': 'http://www.w3.org/2000/09/xmldsig#'})

        for i, cert_data in enumerate(self.certs_data):
            cert = load_pem_x509_certificate(cert_data.encode('utf-8'), default_backend())
            if i == 0:
                rsa_key_value = etree.SubElement(etree.SubElement(key_info, '{http://www.w3.org/2000/09/xmldsig#}KeyValue'), '{http://www.w3.org/2000/09/xmldsig#}RSAKeyValue')
                expononent = cert.public_key().public_numbers().e
                expononent_bytes = expononent.to_bytes(self.minimum_bytes_needed(expononent), byteorder='big')
                expononent_base64 = base64.b64encode(expononent_bytes).decode('utf-8')

                rsa_key_value_elements = [
                    ('{http://www.w3.org/2000/09/xmldsig#}Modulus', f" {self.get_modulus_in_base64(cert)} "),
                    # show exponent as AQAB from 65537
                    ('{http://www.w3.org/2000/09/xmldsig#}Exponent', expononent_base64)

                    # ('{http://www.w3.org/2000/09/xmldsig#}Exponent', cert.public_key().public_numbers().e)
                ]
                for element_name, element_text in rsa_key_value_elements:
                    etree.SubElement(rsa_key_value, element_name).text = str(element_text)
            self.create_x509_data(cert, key_info)

        pretty_printed_key_info = etree.tostring(key_info, pretty_print=True).decode('utf-8')
        self.text_field.insert(tk.END, pretty_printed_key_info)
        self.delete_button.config(state=tk.NORMAL)

    def delete_item(self):
        """
        Deletes the current item from the text field.
        """
        self.text_field.delete(1.0, tk.END)
        self.delete_button.config(state=tk.DISABLED)

    def export_keyinfo(self):
        """
        Exports the key info to a file.
        """
        output_dir = filedialog.askdirectory(title="Select Output Directory")
        if output_dir:
            with open(f"{output_dir}/keyinfo.xml", "w") as keyinfo_file:
                keyinfo_file.write(self.text_field.get(1.0, tk.END))

    def format_certificate(self,certificate_data):
        """
        Formats the certificate data and returns it.

        Parameters
        ----------
            certificate_data : str
                the certificate data
        """
        cleaned_data = certificate_data.replace("-----BEGIN PUBLIC KEY-----", "")
        cleaned_data = cleaned_data.replace("-----END PUBLIC KEY-----", "")
        return f"-----BEGIN CERTIFICATE-----\n{cleaned_data}\n-----END CERTIFICATE-----"

    def extract_certificate(self,key_info):
        """
        Extracts the certificate from the key info and returns it.

        Parameters
        ----------
            key_info : str
                the key info
        """
        tree = etree.fromstring(key_info.encode('utf-8'))
        x509_certificate = tree.find("./X509Data/X509Certificate")
        if x509_certificate is not None:
            return base64.b64decode(x509_certificate.text.strip()).decode('utf-8')
        return None

    def create_gui(self):
        """
        Creates the GUI for the application.
        """
        self.root = tk.Tk()
        self.root.geometry("1920x1080")
        self.root.title("CER/CRT to KeyInfo Converter")

        tk.Label(self.root, text="Party Name Partner A:").grid(sticky="w", row=1, column=1, padx=5, pady=5)
        tk.Button(self.root, text="Upload Certificate", command=self.open_certificate).grid(sticky="w", row=1, column=1, padx=5, pady=5)
        self.delete_button = tk.Button(self.root, text="Delete", command=self.delete_item, state=tk.DISABLED)
        self.delete_button.grid(sticky="w", row=1, column=2, padx=5, pady=5)

        self.text_field = tk.Text(self.root, height=60, width=200)
        self.text_field.grid(sticky="w", row=2, column=1, padx=10, pady=5)

        self.export_button = tk.Button(self.root, text="Export Keyinfo", command=self.export_keyinfo).grid(pady=5)

        self.root.mainloop()

if __name__ == "__main__":
    KeyInfoExtractor(None)