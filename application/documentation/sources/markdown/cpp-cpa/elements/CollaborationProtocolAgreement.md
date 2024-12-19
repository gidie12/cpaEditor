# CollaborationProtocolAgreement

## Attributes
- `cpaid`: A unique identifier for the CPA.
- `version`: The version of the CPA.

## Sub-elements
- [Status](Status.md)
- [Start](Start.md)
- [End](End.md)
- [ConversationConstraints](ConversationConstraints.md) (optional)
- [PartyInfo](PartyInfo.md) (2 occurrences)
- [SimplePart](SimplePart.md) (unbounded)
- [Packaging](Packaging.md) (unbounded)
- [Signature](Signature.md) (optional)
- [Comment](Comment.md) (optional, unbounded)

## Example
```xml
<?xml version="1.0" encoding="UTF-8" ?>
<tns:CollaborationProtocolAgreement xmlns:tns="http://example.com/cpp-cpa-2_0.xsd" cpaid="CPA12345" version="2.0">
    <tns:Status>agreed</tns:Status>
    <tns:Start>2023-01-01T00:00:00Z</tns:Start>
    <tns:End>2024-01-01T00:00:00Z</tns:End>
    <tns:ConversationConstraints>
        <!-- Optional constraints here -->
    </tns:ConversationConstraints>
    <tns:PartyInfo>
        <tns:PartyId>PartyA</tns:PartyId>
        <tns:PartyRef>ReferenceA</tns:PartyRef>
        <tns:CollaborationRole>
            <tns:ProcessSpecification>ProcessSpecA</tns:ProcessSpecification>
            <tns:Role name="RoleA"/>
            <tns:ApplicationCertificateRef>CertRefA</tns:ApplicationCertificateRef>
            <tns:ServiceBinding>
                <tns:Service>ServiceA</tns:Service>
                <tns:CanSend>
                    <tns:ThisPartyActionBinding>
                        <tns:BusinessTransactionCharacteristics>CharacteristicsA</tns:BusinessTransactionCharacteristics>
                        <tns:ActionContext>ContextA</tns:ActionContext>
                        <tns:ChannelId>ChannelA</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>
                        <!-- Other party action binding details -->
                    </tns:OtherPartyActionBinding>
                </tns:CanSend>
                <tns:CanReceive>
                    <tns:ThisPartyActionBinding>
                        <tns:BusinessTransactionCharacteristics>CharacteristicsB</tns:BusinessTransactionCharacteristics>
                        <tns:ActionContext>ContextB</tns:ActionContext>
                        <tns:ChannelId>ChannelB</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>
                        <!-- Other party action binding details -->
                    </tns:OtherPartyActionBinding>
                </tns:CanReceive>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:Certificate>CertA</tns:Certificate>
        <tns:SecurityDetails>
            <tns:TrustAnchors>
                <tns:AnchorCertificateRef>AnchorCertA</tns:AnchorCertificateRef>
            </tns:TrustAnchors>
        </tns:SecurityDetails>
        <tns:DeliveryChannel>
            <tns:MessagingCharacteristics>CharacteristicsC</tns:MessagingCharacteristics>
        </tns:DeliveryChannel>
        <tns:Transport>
            <tns:TransportSender>SenderA</tns:TransportSender>
            <tns:TransportReceiver>ReceiverA</tns:TransportReceiver>
            <tns:TransportClientSecurity>ClientSecurityA</tns:TransportClientSecurity>
            <tns:TransportServerSecurity>ServerSecurityA</tns:TransportServerSecurity>
        </tns:Transport>
        <tns:DocExchange>
            <tns:ebXMLSenderBinding>
                <tns:ReliableMessaging>ReliableA</tns:ReliableMessaging>
                <tns:PersistDuration>DurationA</tns:PersistDuration>
                <tns:SenderNonRepudiation>NonRepudiationA</tns:SenderNonRepudiation>
                <tns:SenderDigitalEnvelope>EnvelopeA</tns:SenderDigitalEnvelope>
            </tns:ebXMLSenderBinding>
            <tns:ebXMLReceiverBinding>
                <tns:ReliableMessaging>ReliableB</tns:ReliableMessaging>
                <tns:PersistDuration>DurationB</tns:PersistDuration>
                <tns:ReceiverNonRepudiation>NonRepudiationB</tns:ReceiverNonRepudiation>
                <tns:ReceiverDigitalEnvelope>EnvelopeB</tns:ReceiverDigitalEnvelope>
                <tns:NamespaceSupported>NamespaceA</tns:NamespaceSupported>
            </tns:ebXMLReceiverBinding>
        </tns:DocExchange>
    </tns:PartyInfo>
    <tns:PartyInfo>
        <tns:PartyId>PartyB</tns:PartyId>
        <tns:PartyRef>ReferenceB</tns:PartyRef>
        <!-- Similar structure as the first PartyInfo -->
    </tns:PartyInfo>
    <tns:SimplePart>PartA</tns:SimplePart>
    <tns:Packaging>PackagingA</tns:Packaging>
    <tns:Signature>SignatureA</tns:Signature>
    <tns:Comment>CommentA</tns:Comment>
</tns:CollaborationProtocolAgreement>
```

[Go Back to Main Page](../Readme.md)