# End

## Type
- `dateTime`

## Format
- Written as `YYYY-MM-DDTHH:MM:SSZ`, for example `2027-03-01T12:00:00Z`.
- The `Z` means Zulu time (UTC). A date with an offset such as `2027-03-01T13:00:00+01:00` is the same moment and is also valid.

## In the CPA Editor
- The **End date** field on the **General** tab.
- Below the field the same moment is shown in Zulu time and in the timezone set on the computer, for example `Zulu: 2027-03-01T12:00:00Z    Local: 2027-03-01 13:00:00 CET (UTC+01:00)`.
- The button **Set from certificates** sets the end date to the expiry date of the certificate in the CPA that expires first. Every certificate counts: both partners and the whole chain (leaf, intermediate and root). Certificate dates are always in Zulu time.
- With **Show debug messages** on, the log also lists every certificate with its common name and end date, the first to expire at the top.
