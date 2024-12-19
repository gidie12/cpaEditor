## cpp-cpa-2_0.xsd

### Table of Contents
## cpp-cpa-2_0.xsd

### Table of Contents
## cpp-cpa-2_0.xsd

### Table of Contents
## cpp-cpa-2_0.xsd

### Table of Contents
  - [CollaborationProtocolAgreement](elements/CollaborationProtocolAgreement.md)
    - [Status](elements/Status.md)
    - [Start](elements/Start.md)
    - [End](elements/End.md)
    - [ConversationConstraints](elements/ConversationConstraints.md)
    - [PartyInfo](elements/PartyInfo.md)
      - [PartyId](elements/PartyId.md)
      - [PartyRef](elements/PartyRef.md)
      - [CollaborationRole](elements/CollaborationRole.md)
        - [ProcessSpecification](elements/ProcessSpecification.md)
        - [Role](elements/Role.md)
        - [ApplicationCertificateRef](elements/ApplicationCertificateRef.md)
        - [ServiceBinding](elements/ServiceBinding.md)
          - [Service](elements/Service.md)
          - [CanSend](elements/CanSend.md)
            - [ThisPartyActionBinding](elements/ThisPartyActionBinding.md)
              - [BusinessTransactionCharacteristics](elements/BusinessTransactionCharacteristics.md)
              - [ActionContext](elements/ActionContext.md)
              - [ChannelId](elements/ChannelId.md)
            - [OtherPartyActionBinding](elements/OtherPartyActionBinding.md)
          - [CanReceive](elements/CanReceive.md)
            - [ThisPartyActionBinding](elements/ThisPartyActionBinding.md)
              - [BusinessTransactionCharacteristics](elements/BusinessTransactionCharacteristics.md)
              - [ActionContext](elements/ActionContext.md)
              - [ChannelId](elements/ChannelId.md)
            - [OtherPartyActionBinding](elements/OtherPartyActionBinding.md)
      - [Certificate](elements/Certificate.md)
      - [SecurityDetails](elements/SecurityDetails.md)
        - [TrustAnchors](elements/TrustAnchors.md)
          - [AnchorCertificateRef](elements/AnchorCertificateRef.md)
      - [DeliveryChannel](elements/DeliveryChannel.md)
        - [MessagingCharacteristics](elements/MessagingCharacteristics.md)
      - [Transport](elements/Transport.md)
        - [TransportSender](elements/TransportSender.md)
        - [TransportReceiver](elements/TransportReceiver.md)
        - [TransportClientSecurity](elements/TransportClientSecurity.md)
        - [TransportServerSecurity](elements/TransportServerSecurity.md)
      - [DocExchange](elements/DocExchange.md)
        - [ebXMLSenderBinding](elements/ebXMLSenderBinding.md)
          - [ReliableMessaging](elements/ReliableMessaging.md)
          - [PersistDuration](elements/PersistDuration.md)
          - [SenderNonRepudiation](elements/SenderNonRepudiation.md)
          - [SenderDigitalEnvelope](elements/SenderDigitalEnvelope.md)
        - [ebXMLReceiverBinding](elements/ebXMLReceiverBinding.md)
          - [ReliableMessaging](elements/ReliableMessaging.md)
          - [PersistDuration](elements/PersistDuration.md)
          - [ReceiverNonRepudiation](elements/ReceiverNonRepudiation.md)
          - [ReceiverDigitalEnvelope](elements/ReceiverDigitalEnvelope.md)
          - [NamespaceSupported](elements/NamespaceSupported.md)
    - [SimplePart](elements/SimplePart.md)
    - [Packaging](elements/Packaging.md)
    - [Signature](elements/Signature.md)
    - [Comment](elements/Comment.md)
  - [CollaborationProtocolProfile](elements/CollaborationProtocolProfile.md)
  - [Protocol](elements/Protocol.md)
  - [SendingProtocol](elements/SendingProtocol.md)
  - [ReceivingProtocol](elements/ReceivingProtocol.md)
  - [OverrideMshActionBinding](elements/OverrideMshActionBinding.md)
  - [ActionBinding.type](elements/ActionBinding.type.md)
  - [NamespaceSupported](elements/NamespaceSupported.md)
  - [MessagingCharacteristics](elements/MessagingCharacteristics.md)
  - [SignatureTransforms](elements/SignatureTransforms.md)
  - [EncryptionTransforms](elements/EncryptionTransforms.md)
  - [Constituent](elements/Constituent.md)
- [Common Types](elements/CommonTypes.md)


### Overview
The `cpp-cpa-2_0.xsd` file defines the structure and constraints for CPA (Collaboration Protocol Agreement) XML documents. This schema ensures that CPA XML files adhere to a specific format and contain the necessary elements and attributes.

### Elements and Attributes
Each element and attribute is documented in its respective markdown file in the `elements` directory.