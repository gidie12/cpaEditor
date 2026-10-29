# -*- mode: python ; coding: utf-8 -*-
# Build the standalone CPA Editor executable:
#   pip install -r requirements-build.txt
#   pyinstaller --noconfirm cpaEditor.spec
# PyInstaller cannot cross-compile, so the Windows .exe must be built on Windows
# (see .github/workflows/build-windows.yml).
import sys

from PyInstaller.compat import is_win
from PyInstaller.utils.hooks import collect_data_files

sys.path.insert(0, SPECPATH)
from application.helper_classes.updateCheck import executable_name
from application.helper_classes.windowsVersion import file_version
from application.version import __version__

# Version resource of the .exe (Properties > Details in Windows Explorer). The build workflow writes the version of
# the release or nightly build to application/version.py before this spec runs
version_info = None
if is_win:
    # Only importable on Windows; a version resource is not used on other platforms
    from PyInstaller.utils.win32.versioninfo import (FixedFileInfo, StringFileInfo, StringStruct, StringTable,
                                                     VarFileInfo, VarStruct, VSVersionInfo)

    version_numbers = file_version(__version__)
    version_info = VSVersionInfo(
        ffi=FixedFileInfo(filevers=version_numbers, prodvers=version_numbers),
        kids=[
            StringFileInfo([StringTable('040904B0', [
                StringStruct('FileDescription', 'CPA Editor'),
                StringStruct('FileVersion', __version__),
                StringStruct('InternalName', 'CpaEditor'),
                StringStruct('OriginalFilename', executable_name(__version__)),
                StringStruct('ProductName', 'CPA Editor'),
                StringStruct('ProductVersion', __version__),
            ])]),
            VarFileInfo([VarStruct('Translation', [0x0409, 1200])]),
        ],
    )

a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[('schemaValidation', 'schemaValidation'), ('README.md', '.'), ('docs', 'docs')] + collect_data_files('xmlschema'),
    hiddenimports=[],
    hookspath=[],
    runtime_hooks=[],
    excludes=['application.tests', 'application.not_released_functions'],
    noarchive=False,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    # The name holds the version, for example CpaEditor-1.0.2.exe; the update check downloads it by this name
    name=executable_name(__version__).removesuffix('.exe'),
    debug=False,
    strip=False,
    upx=False,
    runtime_tmpdir=None,
    console=False,
    version=version_info,
)
