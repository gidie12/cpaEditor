# CanReceive

## Sub-elements
- `ThisPartyActionBinding`
- `OtherPartyActionBinding` (optional)
- `CanSend` (optional, unbounded)

## In the CPA Editor
- Shown on the **Collaboration Role** tab as a **◀ Receive** row with the action, channel, package, [OtherPartyActionBinding](OtherPartyActionBinding.md) and status.
- Select the row to change the action, package, channel and [BusinessTransactionCharacteristics](BusinessTransactionCharacteristics.md) on the right; **Save Changes** applies them. The `id` does not change, so the other partner keeps referring to it.
- **Add Service action** adds a CanReceive; with **Also add the opposite action** the other partner gets the matching [CanSend](CanSend.md) and both refer to each other.
- **Delete action** asks whether the mirrored CanSend of the other partner is deleted as well.
