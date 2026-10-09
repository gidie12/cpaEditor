# How to use the CPA Editor

The CPA Editor opens an ebXML CPA file (Collaboration Protocol Agreement), lets you change it and saves it again as XML.

## 1. Open a CPA

- Click **Load CPA** at the bottom left of the window.
- Choose the CPA file (`.xml`).
- The log above the buttons shows `CPA data loaded successfully.` when the file is opened. An error is shown when the file cannot be read.

Loading another CPA replaces everything on screen. Changes that are not saved are lost.

## 2. Change the CPA

Each tab shows one part of the CPA. Changes are kept in memory until you save the CPA (see step 4).

### General

- Shows the CPA id, status, party names, party ids, party id types, start date and end date.
- Type a new value and leave the field (press Tab or click another field). The value is applied when you leave the field.
- Choose the status from the **Status** list.
- Dates are written as `YYYY-MM-DDTHH:MM:SSZ`, for example `2027-03-01T12:00:00Z`. The `Z` means Zulu time (UTC). In the Netherlands that is 1 hour (winter) or 2 hours (summer) earlier than the time on your clock.
- Below each date the same moment is shown twice: in Zulu time and in the timezone set on your computer. Example: `Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 CEST (UTC+02:00)`. A message is shown there when the date is not valid or has no timezone.
- **Set to now** below the start date fills in the current date and time in Zulu time.
- **Set from certificates** below the end date sets the end date to the expiry date of the certificate that expires first. All certificates of both partners count: leaf, intermediate and root. The log shows which certificate determined the date. A warning is shown when that certificate has already expired.

### Collaboration Role

- Shows the collaboration roles of both partners as a tree. This tab is for viewing; use the **XML editor** tab to change these values.

### Transport

- Shows the transport elements of both partners as a tree.
- Click a row that has a value. The label below the tree shows `Editing:` followed by the name of the field.
- Type the new value in the field below the tree. For `certId` and `securityId` you choose a value from the list instead; the list contains the ids of that partner.
- Click **Save Changes** to apply the value.

### Comment

- Shows the comments in the CPA.
- **New**: type a text in the field below the list and click **New**.
- **Update**: click a comment, change the text and click **Update**.
- **Delete**: click a comment and click **Delete**.

### XML editor

- Shows the complete CPA as a tree with elements and attributes.
- **Change a value**: select a row, double-click its **Value** column, type the new value and press Enter. Press Escape to cancel.
- **Add**: select the parent element, type the name in the first field and the value in the second field, choose **Element** or **Attribute** and click **Add**.
- **Delete**: select a row and click **Delete**.
- **Copy**: select a row, right-click it and choose **Copy Value**.

### Certificates

- Shows per partner each certificate id (`CertId`) with its `KeyInfo`.
- **Replace a certificate**: open a `CertId`, select the **KeyInfo** row, right-click it and choose **Upload Certificate**. Choose a PEM file (`.cer`, `.crt` or `.pem`). The file may contain the whole chain; put the leaf certificate first. The certificate must have an RSA key.
- **Download a certificate**: select the **KeyInfo** row, right-click it and choose **Download Certificate**. Choose a file name. The file is a PEM file with every certificate of that `CertId`, the leaf certificate first. Only certificates are saved; a CPA never contains a private key.
- **Copy**: select the **KeyInfo** row, right-click it and choose **Copy KeyInfo**.

After replacing a certificate you can use **Set from certificates** on the **General** tab to update the end date.

## 3. Validate the CPA

- Open the **CPA Validation** tab and click **Validate CPA**.
- `Geen fouten gevonden` means the CPA is valid according to the CPA schema (`cpp-cpa-2_0.xsd`).
- Otherwise the first error is shown. Fix it and validate again until no error is left.
- `Geen validatie uitgevoerd` means the loaded CPA has not been validated yet.

## 4. Save the CPA

- Click **Save CPA** at the bottom left of the window.
- Choose a file name and location. Use a new name to keep the original file.

## Log and debug messages

- The log above the buttons shows what the application did and which errors occurred.
- Tick **Show debug messages** to see more detail, for example every changed field. This is off when the application starts.
- With debug messages on, **Set from certificates** also lists every certificate with its common name and end date, the first to expire at the top.

## Help

- **Help > How to use (English)** opens this guide.
- **Help > Handleiding (Nederlands)** opens this guide in Dutch.
- **Help > Readme** opens the technical readme (installation, tests, build).

## Tips

- Right-click on macOS: click with two fingers or hold Control while clicking.
- Validate before you save, and keep a copy of the original CPA.
- Both partners must use exactly the same CPA. Send the saved file to the other party after a change.
