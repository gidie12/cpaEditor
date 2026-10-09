import json
import re
import socket
import ssl
import sys
import urllib.error
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


def fetch_with_windows_internet_settings(url, headers, timeout=5):
    """
    Requests a URL through WinINet, the internet layer of Windows itself.

    WinINet uses the proxy settings of the user exactly like the browser does: an automatic configuration script
    (PAC), automatic detection and a proxy that asks for the Windows account. urllib knows none of these.

    Returns:
        bytes: The body of the response.

    Raises:
        OSError: When the request fails or the server does not answer with status 200.

    Test Functions:
        - test_get_latest_version_falls_back_to_windows_internet_settings
    """
    import ctypes
    from ctypes import wintypes

    INTERNET_OPEN_TYPE_PRECONFIG = 0
    INTERNET_OPTION_CONNECT_TIMEOUT, INTERNET_OPTION_RECEIVE_TIMEOUT = 2, 6
    # RELOAD | NO_CACHE_WRITE | SECURE | NO_COOKIES | NO_UI: a fresh answer over TLS, without dialogs
    flags = 0x80000000 | 0x04000000 | 0x00800000 | 0x00080000 | 0x00000200
    HTTP_QUERY_STATUS_CODE_AS_NUMBER = 19 | 0x20000000

    wininet = ctypes.WinDLL('wininet', use_last_error=True)
    wininet.InternetOpenW.restype = ctypes.c_void_p
    wininet.InternetOpenW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.LPCWSTR, wintypes.LPCWSTR,
                                      wintypes.DWORD]
    wininet.InternetOpenUrlW.restype = ctypes.c_void_p
    wininet.InternetOpenUrlW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR, wintypes.LPCWSTR, wintypes.DWORD,
                                         wintypes.DWORD, ctypes.c_void_p]
    wininet.InternetSetOptionW.restype = wintypes.BOOL
    wininet.InternetSetOptionW.argtypes = [ctypes.c_void_p, wintypes.DWORD, ctypes.c_void_p, wintypes.DWORD]
    wininet.HttpQueryInfoW.restype = wintypes.BOOL
    wininet.HttpQueryInfoW.argtypes = [ctypes.c_void_p, wintypes.DWORD, ctypes.c_void_p,
                                       ctypes.POINTER(wintypes.DWORD), ctypes.POINTER(wintypes.DWORD)]
    wininet.InternetReadFile.restype = wintypes.BOOL
    wininet.InternetReadFile.argtypes = [ctypes.c_void_p, ctypes.c_void_p, wintypes.DWORD,
                                         ctypes.POINTER(wintypes.DWORD)]
    wininet.InternetCloseHandle.restype = wintypes.BOOL
    wininet.InternetCloseHandle.argtypes = [ctypes.c_void_p]

    session = wininet.InternetOpenW(headers.get('User-Agent', ''), INTERNET_OPEN_TYPE_PRECONFIG, None, None, 0)
    if not session:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        milliseconds = wintypes.DWORD(int(timeout * 1000))
        for option in (INTERNET_OPTION_CONNECT_TIMEOUT, INTERNET_OPTION_RECEIVE_TIMEOUT):
            wininet.InternetSetOptionW(session, option, ctypes.byref(milliseconds), ctypes.sizeof(milliseconds))
        header_lines = ''.join(f"{name}: {value}\r\n" for name, value in headers.items() if name != 'User-Agent')
        response = wininet.InternetOpenUrlW(session, url, header_lines, len(header_lines), flags, None)
        if not response:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            status, size = wintypes.DWORD(0), wintypes.DWORD(ctypes.sizeof(wintypes.DWORD))
            if not wininet.HttpQueryInfoW(response, HTTP_QUERY_STATUS_CODE_AS_NUMBER, ctypes.byref(status),
                                          ctypes.byref(size), None):
                raise ctypes.WinError(ctypes.get_last_error())
            if status.value != 200:
                raise OSError(f"HTTP status {status.value}")
            body, buffer, read = b'', ctypes.create_string_buffer(8192), wintypes.DWORD(0)
            while True:
                if not wininet.InternetReadFile(response, buffer, len(buffer), ctypes.byref(read)):
                    raise ctypes.WinError(ctypes.get_last_error())
                if read.value == 0:
                    return body
                body += buffer.raw[:read.value]
        finally:
            wininet.InternetCloseHandle(response)
    finally:
        wininet.InternetCloseHandle(session)


def get_latest_version(timeout=5):
    """
    Asks GitHub for the version of the latest release of the CPA Editor.

    Only the public release information is requested; nothing about the CPA or the user is sent.

    On Windows a request that cannot get out directly is repeated with the internet settings of Windows, so the
    check also works behind a company proxy.

    Returns:
        str: The tag of the latest release, for example 'v1.0.2'.

    Test Functions:
        - test_get_latest_version_reads_tag
        - test_get_latest_version_falls_back_to_windows_internet_settings
    """
    headers = {'Accept': 'application/vnd.github+json', 'User-Agent': 'cpaEditor-update-check'}
    request = urllib.request.Request(LATEST_RELEASE_API_URL, headers=headers)
    # Verify the server with the certificates trusted by the operating system: a Python installation does not
    # always have its own, and a company network may use its own root certificate
    context = truststore.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    try:
        with urllib.request.urlopen(request, timeout=timeout, context=context) as response:
            body = response.read()
    except urllib.error.URLError as error:
        # An answer of GitHub itself (rate limit, not found) is final; 407 is the proxy asking for a login
        answered = isinstance(error, urllib.error.HTTPError) and error.code != 407
        if sys.platform != 'win32' or answered:
            raise
        try:
            body = fetch_with_windows_internet_settings(LATEST_RELEASE_API_URL, headers, timeout)
        except OSError as windows_error:
            raise error from windows_error
    return json.loads(body)['tag_name']


def describe_network_error(error):
    """
    Explains a failed request in words a user can act on.

    A name that cannot be resolved ('[Errno 11001] getaddrinfo failed' on Windows) means there is no internet
    connection, or the network only reaches the internet through a proxy that Python does not know about.

    Test Functions:
        - test_describe_network_error
    """
    message = str(error)
    if isinstance(getattr(error, 'reason', None), socket.gaierror):
        message = (f"api.github.com could not be found ({error.reason}): there is no internet connection, or the "
                   f"network needs a proxy (set HTTPS_PROXY). The latest version is at {LATEST_RELEASE_PAGE_URL}")
    if error.__cause__ is not None:
        message += f" (the internet settings of Windows did not work either: {error.__cause__})"
    return message


def check_for_update(current_version, timeout=5):
    """
    Compares the running version with the latest release.

    Returns:
        dict: 'current' and, when the check succeeded, 'latest' and 'update_available'; 'error' when it failed.

    Test Functions:
        - test_check_for_update_newer_version
        - test_check_for_update_up_to_date
        - test_check_for_update_reports_error
        - test_describe_network_error
    """
    result = {'current': current_version}
    try:
        latest_version = get_latest_version(timeout)
        result['update_available'] = is_newer(latest_version, current_version)
        result['latest'] = latest_version.lstrip('v')
    except urllib.error.URLError as e:
        result['error'] = describe_network_error(e)
    except Exception as e:
        result['error'] = str(e)
    return result
