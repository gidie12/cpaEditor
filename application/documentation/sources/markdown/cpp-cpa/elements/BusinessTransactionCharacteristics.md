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

## In the CPA Editor
- On the **Collaboration Role** tab the attributes `isAuthenticated`, `isAuthorizationRequired`, `isIntelligibleCheckRequired`, `isNonRepudiationReceiptRequired`, `isNonRepudiationRequired`, `isConfidential`, `isTamperProof` and `timeToPerform` can be changed for the selected action. Boolean attributes are `true` or `false`, the persistence levels `none`, `transient`, `persistent` or `transient-and-persistent`. An empty value removes the attribute.
- A new action gets the values of the first action of the same service, else `transient` / `true` / `false` / `PT1H` as defaults.
