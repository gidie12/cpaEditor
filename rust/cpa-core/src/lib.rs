//! Core of the CPA Editor: the XML document, the CPA model and the editing logic, without user interface.

pub mod certificates;
pub mod collaboration_roles;
pub mod cpa;
pub mod dates;
pub mod diff;
pub mod fixes;
pub mod help;
pub mod update_check;
pub mod validation;
pub mod xml;

/// Namespace of the CPA schema (cpp-cpa-2_0.xsd).
pub const CPA_NAMESPACE: &str = "http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd";
/// Namespace of XML Signature (KeyInfo, X509Certificate).
pub const DS_NAMESPACE: &str = "http://www.w3.org/2000/09/xmldsig#";
/// Namespace of XLink (xlink:href, xlink:type).
pub const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";
/// Date format of the CPA (xsd:dateTime in Zulu time).
pub const CPA_DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

/// Version of the CPA Editor, compared with the latest release by the update check.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
