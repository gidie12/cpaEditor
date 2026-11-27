# TODO – Rust version

## Digikoppeling check (CPA Validation tab)
Recognise the Digikoppeling ebMS2 profile of a CPA and report deviations per rule, with advice, like the schema
errors. Sources: Koppelvlakstandaard ebMS2 3.3.2 (chapter 5, profile tables with the CPA paths),
Best Practices ebMS2 3.2.2, Beveiligingsstandaarden 3.0.1
(https://gitdocumentatie.logius.nl/publicatie/dk/ebms/, …/bpebms/, …/beveilig/3.0.1/). Logius publishes no
Digikoppeling XSD or Schematron.

- Profiles: osb-be, osb-rm, osb-be-s, osb-rm-s, osb-be-e, osb-rm-e; all with two-way TLS.
- Reliable (-rm): `MessagingCharacteristics/@ackRequested="always"` with
  `@actor="urn:oasis:names:tc:ebxml-msg:actor:toPartyMSH"`, `@duplicateElimination="always"`,
  `@syncReplyMode` `none` or `mshSignalsOnly`; `ReliableMessaging` present (Retries/RetryInterval/PersistDuration
  free, default 5 days); `MessageOrderSemantics` discouraged.
- Best effort (-be): no `ReliableMessaging`, `duplicateElimination="never"`.
- Signed (-s): `ackSignatureRequested="always"`, `isNonRepudiationReceiptRequired`, `isAuthenticated`,
  `isTamperProof` = `persistent`; SHA-256 or stronger, `xml-exc-c14n`.
- Encrypted (-e): also `isConfidential="persistent"`; AES-128/256 (GCM preferred), RSA-OAEP, no 3DES.
- `PartyId/@type="urn:osb:oin"` with the OIN of the end party, equal to the OIN in the certificate
  (Subject serialNumber); environment suffix `_O`/`_T`/`_A` (none for production).
- Service type `urn:osb:services`; one profile per CPA; exactly one `defaultMshChannelId` per PartyInfo; one role
  per party.
- CPAId `[ServiceID]_[PartyId-A]_[PartyId-B]_[version]` without dots.
- Start/End follow the certificates; certificates PKIoverheid (G1/G3 to G4 before June 2027); TLS 1.2+.
- Do not check channel id names like `…_Reliable_S_Transport_R_3PT10M` (generator output, not a rule).
- To verify first: exact algorithm URIs, OIN format (20 digits), whether -e maps to `SenderDigitalEnvelope`.

## UI review (all screens 8 or higher)
- 8: Start, Collaboration Role, CPA Validation, Certificates, General, Comment.
- Transport (7): disabled buttons clearly weaker, keep the partner in view after a selection, show CN/expiry for
  a certId.
- XML editor (7): collapse whitespace in values (schemaLocation), no child elements under an element with a
  value, disabled buttons clearly weaker, no wrapping in Path/Namespace.

## Open from the tab builders
- Clone transport does not update references (DeliveryChannel `transportId`) to the new id.
- XML editor: an attribute added with a plain name gets no namespace (schema rejects CPA attributes; Python same).
- Unsaved edits in a detail panel are replaced when another tab changes the CPA; "+ New comment" drops them
  without asking.
- Certificates: SHA-256 fingerprint with Copy; "Open XML editor" button in the empty state; relative time in the
  tree; snapshots with a valid and an expiring certificate.
- Validation: "Show in XML editor" / "Go to Certificates" button on an error card.

## Docs
- README, EN/NL guides and schema reference pages for the Rust version.
  Include: General lists the other PartyIds of a partner (the first is edited, the others shown, change them in
  the XML editor). Certificates: "Remove all certificates…" strips certificates, transport TLS, trust anchors and
  message signing/encryption; https endpoints become http (warning per endpoint in the log).
