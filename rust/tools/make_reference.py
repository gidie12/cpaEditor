"""Writes what the Python CPA Editor saves for every test CPA, so the Rust tests can compare byte for byte.

Run from the rust directory with a Python that has the packages of requirements.txt:
    python tools/make_reference.py
"""
import pathlib

from lxml import etree

CPAS = pathlib.Path('../application/tests/resources/cpas')
OUT = pathlib.Path('cpa-core/tests/reference')
PARSER = etree.XMLParser(resolve_entities=False, no_network=True, load_dtd=False)

for cpa in sorted(CPAS.glob('*.xml')):
    tree = etree.parse(str(cpa), PARSER)
    (OUT / cpa.name).write_bytes(etree.tostring(tree, encoding='UTF-8', xml_declaration=True))
    print(cpa.name)
