//! Validation of a CPA against the CPA schema (cpp-cpa-2_0.xsd) with libxml2.
//!
//! The CPA schema imports the xlink, xmldsig and xml schemas from the internet. The validation never goes online:
//! the schemas are part of the program (the same fallback schemas the Python package xmlschema uses), they are
//! written to a temporary folder and the imports are pointed at those files.

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::collections::HashSet;
use std::path::Path;

use crate::xml::Document;
use crate::CPA_NAMESPACE;

const CPA_SCHEMA: &str = include_str!("../../../schemaValidation/cpp-cpa-2_0.xsd");
const XLINK_SCHEMA: &str = include_str!("../schemas/xlink.xsd");
const XMLDSIG_SCHEMA: &str = include_str!("../schemas/xmldsig-core-schema.xsd");
const XML_SCHEMA: &str = include_str!("../schemas/xml_minimal.xsd");

/// Remote schema locations and the bundled file that replaces each.
const LOCAL_SCHEMAS: &[(&str, &str)] = &[
    ("http://www.oasis-open.org/committees/ebxml-msg/schema/xlink.xsd", "xlink.xsd"),
    ("http://www.w3.org/TR/xmldsig-core/xmldsig-core-schema.xsd", "xmldsig-core-schema.xsd"),
    ("http://www.w3.org/2001/03/xml.xsd", "xml.xsd"),
    ("http://www.w3.org/2001/xml.xsd", "xml.xsd"),
];

/// One problem found by the validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationError {
    /// Line in the saved CPA (as written by `Document::to_xml`), 0 when unknown.
    pub line: u32,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(f, "Line {}: {}", self.line, self.message)
        } else {
            f.write_str(&self.message)
        }
    }
}

/// Validates a CPA (the XML text as it would be saved) against the CPA schema.
///
/// Returns the problems in the order libxml2 found them; empty when the CPA is valid. An Err means the schema
/// itself could not be loaded.
pub fn validate(xml: &str) -> Result<Vec<ValidationError>, String> {
    let folder = tempfile::tempdir().map_err(|e| format!("Cannot create a temporary folder: {e}"))?;
    write_schemas(folder.path())?;
    let schema_path = folder.path().join("cpp-cpa-2_0.xsd");
    // libxml2 keeps global state: one validation at a time
    let _guard = LIBXML_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe { validate_with_libxml(&schema_path, xml) }
}

/// Validates a CPA document: the schema validation of [`validate`] plus the check that every IDREF refers to an
/// existing ID, which libxml2 leaves out (the Python version reports it as "IDREF '...' not found in XML document").
pub fn validate_document(document: &Document) -> Result<Vec<ValidationError>, String> {
    let mut errors = validate(&document.to_xml())?;
    errors.extend(check_idrefs(document));
    Ok(errors)
}

/// Attributes of type ID per element in the CPA schema; `None` stands for any element (the global attribute
/// tns:id).
const ID_ATTRIBUTES: &[(Option<&str>, &str)] = &[
    (None, "id"),
    (Some("DeliveryChannel"), "channelId"),
    (Some("Transport"), "transportId"),
    (Some("Certificate"), "certId"),
    (Some("DocExchange"), "docExchangeId"),
    (Some("SecurityDetails"), "securityId"),
];

/// Attributes of type IDREF in the CPA schema, on every element where they are not an ID (see ID_ATTRIBUTES).
const IDREF_ATTRIBUTES: &[&str] = &[
    "idref", "channelId", "packageId", "defaultMshChannelId", "defaultMshPackageId", "transportId",
    "docExchangeId", "certId", "securityId",
];

/// Elements whose text is of type IDREF in the CPA schema.
const IDREF_ELEMENTS: &[&str] = &["ChannelId", "OtherPartyActionBinding"];

/// Reports every IDREF of the CPA whose ID does not exist.
pub fn check_idrefs(document: &Document) -> Vec<ValidationError> {
    let Some(root) = document.root() else { return Vec::new() };
    let is_id = |element: &str, attribute: &str| {
        ID_ATTRIBUTES.iter().any(|(owner, name)| *name == attribute && owner.is_none_or(|owner| owner == element))
    };
    let mut ids = HashSet::new();
    let mut references = Vec::new();
    for node in document.descendants(root) {
        let Some(name) = document.name(node) else { continue };
        if name.namespace.as_deref() != Some(CPA_NAMESPACE) {
            continue;
        }
        for attribute in document.attributes(node) {
            if attribute.name.namespace.as_deref() != Some(CPA_NAMESPACE) {
                continue;
            }
            if is_id(&name.local, &attribute.name.local) {
                ids.insert(attribute.value.trim().to_string());
            } else if IDREF_ATTRIBUTES.contains(&attribute.name.local.as_str()) {
                references.push(attribute.value.trim().to_string());
            }
        }
        if IDREF_ELEMENTS.contains(&name.local.as_str()) {
            references.push(document.text_trimmed(node));
        }
    }
    let mut reported = HashSet::new();
    references
        .into_iter()
        .filter(|reference| !ids.contains(reference) && reported.insert(reference.clone()))
        .map(|reference| ValidationError { line: 0, message: format!("IDREF '{reference}' not found in XML document") })
        .collect()
}

