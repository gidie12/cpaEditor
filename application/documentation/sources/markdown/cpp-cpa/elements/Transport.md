# Transport

## Attributes
- `transportId`: A unique identifier for the transport.

## Sub-elements
- [TransportSender](TransportSender.md) (optional)
- [TransportReceiver](TransportReceiver.md) (optional)

## Example
```xml
<tns:Transport tns:transportId="transportA1">
    <tns:TransportSender>
        <tns:TransportProtocol tns:version="1.1">HTTP</tns:TransportProtocol>
        <tns:AccessAuthentication>basic</tns:AccessAuthentication>
        <tns:TransportClientSecurity>
            <tns:TransportSecurityProtocol tns:version="3.0">SSL</tns:TransportSecurityProtocol>
            <tns:ClientCertificateRef tns:certId="CompanyA_ClientCert" />
            <tns:ServerSecurityDetailsRef tns:securityId="CompanyA_TransportSecurity" />
        </tns:TransportClientSecurity>
    </tns:TransportSender>
    <tns:TransportReceiver>
        <tns:TransportProtocol tns:version="1.1">HTTP</tns:TransportProtocol>
        <tns:AccessAuthentication>basic</tns:AccessAuthentication>
        <tns:Endpoint tns:uri="https://www.CompanyA.com/servlets/ebxmlhandler/async" tns:type="allPurpose" />
        <tns:TransportServerSecurity>
            <tns:TransportSecurityProtocol tns:version="3.0">SSL</tns:TransportSecurityProtocol>
            <tns:ServerCertificateRef tns:certId="CompanyA_ServerCert" />
            <tns:ClientSecurityDetailsRef tns:securityId="CompanyA_TransportSecurity" />
        </tns:TransportServerSecurity>
    </tns:TransportReceiver>
</tns:Transport>
```