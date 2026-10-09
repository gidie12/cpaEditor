# -*- mode: python ; coding: utf-8 -*-
# Build the standalone CPA Editor executable:
#   pip install -r requirements-build.txt
#   pyinstaller --noconfirm cpaEditor.spec
# PyInstaller cannot cross-compile, so the Windows .exe must be built on Windows
# (see .github/workflows/build-windows.yml).
from PyInstaller.utils.hooks import collect_data_files

a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[('schemaValidation', 'schemaValidation')] + collect_data_files('xmlschema'),
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
    name='CpaEditor',
    debug=False,
    strip=False,
    upx=False,
    runtime_tmpdir=None,
    console=False,
)
