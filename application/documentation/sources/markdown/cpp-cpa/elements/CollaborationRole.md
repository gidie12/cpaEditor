# CollaborationRole

## Sub-elements
- [ProcessSpecification](ProcessSpecification.md)
- [Role](Role.md)
- `ApplicationCertificateRef` (optional, unbounded)
- `ApplicationSecurityDetailsRef` (optional)
- [ServiceBinding](ServiceBinding.md)

## In the CPA Editor
- The **Collaboration Role (experimental)** tab is experimental: check the result on the **CPA Validation** tab before you save the CPA.
- Shown on the **Collaboration Role** tab as a **Service** row below the partner, with the service type, the role and the process.
- **Add Collaboration Role** (toolbar or right-click) adds a CollaborationRole with a new service to the partner of the selected row, by default also at the other partner with its own role name. The fields are filled in from an existing CollaborationRole of the partner; **Generate UUID** fills in a `urn:uuid:` value for the process.
- **Delete Collaboration Role** (right-click or **Delete**) removes it with all its actions. A partner keeps at least one CollaborationRole (schema).
