# Anchor Certificate Reference

## Description
The `AnchorCertificateRef` element is used to reference an anchor certificate.

## Attributes
- `certificateId`: The unique identifier for the certificate.
- `issuer`: The issuer of the certificate.
- `validFrom`: The start date of the certificate's validity.
- `validTo`: The end date of the certificate's validity.

## Example
```xml
<AnchorCertificateRef certificateId="54321" issuer="CA" validFrom="2022-01-01" validTo="2024-01-01" />