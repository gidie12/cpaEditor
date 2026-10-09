import os
import sys
import xml.etree.ElementTree as ET
import xmlschema

# In a PyInstaller build the bundled data files live in sys._MEIPASS instead of the project root
BUNDLE_DIR = getattr(sys, '_MEIPASS', os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
DEFAULT_XSD = os.path.join(BUNDLE_DIR, 'schemaValidation', 'cpp-cpa-2_0.xsd')

class xmlValidation():
    def __init__(self, root=None, is_xml_object=False, xml_file=None, xsd_filename=DEFAULT_XSD):
        self.xsd_filename = xsd_filename
        self.xml_file = xml_file
        self.schema = xmlschema.XMLSchema(self.xsd_filename)
        self.is_xml_object = is_xml_object
        if is_xml_object:
            self.root = root
        self.validation_errors = ''

    def validate(self):
        try:
            # Load the XML Schema
            schema = xmlschema.XMLSchema(self.xsd_filename)

            if not self.is_xml_object:
                # Parse the XML document
                tree = ET.parse(self.xml_file)

                # Get the root element
                self.root = tree.getroot()
            # Validate the XML document against the schema
            try:
                schema.validate(self.root)
            except xmlschema.XMLSchemaValidationError as e:
                self.validation_errors = e
                return False
            return True
        except Exception as e:
            print('Cant open file')
            return False

    def get_validation_errors(self):
        return self.validation_errors