# TransportSender

## Sub-elements
- `TransportProtocol`
- [AccessAuthentication](AccessAuthentication.md) (optional, unbounded)
- [TransportClientSecurity](TransportClientSecurity.md) (optional)

## Example
```xml
<tns:TransportSender>
    <tns:TransportProtocol tns:version="1.1">HTTP</tns:TransportProtocol>
    <tns:AccessAuthentication>basic</tns:AccessAuthentication>
    <tns:TransportClientSecurity>
        <tns:TransportSecurityProtocol tns:version="3.0">SSL</tns:TransportSecurityProtocol>
        <tns:ClientCertificateRef tns:certId="CompanyA_ClientCert" />
        <tns:ServerSecurityDetailsRef tns:securityId="CompanyA_TransportSecurity" />
    </tns:TransportClientSecurity>
</tns:TransportSender>
```