# OtherPartyActionBinding

## Description
The `OtherPartyActionBinding` element contains the `id` of the [ThisPartyActionBinding](ThisPartyActionBinding.md) of the other partner that belongs to this action (type `IDREF`). A [CanSend](CanSend.md) of one partner refers to the [CanReceive](CanReceive.md) of the other partner, and the other way round.

## Example
```xml
<tns:CanSend>
    <tns:ThisPartyActionBinding tns:id="A_S1_S_Order_PV_1" tns:action="Order" tns:packageId="defaultPackage">
        ...
    </tns:ThisPartyActionBinding>
    <tns:OtherPartyActionBinding>B_S1_R_Order_PV_1</tns:OtherPartyActionBinding>
</tns:CanSend>
```

## In the CPA Editor
- Shown in the column **OtherPartyActionBinding** on the **Collaboration Role** tab.
- **Add Service action** fills it in for both actions when the mirrored action is added.
- An action is marked red when this element is missing, refers to an id that is not an action of the other partner, refers to an action with the same direction, or when that action does not refer back.
