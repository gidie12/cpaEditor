# Certificate

## Attributes
- `certId`: A unique identifier for the certificate.

## Sub-elements
- `ds:KeyInfo`

## In the CPA Editor
- Shown on the **Certificates** tab: per partner each `certId` with its `KeyInfo`.
- Right-click a **KeyInfo** row for the actions below.
- **Upload Certificate** replaces the `KeyInfo` with the certificates of a PEM file (`.cer`, `.crt` or `.pem`). The file may contain the whole chain, the leaf certificate first. The certificate must have an RSA key.
- **Download Certificate** saves the certificates of the `KeyInfo` as a PEM file, in the order of the CPA (leaf first). The default file name is the `certId`. The file can be uploaded again with **Upload Certificate**.
- **Copy KeyInfo** copies the `KeyInfo` XML to the clipboard.
- The expiry dates of these certificates are used by **Set from certificates** on the **General** tab, see [End](End.md).
