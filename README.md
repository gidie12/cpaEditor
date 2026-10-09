# CPA Editor

Desktop application (Tkinter) for viewing and editing ebXML CPA files (Collaboration Protocol Agreement, `cpp-cpa-2_0.xsd`).

## Setup

Python 3.11 is used for development and for the Windows build.

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Run

```bash
python main.py
```

Use **Load CPA** to open a CPA file and **Save CPA** to write the edited CPA to a file.

A step-by-step guide is available in English (`docs/how-to-use.en.md`) and Dutch (`docs/how-to-use.nl.md`). Inside the application the guides and this readme are in the **Help** menu.

## Tabs

- **General**: CPA id, status, party names, party ids, start date and end date
- **Collaboration Role**: collaboration roles of both partners
- **Transport**: transport elements of both partners
- **Comment**: comments in the CPA
- **XML editor**: the complete CPA as an editable tree
- **Certificates**: certificates and KeyInfo of both partners, upload of a new certificate chain, download of a certificate chain as PEM file
- **CPA Validation**: validates the CPA against the schema

Loading another CPA replaces the content of all tabs, including the validation result and any selected item.

## Zulu time and local time

Dates in a CPA are normally written in Zulu time (UTC), recognisable by the `Z` at the end. On the **General** tab the start date and end date are shown below their fields in both notations: in Zulu time and in the timezone set on the computer, for example `Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 CEST (UTC+02:00)`. Both are the same moment. A message is shown instead when the date is not valid or has no timezone.

## Set start date to now

On the **General** tab, the button **Set to now** below the start date sets the start date of the CPA to the current date and time in UTC, written as `YYYY-MM-DDTHH:MM:SSZ`.

## Set end date from certificates

On the **General** tab, the button **Set from certificates** below the end date sets the end date of the CPA to the expiry date of the certificate that expires first.

- Every certificate in the CPA counts: both partners and the whole chain (leaf, intermediate and root).
- The end date is written as `YYYY-MM-DDTHH:MM:SSZ` (UTC), in the field and in the `End` element of the CPA.
- The log shows which certificate determined the date: type, subject and certId.
- A warning is logged when that certificate has already expired, because the end date is then in the past.
- Without certificates in the CPA an error is logged and the end date is left unchanged.

A certificate without the CA flag is a leaf, a CA certificate issued by itself is a root, and any other CA certificate is an intermediate.

## Download a certificate

On the **Certificates** tab, right-click a **KeyInfo** row and choose **Download Certificate** to save its certificates as a PEM file. The file contains every certificate of that certId in the order of the CPA (leaf first) and can be uploaded again with **Upload Certificate**. The default file name is the certId.

## Debug messages

The checkbox **Show debug messages** below the log, next to the Save and Load buttons, switches debug messages in the log on and off. It is off by default and is not remembered after closing the application. Info, warning and error messages are always shown.

With debug messages on, **Set from certificates** also lists every certificate with its common name and end date, ordered from first to last to expire:

```
DEBUG - Certificate 1/3: partner.example.com expires 2027-03-01T12:00:00Z (leaf, certId PartnerA_ServerCert)
DEBUG - Certificate 2/3: Example Intermediate CA expires 2031-06-15T00:00:00Z (intermediate, certId PartnerA_ServerCert)
DEBUG - Certificate 3/3: Example Root CA expires 2040-01-01T00:00:00Z (root, certId PartnerA_ServerCert)
INFO - End date set to 2027-03-01T12:00:00Z: expiry of leaf certificate 'CN=partner.example.com' (certId PartnerA_ServerCert)
```

A certificate that is used under more than one certId is listed once per certId.

## Tests

```bash
python -m unittest discover -s application/tests -p "Test*.py"
```

## Build the Windows executable

```bash
pip install -r requirements-build.txt
pyinstaller --noconfirm cpaEditor.spec
```

The executable is written to `dist/CpaEditor.exe`. The GitHub workflow `Build Windows executable` runs the same build for pull requests and for `v*` tags.
