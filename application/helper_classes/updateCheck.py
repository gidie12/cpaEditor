import json
import re
import ssl
import urllib.request

import truststore

LATEST_RELEASE_API_URL = 'https://api.github.com/repos/gidie12/cpaEditor/releases/latest'
# Fixed addresses of the download: nothing from the response of the update check is ever opened
LATEST_RELEASE_PAGE_URL = 'https://github.com/gidie12/cpaEditor/releases/latest'
# The Windows executable that the release workflow attaches to every release
LATEST_RELEASE_DOWNLOAD_URL = 'https://github.com/gidie12/cpaEditor/releases/latest/download/CpaEditor.exe'


def parse_version(version):
    """
    Converts a version such as 'v1.0.2' or '1.0.2' to a tuple that can be compared.

    Returns:
        tuple: The numeric parts of the version, for example (1, 0, 2).

    Raises:
        ValueError: When the version is not a dotted number.

    Test Functions:
        - test_parse_version
        - test_parse_version_invalid
    """
    match = re.fullmatch(r'v?(\d+(?:\.\d+)*)', str(version).strip())
    if not match:
        raise ValueError(f"Not a version number: {version!r}")
    return tuple(int(part) for part in match.group(1).split('.'))


def is_newer(latest_version, current_version):
    """
    Tells whether latest_version is newer than current_version; '1.1' and '1.1.0' are the same version.

    Test Functions:
        - test_is_newer
    """
    latest, current = parse_version(latest_version), parse_version(current_version)
    length = max(len(latest), len(current))
    pad = lambda version: version + (0,) * (length - len(version))
    return pad(latest) > pad(current)


def get_latest_version(timeout=5):
    """
    Asks GitHub for the version of the latest release of the CPA Editor.

    Only the public release information is requested; nothing about the CPA or the user is sent.

    Returns:
        str: The tag of the latest release, for example 'v1.0.2'.

    Test Functions:
        - test_get_latest_version_reads_tag
    """
    request = urllib.request.Request(LATEST_RELEASE_API_URL, headers={'Accept': 'application/vnd.github+json',
                                                                      'User-Agent': 'cpaEditor-update-check'})
    # Verify the server with the certificates trusted by the operating system: a Python installation does not
    # always have its own, and a company network may use its own root certificate
    context = truststore.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    with urllib.request.urlopen(request, timeout=timeout, context=context) as response:
        return json.load(response)['tag_name']


def check_for_update(current_version, timeout=5):
    """
    Compares the running version with the latest release.

    Returns:
        dict: 'current' and, when the check succeeded, 'latest' and 'update_available'; 'error' when it failed.

    Test Functions:
        - test_check_for_update_newer_version
        - test_check_for_update_up_to_date
        - test_check_for_update_reports_error
    """
    result = {'current': current_version}
    try:
        latest_version = get_latest_version(timeout)
        result['update_available'] = is_newer(latest_version, current_version)
        result['latest'] = latest_version.lstrip('v')
    except Exception as e:
        result['error'] = str(e)
    return result
