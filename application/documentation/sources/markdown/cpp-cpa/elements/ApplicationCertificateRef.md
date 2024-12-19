# Application Certificate Reference

## Description
The `ApplicationCertificateRef` element is used to reference an application certificate.

## Attributes
- `certificateId`: The unique identifier for the certificate.
- `issuer`: The issuer of the certificate.
- `validFrom`: The start date of the certificate's validity.
- `validTo`: The end date of the certificate's validity.

## Example
```xml
<ApplicationCertificateRef certificateId="12345" issuer="CA" validFrom="2021-01-01" validTo="2023-01-01" />