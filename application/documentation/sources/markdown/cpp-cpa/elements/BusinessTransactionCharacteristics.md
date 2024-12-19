# BusinessTransactionCharacteristics

## Attributes
- `isNonRepudiationRequired`: Whether non-repudiation is required (boolean).
- `isNonRepudiationReceiptRequired`: Whether non-repudiation receipt is required (boolean).
- `isConfidential`: The confidentiality level (persistenceLevel.type).
- `isAuthenticated`: The authentication level (persistenceLevel.type).
- `isTamperProof`: The tamper-proof level (persistenceLevel.type).
- `isAuthorizationRequired`: Whether authorization is required (boolean).
- `isIntelligibleCheckRequired`: Whether intelligible check is required (boolean).
- `timeToAcknowledgeReceipt`: The time to acknowledge receipt (duration).
- `timeToAcknowledgeAcceptance`: The time to acknowledge acceptance (duration).
- `timeToPerform`: The time to perform (duration).
- `retryCount`: The retry count (integer).