# ServiceBinding

## Sub-elements
- `Service`
- `CanSend` (optional, unbounded)
- `CanReceive` (optional, unbounded)

## In the CPA Editor
- On the **Collaboration Role** tab every [CanSend](CanSend.md) and [CanReceive](CanReceive.md) of the ServiceBinding is a row below its service.
- **Add Service action** adds a new action in the order of the schema: a `CanSend` after the last `CanSend`, a `CanReceive` at the end.
