# CanSend

## Sub-elements
- `ThisPartyActionBinding`
- `OtherPartyActionBinding` (optional)
- `CanReceive` (optional, unbounded)

## In the CPA Editor
- Shown on the **Collaboration Role** tab as a **▶ Send** row with the action, channel, package, [OtherPartyActionBinding](OtherPartyActionBinding.md) and status.
- Select the row to change the action, package, channel and [BusinessTransactionCharacteristics](BusinessTransactionCharacteristics.md) on the right; **Save Changes** applies them. The `id` does not change, so the other partner keeps referring to it.
- **Add Service action** adds a CanSend; with **Also add the opposite action** the other partner gets the matching [CanReceive](CanReceive.md) and both refer to each other.
- **Delete action** asks whether the mirrored CanReceive of the other partner is deleted as well.
