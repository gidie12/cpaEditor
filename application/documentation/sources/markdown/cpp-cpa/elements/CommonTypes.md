### Common Types
- **statusValue.type**
  - Possible values: `agreed`, `signed`, `proposed`

- **endpointType.type**
  - Possible values: `login`, `request`, `response`, `error`, `allPurpose`

- **non-empty-string**
  - A string with a minimum length of 1

- **syncReplyMode.type**
  - Possible values: `mshSignalsOnly`, `responseOnly`, `signalsAndResponse`, `signalsOnly`, `none`

- **service.type**
  - A non-empty string with an optional `type` attribute

- **protocol.type**
  - A non-empty string with an optional `version` attribute

- **perMessageCharacteristics.type**
  - Possible values: `always`, `never`, `perMessage`

- **actor.type**
  - Possible values: `urn:oasis:names:tc:ebxml-msg:actor:nextMSH`, `urn:oasis:names:tc:ebxml-msg:actor:toPartyMSH`

- **messageOrderSemantics.type**
  - Possible values: `Guaranteed`, `NotGuaranteed`

- **persistenceLevel.type**
  - Possible values: `none`, `transient`, `persistent`, `transient-and-persistent`

- **accessAuthentication.type**
  - Possible values: `basic`, `digest`