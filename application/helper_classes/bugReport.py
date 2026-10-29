import platform
import urllib.parse

# New issue with the template .github/ISSUE_TEMPLATE/bug_report.yml; a query parameter with the id of a field
# of the template fills in that field
NEW_BUG_REPORT_URL = 'https://github.com/gidie12/cpaEditor/issues/new'
BUG_REPORT_TEMPLATE = 'bug_report.yml'


def system_description():
    """
    Returns the operating system for a bug report, for example 'Windows 11' or 'macOS 15.6'.

    Test Functions:
        - test_system_description
    """
    if platform.system() == 'Darwin':
        return f"macOS {platform.mac_ver()[0]}".strip()
    return f"{platform.system()} {platform.release()}".strip()


def bug_report_url(version, system):
    """
    Returns the address of a new bug report with the version and operating system filled in.

    Only these two are sent along: nothing of the CPA or the log is put in the address.

    Test Functions:
        - test_bug_report_url
    """
    query = urllib.parse.urlencode({'template': BUG_REPORT_TEMPLATE, 'version': version, 'os': system})
    return f'{NEW_BUG_REPORT_URL}?{query}'
