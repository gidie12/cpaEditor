from datetime import datetime, timezone

ZULU_DATE_FORMAT = '%Y-%m-%dT%H:%M:%SZ'


def describe_cpa_date(value, local_timezone=None):
    """
    Describes a CPA date (xsd dateTime) in Zulu time (UTC) and in local time.

    Args:
        value (str): The date as written in the CPA, for example '2026-10-09T10:44:32Z'.
        local_timezone (tzinfo): The timezone for the local time; the timezone set on this computer when omitted.

    Returns:
        str: Both notations of the same moment, or the reason why they cannot be given.

    Test Functions:
        - test_describe_cpa_date_zulu_and_local
        - test_describe_cpa_date_with_offset
        - test_describe_cpa_date_without_timezone
        - test_describe_cpa_date_invalid
        - test_describe_cpa_date_empty
    """
    if not value or not value.strip():
        return ""
    try:
        moment = datetime.fromisoformat(value.strip())
    except ValueError:
        return "Not a valid date, expected for example 2026-10-09T10:44:32Z"
    if moment.tzinfo is None:
        return "Date has no timezone: add Z for Zulu time (UTC)"
    local = moment.astimezone(local_timezone)
    offset = local.strftime('%z')
    return (f"Zulu: {moment.astimezone(timezone.utc).strftime(ZULU_DATE_FORMAT)}    "
            f"Local: {local.strftime('%Y-%m-%d %H:%M:%S')} {local.tzname()} (UTC{offset[:3]}:{offset[3:]})")
