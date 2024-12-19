from xmlschema import XMLSchema

# Load the XSD schema
xsd = XMLSchema('../../schemaValidation/cpp-cpa-2_0.xsd')

# Generate XML data based on the XSD
xml_data = xsd.tostring(root='root')

# Write the XML data to a file
with open('schema_xml.xml', 'w') as f:
    f.write(xml_data)