from application.helper_classes.updateCheck import parse_version

# Each number of a Windows file version is 16 bits
MAX_VERSION_PART = 65535


def file_version(version):
    """
    Converts the version of the CPA Editor to the four numbers of a Windows file version.

    A prerelease label or build information is left out ('1.0.5+nightly.20261029.abc1234' becomes (1, 0, 5, 0));
    the full version is kept in the text of the version resource (see cpaEditor.spec).

    Test Functions:
        - test_file_version_pads_to_four_numbers
        - test_file_version_leaves_out_label
        - test_file_version_limits_numbers
    """
    parts = parse_version(version)[:4]
    return tuple(min(part, MAX_VERSION_PART) for part in parts + (0,) * (4 - len(parts)))