static LIBXML_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn point_imports_to_local_files(schema: &str) -> String {
    LOCAL_SCHEMAS.iter().fold(schema.to_string(), |text, (remote, local)| text.replace(remote, local))
}

/// libxml2 only accepts an xs:integer of at most 24 digits, but the serial number of a certificate has up to 49
/// digits. X509SerialNumber gets a type with the same lexical space (an optional sign and digits) without that
/// limit.
const SERIAL_NUMBER_INTEGER: &str = r#"<element name="X509SerialNumber" type="integer"/>"#;
const SERIAL_NUMBER_DIGITS: &str = r#"<element name="X509SerialNumber"><simpleType><restriction base="string"><whiteSpace value="collapse"/><pattern value="[\-+]?[0-9]+"/></restriction></simpleType></element>"#;

fn write_schemas(folder: &Path) -> Result<(), String> {
    for (name, text) in [
        ("cpp-cpa-2_0.xsd", CPA_SCHEMA),
        ("xlink.xsd", XLINK_SCHEMA),
        ("xmldsig-core-schema.xsd", &XMLDSIG_SCHEMA.replace(SERIAL_NUMBER_INTEGER, SERIAL_NUMBER_DIGITS)),
        ("xml.xsd", XML_SCHEMA),
    ] {
        if name == "xmldsig-core-schema.xsd" && !text.contains(SERIAL_NUMBER_DIGITS) {
            return Err("The X509SerialNumber of the bundled xmldsig schema is not where it was expected".into());
        }
        std::fs::write(folder.join(name), point_imports_to_local_files(text))
            .map_err(|e| format!("Cannot write the schema {name}: {e}"))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------
// libxml2
// ---------------------------------------------------------------------------------------------------------------

/// xmlError of libxml2 (the layout is the same in 2.9 up to 2.14).
#[repr(C)]
struct XmlError {
    domain: c_int,
    code: c_int,
    message: *const c_char,
    level: c_int,
    file: *const c_char,
    line: c_int,
    str1: *const c_char,
    str2: *const c_char,
    str3: *const c_char,
    int1: c_int,
    int2: c_int,
    ctxt: *mut c_void,
    node: *mut c_void,
}

type StructuredErrorFunc = unsafe extern "C" fn(user_data: *mut c_void, error: *const XmlError);

const XML_PARSE_NONET: c_int = 1 << 11;
const XML_PARSE_NOENT: c_int = 1 << 1;
const XML_PARSE_DTDLOAD: c_int = 1 << 2;

#[allow(non_snake_case)]
extern "C" {
    fn xmlSchemaNewParserCtxt(url: *const c_char) -> *mut c_void;
    fn xmlSchemaSetParserStructuredErrors(ctxt: *mut c_void, handler: StructuredErrorFunc, data: *mut c_void);
    fn xmlSchemaParse(ctxt: *mut c_void) -> *mut c_void;
    fn xmlSchemaFreeParserCtxt(ctxt: *mut c_void);
    fn xmlSchemaFree(schema: *mut c_void);
    fn xmlSchemaNewValidCtxt(schema: *mut c_void) -> *mut c_void;
    fn xmlSchemaSetValidStructuredErrors(ctxt: *mut c_void, handler: StructuredErrorFunc, data: *mut c_void);
    fn xmlSchemaValidateDoc(ctxt: *mut c_void, doc: *mut c_void) -> c_int;
    fn xmlSchemaFreeValidCtxt(ctxt: *mut c_void);
    fn xmlReadMemory(
        buffer: *const c_char,
        size: c_int,
        url: *const c_char,
        encoding: *const c_char,
        options: c_int,
    ) -> *mut c_void;
    fn xmlFreeDoc(doc: *mut c_void);
    fn xmlSetStructuredErrorFunc(ctx: *mut c_void, handler: Option<StructuredErrorFunc>);
}

unsafe extern "C" fn collect_error(user_data: *mut c_void, error: *const XmlError) {
    if user_data.is_null() || error.is_null() {
        return;
    }
    let errors = &mut *(user_data as *mut Vec<ValidationError>);
    let error = &*error;
    let message = if error.message.is_null() {
        "Unknown error".to_string()
    } else {
        CStr::from_ptr(error.message).to_string_lossy().trim_end().to_string()
    };
    errors.push(ValidationError { line: error.line.max(0) as u32, message });
}

unsafe fn validate_with_libxml(schema_path: &Path, xml: &str) -> Result<Vec<ValidationError>, String> {
    let mut schema_errors: Vec<ValidationError> = Vec::new();
    let mut errors: Vec<ValidationError> = Vec::new();
    let path = CString::new(schema_path.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;

    let parser = xmlSchemaNewParserCtxt(path.as_ptr());
    if parser.is_null() {
        return Err("Cannot create the schema parser".into());
    }
    xmlSchemaSetParserStructuredErrors(parser, collect_error, &mut schema_errors as *mut _ as *mut c_void);
    let schema = xmlSchemaParse(parser);
    xmlSchemaFreeParserCtxt(parser);
    if schema.is_null() {
        let details: Vec<String> = schema_errors.iter().map(|e| e.to_string()).collect();
        return Err(format!("Cannot load the CPA schema: {}", details.join("; ")));
    }

    // Errors of parsing the CPA itself (normally none: the text comes from Document::to_xml)
    xmlSetStructuredErrorFunc(&mut errors as *mut _ as *mut c_void, Some(collect_error));
    let document = xmlReadMemory(
        xml.as_ptr() as *const c_char,
        xml.len() as c_int,
        c"cpa.xml".as_ptr(),
        c"UTF-8".as_ptr(),
        XML_PARSE_NONET & !(XML_PARSE_NOENT | XML_PARSE_DTDLOAD),
    );
    xmlSetStructuredErrorFunc(std::ptr::null_mut(), None);
    if document.is_null() {
        xmlSchemaFree(schema);
        if errors.is_empty() {
            errors.push(ValidationError { line: 0, message: "The CPA is not well-formed XML".into() });
        }
        return Ok(errors);
    }

    let validator = xmlSchemaNewValidCtxt(schema);
    xmlSchemaSetValidStructuredErrors(validator, collect_error, &mut errors as *mut _ as *mut c_void);
    let result = xmlSchemaValidateDoc(validator, document);
    xmlSchemaFreeValidCtxt(validator);
    xmlFreeDoc(document);
    xmlSchemaFree(schema);
    if result != 0 && errors.is_empty() {
        errors.push(ValidationError { line: 0, message: format!("Validation failed (libxml2 code {result})") });
    }
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpa(name: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas").join(name);
        Document::parse_bytes(&std::fs::read(path).unwrap()).unwrap().to_xml()
    }

    fn document(name: &str) -> Document {
        Document::parse(&cpa(name)).unwrap()
    }

    #[test]
    fn valid_cpa_has_no_errors() {
        assert_eq!(validate_document(&document("full_cpa_p.xml")).unwrap(), vec![]);
    }

    #[test]
    fn dangling_idref_is_reported() {
        let errors = validate_document(&document("full_test_cpa.xml")).unwrap();
        assert_eq!(errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>(),
                   vec!["IDREF 'BROKER_ClientCerts' not found in XML document"]);
    }

    /// The number of problems per test CPA is the same as in the Python version (xmlschema).
    #[test]
    fn same_number_of_problems_as_python() {
        for (name, expected) in [("example.xml", 0), ("full_cpa_p.xml", 0), ("full_cpa_p_without_comments.xml", 0),
                                 ("full_test_cpa.xml", 1), ("new_cpa123.xml", 2), ("output.xml", 0),
                                 ("test123.xml", 0), ("test_cpa.xml", 2)] {
            let errors = validate_document(&document(name)).unwrap();
            assert_eq!(errors.len(), expected, "{name}: {errors:#?}");
        }
        // A CPA in another namespace is not valid at all
        assert!(!validate_document(&document("full_cpa.xml")).unwrap().is_empty());
    }

    #[test]
    fn long_serial_number_is_valid_and_text_is_not() {
        let xml = cpa("full_test_cpa.xml");
        assert!(xml.contains("594083952876165257564138781922060039604540126542"));
        let errors = validate(&xml.replacen("594083952876165257564138781922060039604540126542", "12ab", 1)).unwrap();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains("X509SerialNumber"));
    }

    #[test]
    fn invalid_cpa_reports_line_and_message() {
        let xml = cpa("full_test_cpa.xml").replacen("tns:value=\"agreed\"", "tns:value=\"maybe\"", 1);
        let errors = validate(&xml).unwrap();
        assert!(!errors.is_empty(), "the status 'maybe' is not allowed");
        assert!(errors[0].line > 0);
        assert!(errors[0].message.contains("maybe"), "{:?}", errors);
    }
}
