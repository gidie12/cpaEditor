# Certificate

## Attributes
- `certId`: A unique identifier for the certificate.

## Sub-elements
- `ds:KeyInfo`

## In the CPA Editor
- Shown on the **Certificates** tab: per partner each `certId` with its `KeyInfo`.
- Open a **CertId** row for a **Certificate** row per certificate of the `KeyInfo`, leaf first, with its type (leaf, intermediate or root), common name and expiry date. Open that row for the subject, issuer, serial number (hexadecimal), **Valid from** and **Valid until** (Zulu time).
- An expired certificate is marked with `(EXPIRED)`. The details are updated after **Upload Certificate**.
- A certificate that no element refers to is shown in grey; the other certificates are in use. References are `ClientCertificateRef`, `ServerCertificateRef`, `SigningCertificateRef`, `EncryptionCertificateRef`, [ApplicationCertificateRef](ApplicationCertificateRef.md) and [AnchorCertificateRef](AnchorCertificateRef.md).
- Right-click a **KeyInfo** row for the actions below.
- **Upload Certificate** replaces the `KeyInfo` with the certificates of a PEM file (`.cer`, `.crt` or `.pem`). The file may contain the whole chain, the leaf certificate first. The certificate must have an RSA key.
- **Download Certificate** saves the certificates of the `KeyInfo` as a PEM file, in the order of the CPA (leaf first). The default file name is the `certId`. The file can be uploaded again with **Upload Certificate**.
- **Copy KeyInfo** copies the `KeyInfo` XML to the clipboard.
- The expiry dates of these certificates are used by **Set from certificates** on the **General** tab, see [End](End.md).
