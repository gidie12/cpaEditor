# ThisPartyActionBinding

## Attributes
- `id`: A unique identifier for the action binding.
- `action`: The action name.
- `packageId`: The package ID.
- `xlink:href`: The link reference (optional).
- `xlink:type`: The link type (fixed to "simple").

## Sub-elements
- [BusinessTransactionCharacteristics](BusinessTransactionCharacteristics.md)
- [ActionContext](ActionContext.md) (optional)
- [ChannelId](ChannelId.md) (unbounded)
- Any other elements (optional, unbounded)

## In the CPA Editor
- **Add Service action** on the **Collaboration Role** tab proposes the `id` from an action with the same direction in the same service: its action name is replaced by the new action name, so the naming convention of the CPA is kept (for example `BROKER_S1_BrokerX_Ketenpartner_S_ProcessAntwoord_PV_0.1` becomes `..._S_ProcessStatus_PV_0.1`). Without such an action the id is `<partyName>_<service>_<S or R>_<action>`. The id is made unique in the CPA and can be changed before **Add**.
- The `packageId` is chosen from the `Packaging` elements of the CPA and the [ChannelId](ChannelId.md) from the `DeliveryChannel` elements of the partner.
