# X509Data

## Attributes


## Sub-elements

- **[X509IssuerSerial](X509IssuerSerial.md)**: Contains issuer and serial number.
- **[X509SubjectName](X509SubjectName.md)**: Contains the subject name.
- **[X509Certificate](X509Certificate.md)**: Contains the X.509 certificate.

## Examples

```xml
<ds:X509Data xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
  <ds:X509SubjectName>CN=Leaf Certificate 1</ds:X509SubjectName>
  <ds:X509Certificate>MIIC4DCCAcigAwIBAgIUGlfPOO6c1Ka9Udii02FUljZX3yMwDQYJKoZIhvcNAQELBQAwJTEjMCEGA1UEAwwaSW50ZXJtZWRpYXRlIENlcnRpZmljYXRlIDEwHhcNMjQwNDI1MTM1MzUxWhcNMjYwNDI2MTM1MzUxWjAdMRswGQYDVQQDDBJMZWFmIENlcnRpZmljYXRlIDEwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQCNVlUO2wP31/p/mLH0kZIPJgrq7sLQmO6g49SxJzfmrEaTEvADJZZUlcHmLpaHn55j5keITNt0Mvag9cpf8AE70XI7A7hPHMdgGFlbrZA3NBbVPZY6It/FZDMq08KZ/K9kndlEGmfMwoIDCZrfLJBTzPNAghUEmVbnyWwOCTt0GfeTv5uStyUOnnAsFyejSbYfrCWK1ftQK1X/5Qv02f1QJIlxKK0qk6kDwRJImBZFEM7j2+ZvzIvGyeLmKR3xCVP8lFrs40yPEE2AP5I4WoM4i8wqiT2dAahVZFgDspybXOG0YTfnWim0g9Go5z3CazzYbWDGnE73bm9C9pGEy1tJAgMBAAGjEDAOMAwGA1UdEwEB/wQCMAAwDQYJKoZIhvcNAQELBQADggEBAAXiYTvHc8ohJ+qw4z5dNoapZQWnDZr4r3I7ZCjwvap0AygT8dGO4XWDKCx1JkDs1jFYgMiroHR3gRGmHqXb7TjKbiSd7gNWlirM8kemqLnenyR77lgBKLHjkV9D1dlwMeXWxHUsGMSdvUgjTOX1Q17RpDOSDbyFFrjhTT3lVva8HG/Mmi46HVwIqGYMabFfvqBOVb99htPK9Xv3X4409WY19qibrgg2dXlhUDGCm1a+8DdQC3R7OiQy/2A7hd9kA4txHcqCAg8gIcAfkbZHOZnqpznNe57/Kgrq7tF6kjwIpa/UBeC+B7/K4qhctL+hrfYRyRMPpD4owOlopLQ4MTE=</ds:X509Certificate>
  <ds:X509IssuerSerial>
    <ds:X509IssuerName>CN=Intermediate Certificate 1</ds:X509IssuerName>
    <ds:X509SerialNumber>150391976489862988175029128674197088349881622307</ds:X509SerialNumber>
  </ds:X509IssuerSerial>
</ds:X509Data>

```
