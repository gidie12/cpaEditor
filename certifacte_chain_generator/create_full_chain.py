from cryptography import x509
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
from cryptography.x509.oid import NameOID
import datetime
import os

# Define the directories for the root, intermediate, and leaf certificates
root_dir = '../application/tests/resources/certs/root/certs'
intermediate_dir = '../application/tests/resources/certs/intermediate/certs'
leaf_dir = '../application/tests/resources/certs/leaf/certs'

# Create the directories if they do not exist
os.makedirs(root_dir, exist_ok=True)
os.makedirs(intermediate_dir, exist_ok=True)
os.makedirs(leaf_dir, exist_ok=True)

def generate_root_cert():
    # Generate a key pair
    key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
    )

    # Generate a CSR
    builder = x509.CertificateBuilder()
    builder = builder.subject_name(x509.Name([
        x509.NameAttribute(NameOID.COMMON_NAME, u'Root Certificate'),
    ]))
    builder = builder.issuer_name(x509.Name([
        x509.NameAttribute(NameOID.COMMON_NAME, u'Root Certificate'),
    ]))
    builder = builder.not_valid_before(datetime.datetime.today() - datetime.timedelta(days=1))
    builder = builder.not_valid_after(datetime.datetime.today() + datetime.timedelta(days=10*365))
    builder = builder.serial_number(x509.random_serial_number())
    builder = builder.public_key(key.public_key())
    builder = builder.add_extension(
        x509.BasicConstraints(ca=True, path_length=None), critical=True,
    )

    # Sign the CSR with the root certificate
    certificate = builder.sign(
        private_key=key, algorithm=hashes.SHA256(),
    )

    # Save the root certificate and key pair to a file
    with open(os.path.join(root_dir, "root.pem"), "wb") as f:
        f.write(certificate.public_bytes(serialization.Encoding.PEM))

    with open(os.path.join(root_dir, "root_key.pem"), "wb") as f:
        f.write(key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.TraditionalOpenSSL,
            encryption_algorithm=serialization.NoEncryption(),
        ))

    return certificate, key


def generate_intermediate_cert(root_cert, root_key, common_name):
    # Generate a key pair for the intermediate certificate
    key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
    )

    # Generate a CSR for the intermediate certificate
    builder = x509.CertificateBuilder()
    builder = builder.subject_name(x509.Name([
        x509.NameAttribute(NameOID.COMMON_NAME, common_name),
    ]))
    builder = builder.issuer_name(root_cert.subject)
    builder = builder.not_valid_before(datetime.datetime.today() - datetime.timedelta(days=1))
    builder = builder.not_valid_after(datetime.datetime.today() + datetime.timedelta(days=5*365))
    builder = builder.serial_number(x509.random_serial_number())
    builder = builder.public_key(key.public_key())
    builder = builder.add_extension(
        x509.BasicConstraints(ca=False, path_length=None), critical=True,
    )

    # Sign the CSR with the root certificate
    intermediate_cert = builder.sign(
        private_key=root_key, algorithm=hashes.SHA256(),
    )

    # Save the intermediate certificate and key pair to a file
    with open(os.path.join(intermediate_dir, f"{common_name}.pem"), "wb") as f:
        f.write(intermediate_cert.public_bytes(serialization.Encoding.PEM))

    with open(os.path.join(intermediate_dir, f"{common_name}_key.pem"), "wb") as f:
        f.write(key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.TraditionalOpenSSL,
            encryption_algorithm=serialization.NoEncryption(),
        ))

    # Save the full chain to a file
    with open(os.path.join(intermediate_dir, f"{common_name}_full_chain.pem"), "wb") as f:
        f.write(intermediate_cert.public_bytes(serialization.Encoding.PEM))
        f.write(root_cert.public_bytes(serialization.Encoding.PEM))

    return intermediate_cert, key


def generate_leaf_cert(intermediate_cert, intermediate_key, common_name):
    # Generate a key pair for the leaf certificate
    key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
    )

    # Generate a CSR for the leaf certificate
    builder = x509.CertificateBuilder()
    builder = builder.subject_name(x509.Name([
        x509.NameAttribute(NameOID.COMMON_NAME, common_name),
    ]))
    builder = builder.issuer_name(intermediate_cert.subject)
    builder = builder.not_valid_before(datetime.datetime.today() - datetime.timedelta(days=1))
    builder = builder.not_valid_after(datetime.datetime.today() + datetime.timedelta(days=2*365))
    builder = builder.serial_number(x509.random_serial_number())
    builder = builder.public_key(key.public_key())
    builder = builder.add_extension(
        x509.BasicConstraints(ca=False, path_length=None), critical=True,
    )

    # Sign the CSR with the intermediate certificate
    leaf_cert = builder.sign(
        private_key=intermediate_key, algorithm=hashes.SHA256(),
    )

    # Save the leaf certificate and key pair to a file
    with open(os.path.join(leaf_dir, f"{common_name}.pem"), "wb") as f:
        f.write(leaf_cert.public_bytes(serialization.Encoding.PEM))

    with open(os.path.join(leaf_dir, f"{common_name}_key.pem"), "wb") as f:
        f.write(key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.TraditionalOpenSSL,
            encryption_algorithm=serialization.NoEncryption(),
        ))

    # Save the full chain to a file
    with open(os.path.join(leaf_dir, f"{common_name}_full_chain.pem"), "wb") as f:
        f.write(leaf_cert.public_bytes(serialization.Encoding.PEM))
        f.write(intermediate_cert.public_bytes(serialization.Encoding.PEM))
        with open(os.path.join(root_dir, "root.pem"), "rb") as root_cert_file:
            f.write(root_cert_file.read())


# Generate the root certificate
root_cert, root_key = generate_root_cert()

# Generate the intermediate certificate
intermediate_cert, intermediate_key = generate_intermediate_cert(root_cert, root_key, 'Intermediate Certificate 1')

# Generate the leaf certificate
generate_leaf_cert(intermediate_cert, intermediate_key, 'Leaf Certificate 1')
generate_leaf_cert(intermediate_cert, intermediate_key, 'Leaf Certificate 2')