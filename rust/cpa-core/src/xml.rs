//! A small mutable XML document that keeps everything the CPA Editor must write back unchanged: namespace
//! declarations and their prefixes, the order of attributes, comments, processing instructions and all
//! whitespace. Serialisation follows lxml (`etree.tostring(tree, encoding='UTF-8', xml_declaration=True)`), so a
//! CPA saved without changes is byte for byte the CPA the Python version writes.
//!
//! Nodes live in an arena and are addressed by [`NodeId`]. A removed node stays in the arena without a parent, so a
//! `NodeId` held by the GUI never points to another node.
//!
//! Text is stored as text nodes. [`Document::text`] and [`Document::set_text`] give the lxml view: the text before
//! the first child element.

use std::fmt::Write as _;

use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;

pub const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

/// Identifies a node of a [`Document`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    /// A number that identifies the node within its document, for example as key of a row in the GUI.
    pub fn index(self) -> usize {
        self.0
    }
}

/// A name with its namespace URI. The prefix is the one of the parsed document; a new name gets the prefix that is
/// in scope when the document is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QName {
    pub namespace: Option<String>,
    pub local: String,
    pub prefix: Option<String>,
}

impl QName {
    pub fn new(namespace: Option<&str>, local: &str) -> Self {
        QName { namespace: namespace.map(str::to_string), local: local.to_string(), prefix: None }
    }

    pub fn is(&self, namespace: Option<&str>, local: &str) -> bool {
        self.local == local && self.namespace.as_deref() == namespace
    }

    /// The name in Clark notation as lxml shows it: `{namespace}local`.
    pub fn clark(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{{{ns}}}{}", self.local),
            None => self.local.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub name: QName,
    pub value: String,
}

/// A namespace declaration: `xmlns="uri"` (prefix None) or `xmlns:prefix="uri"`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceDecl {
    pub prefix: Option<String>,
    pub uri: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub name: QName,
    pub namespaces: Vec<NamespaceDecl>,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    Element(Element),
    Text(String),
    Comment(String),
    ProcessingInstruction { target: String, data: String },
    DocType(String),
}

#[derive(Clone, Debug)]
struct Node {
    kind: NodeKind,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

#[derive(Debug, thiserror::Error)]
pub enum XmlError {
    #[error("XML syntax error: {0}")]
    Syntax(String),
    #[error("unknown namespace prefix '{0}'")]
    UnknownPrefix(String),
    #[error("the document has no root element")]
    NoRoot,
}

impl From<quick_xml::Error> for XmlError {
    fn from(error: quick_xml::Error) -> Self {
        XmlError::Syntax(error.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for XmlError {
    fn from(error: quick_xml::events::attributes::AttrError) -> Self {
        XmlError::Syntax(error.to_string())
    }
}

/// A mutable XML document.
#[derive(Clone, Debug)]
pub struct Document {
    nodes: Vec<Node>,
}

impl Default for Document {
    fn default() -> Self {
        Document { nodes: vec![Node { kind: NodeKind::Document, parent: None, children: Vec::new() }] }
    }
}

impl Document {
    // ----------------------------------------------------------------------------------------------------------
    // Parsing
    // ----------------------------------------------------------------------------------------------------------

    /// Parses the bytes of an XML file: a byte order mark and the encoding of the XML declaration are taken into
    /// account (UTF-8 when there is none), like lxml does.
    pub fn parse_bytes(bytes: &[u8]) -> Result<Document, XmlError> {
        let (encoding, bom_length) = match encoding_rs::Encoding::for_bom(bytes) {
            Some((encoding, length)) => (encoding, length),
            None => (declared_encoding(bytes).unwrap_or(encoding_rs::UTF_8), 0),
        };
        let (text, _, had_errors) = encoding.decode(&bytes[bom_length..]);
        if had_errors {
            return Err(XmlError::Syntax(format!("the file is not valid {}", encoding.name())));
        }
        Document::parse(&text)
    }

    /// Parses an XML document. External entities and DTDs are never loaded (CPA files come from other parties): a
    /// DOCTYPE is kept as text and written back, an unknown entity is an error.
    ///
    /// Line ends are normalised to `\n` and tabs and line ends in attribute values to spaces, as the XML
    /// specification requires (and lxml does); character references such as `&#10;` keep their character.
    pub fn parse(xml: &str) -> Result<Document, XmlError> {
        let xml = xml.strip_prefix('\u{feff}').unwrap_or(xml);
        let normalised;
        let xml = if xml.contains('\r') {
            normalised = xml.replace("\r\n", "\n").replace('\r', "\n");
            normalised.as_str()
        } else {
            xml
        };
        let mut document = Document::default();
        let mut reader = NsReader::from_str(xml);
        let config = reader.config_mut();
        config.trim_text(false);
        config.expand_empty_elements = false;
        config.check_end_names = true;

        let mut stack: Vec<NodeId> = vec![document.document_node()];
        loop {
            let event = reader.read_event()?;
            let parent = *stack.last().expect("the document node stays on the stack");
            match event {
                Event::Start(start) => {
                    let element = document.read_element(&reader, &start)?;
                    let id = document.push_node(NodeKind::Element(element), Some(parent));
                    stack.push(id);
                }
                Event::Empty(start) => {
                    let element = document.read_element(&reader, &start)?;
                    document.push_node(NodeKind::Element(element), Some(parent));
                }
                Event::End(_) => {
                    stack.pop();
                }
                Event::Text(text) => {
                    // Whitespace outside the root element is not kept, like lxml does
                    if parent == document.document_node() {
                        continue;
                    }
                    let value = text.unescape().map_err(|e| XmlError::Syntax(e.to_string()))?.into_owned();
                    document.append_text(parent, &value);
                }
                Event::CData(data) => {
                    // lxml replaces CDATA sections by text (strip_cdata)
                    let value = String::from_utf8_lossy(&data.into_inner()).into_owned();
                    if parent != document.document_node() {
                        document.append_text(parent, &value);
                    }
                }
                Event::Comment(comment) => {
                    let value = String::from_utf8_lossy(&comment.into_inner()).into_owned();
                    document.push_node(NodeKind::Comment(value), Some(parent));
                }
                Event::PI(pi) => {
                    let content = String::from_utf8_lossy(&pi.into_inner()).into_owned();
                    let (target, data) = match content.find(|c: char| c.is_whitespace()) {
                        Some(position) => {
                            (content[..position].to_string(), content[position..].trim_start().to_string())
                        }
                        None => (content.clone(), String::new()),
                    };
                    document.push_node(NodeKind::ProcessingInstruction { target, data }, Some(parent));
                }
                Event::DocType(doctype) => {
                    let value = String::from_utf8_lossy(&doctype.into_inner()).trim().to_string();
                    document.push_node(NodeKind::DocType(value), Some(parent));
                }
                Event::Decl(_) => {}
                Event::Eof => break,
            }
        }
        if stack.len() != 1 {
            return Err(XmlError::Syntax("unexpected end of the document".into()));
        }
        if document.root().is_none() {
            return Err(XmlError::NoRoot);
        }
        Ok(document)
    }

    fn read_element(&self, reader: &NsReader<&[u8]>, start: &BytesStart) -> Result<Element, XmlError> {
        let (resolved, local) = reader.resolve_element(start.name());
        let raw_name = String::from_utf8_lossy(start.name().as_ref()).into_owned();
        let name = QName {
            namespace: resolve_namespace(resolved, &raw_name)?,
            local: String::from_utf8_lossy(local.as_ref()).into_owned(),
            prefix: prefix_of(&raw_name),
        };
        let mut namespaces = Vec::new();
        let mut attributes = Vec::new();
        for attribute in start.attributes().with_checks(true) {
            let attribute = attribute?;
            let key = String::from_utf8_lossy(attribute.key.as_ref()).into_owned();
            // Attribute-value normalisation on the raw value: a literal tab or line end becomes a space
            let raw = String::from_utf8_lossy(&attribute.value).replace(['\t', '\n'], " ");
            let value = quick_xml::escape::unescape(&raw).map_err(|e| XmlError::Syntax(e.to_string()))?.into_owned();
            if key == "xmlns" {
                namespaces.push(NamespaceDecl { prefix: None, uri: value });
                continue;
            }
            if let Some(prefix) = key.strip_prefix("xmlns:") {
                namespaces.push(NamespaceDecl { prefix: Some(prefix.to_string()), uri: value });
                continue;
            }
            let (resolved, local) = reader.resolve_attribute(attribute.key);
            attributes.push(Attribute {
                name: QName {
                    namespace: resolve_namespace(resolved, &key)?,
                    local: String::from_utf8_lossy(local.as_ref()).into_owned(),
                    prefix: prefix_of(&key),
                },
                value,
            });
        }
        Ok(Element { name, namespaces, attributes })
    }

    // ----------------------------------------------------------------------------------------------------------
    // Serialisation
    // ----------------------------------------------------------------------------------------------------------

    /// Writes the document like lxml: an XML declaration with single quotes on its own line, then the nodes
    /// before the root element, the root element and the nodes after it without whitespace in between, empty
    /// elements as `<a/>`.
    pub fn to_xml(&self) -> String {
        let mut out = String::from("<?xml version='1.0' encoding='UTF-8'?>\n");
        for &child in &self.node(self.document_node()).children {
            self.write_node(child, &mut out, &mut Vec::new());
        }
        out
    }

    /// Writes one element with its content, without XML declaration. The namespaces of its ancestors that it uses
    /// are declared on the element itself, like lxml's `tostring(element)`.
    pub fn element_to_xml(&self, id: NodeId) -> String {
        let mut out = String::new();
        let mut inherited: Vec<NamespaceDecl> = Vec::new();
        for ancestor in self.ancestors(id).skip(1) {
            if let NodeKind::Element(element) = &self.node(ancestor).kind {
                for declaration in &element.namespaces {
                    if !inherited.iter().any(|d| d.prefix == declaration.prefix) {
                        inherited.push(declaration.clone());
                    }
                }
            }
        }
        let used = self.used_namespaces(id);
        let mut extra: Vec<NamespaceDecl> =
            inherited.into_iter().filter(|d| used.iter().any(|u| u == &d.uri)).collect();
        // Outermost ancestor first, like lxml
        extra.reverse();
        self.write_element(id, &mut out, &mut Vec::new(), extra);
        out
    }

    fn used_namespaces(&self, id: NodeId) -> Vec<String> {
        let mut used = Vec::new();
        for node in self.descendants(id) {
            if let Some(element) = self.element(node) {
                let names = std::iter::once(&element.name).chain(element.attributes.iter().map(|a| &a.name));
                for name in names {
                    if let Some(ns) = &name.namespace {
                        if !used.contains(ns) {
                            used.push(ns.clone());
                        }
                    }
                }
            }
        }
        used
    }

    fn write_node(&self, id: NodeId, out: &mut String, scope: &mut Vec<Vec<NamespaceDecl>>) {
        match &self.node(id).kind {
            NodeKind::Document => {}
            NodeKind::Element(_) => self.write_element(id, out, scope, Vec::new()),
            NodeKind::Text(text) => escape_text(text, out),
            NodeKind::Comment(comment) => {
                let _ = write!(out, "<!--{comment}-->");
            }
            NodeKind::ProcessingInstruction { target, data } => {
                if data.is_empty() {
                    let _ = write!(out, "<?{target}?>");
                } else {
                    let _ = write!(out, "<?{target} {data}?>");
                }
            }
            NodeKind::DocType(doctype) => {
                let _ = write!(out, "<!DOCTYPE {doctype}>");
            }
        }
    }

    fn write_element(
        &self,
        id: NodeId,
        out: &mut String,
        scope: &mut Vec<Vec<NamespaceDecl>>,
        extra: Vec<NamespaceDecl>,
    ) {
        let NodeKind::Element(element) = &self.node(id).kind else { return };
        let mut declarations: Vec<NamespaceDecl> = extra;
        for declaration in &element.namespaces {
            declarations.retain(|d| d.prefix != declaration.prefix);
            declarations.push(declaration.clone());
        }
        scope.push(declarations);

        let tag = self.qualified_name(&element.name, true, scope);
        let mut attributes = String::new();
        for attribute in &element.attributes {
            let name = self.qualified_name(&attribute.name, false, scope);
            attributes.push(' ');
            attributes.push_str(&name);
            attributes.push_str("=\"");
            escape_attribute(&attribute.value, &mut attributes);
            attributes.push('"');
        }
        // Declarations, including those added for new names, come before the attributes (lxml)
        out.push('<');
        out.push_str(&tag);
        for declaration in scope.last().expect("pushed above") {
            match &declaration.prefix {
                Some(prefix) => {
                    let _ = write!(out, " xmlns:{prefix}=\"");
                }
                None => out.push_str(" xmlns=\""),
            }
            escape_attribute(&declaration.uri, out);
            out.push('"');
        }
        out.push_str(&attributes);

        let children = &self.node(id).children;
        if children.is_empty() {
            out.push_str("/>");
        } else {
            out.push('>');
            for &child in children {
                self.write_node(child, out, scope);
            }
            let _ = write!(out, "</{tag}>");
        }
        scope.pop();
    }

    /// The prefixed name for writing. The prefix of the parsed document is used when it still means the same
    /// namespace; otherwise a prefix in scope, otherwise a new declaration `ns0`, `ns1`, ... on this element.
    fn qualified_name(&self, name: &QName, is_element: bool, scope: &mut [Vec<NamespaceDecl>]) -> String {
        let Some(namespace) = &name.namespace else {
            return name.local.clone();
        };
        if namespace == XML_NAMESPACE {
            return format!("xml:{}", name.local);
        }
        let lookup = |scope: &[Vec<NamespaceDecl>], prefix: &Option<String>| -> Option<String> {
            scope.iter().rev().flat_map(|level| level.iter()).find(|d| &d.prefix == prefix).map(|d| d.uri.clone())
        };
        if let Some(prefix) = &name.prefix {
            if lookup(scope, &Some(prefix.clone())).as_deref() == Some(namespace) {
                return format!("{prefix}:{}", name.local);
            }
        } else if is_element && lookup(scope, &None).as_deref() == Some(namespace) {
            return name.local.clone();
        }
        // Any prefix in scope for the namespace that is not shadowed; attributes never use the default namespace
        let mut seen: Vec<Option<String>> = Vec::new();
        for level in scope.iter().rev() {
            for declaration in level.iter().rev() {
                if seen.contains(&declaration.prefix) {
                    continue;
                }
                seen.push(declaration.prefix.clone());
                if &declaration.uri == namespace && (declaration.prefix.is_some() || is_element) {
                    return match &declaration.prefix {
                        Some(prefix) => format!("{prefix}:{}", name.local),
                        None => name.local.clone(),
                    };
                }
            }
        }
        let mut number = 0;
        let prefix = loop {
            let candidate = format!("ns{number}");
            if lookup(scope, &Some(candidate.clone())).is_none() {
                break candidate;
            }
            number += 1;
        };
        scope
            .last_mut()
            .expect("the element pushed its level")
            .push(NamespaceDecl { prefix: Some(prefix.clone()), uri: namespace.clone() });
        format!("{prefix}:{}", name.local)
    }

    // ----------------------------------------------------------------------------------------------------------
    // Navigation
    // ----------------------------------------------------------------------------------------------------------

    fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0]
    }

    pub fn document_node(&self) -> NodeId {
        NodeId(0)
    }

    /// The root element.
    pub fn root(&self) -> Option<NodeId> {
        self.node(self.document_node()).children.iter().copied().find(|&child| self.is_element(child))
    }

    pub fn kind(&self, id: NodeId) -> &NodeKind {
        &self.node(id).kind
    }

    pub fn is_element(&self, id: NodeId) -> bool {
        matches!(self.node(id).kind, NodeKind::Element(_))
    }

    pub fn element(&self, id: NodeId) -> Option<&Element> {
        match &self.node(id).kind {
            NodeKind::Element(element) => Some(element),
            _ => None,
        }
    }

    pub fn element_mut(&mut self, id: NodeId) -> Option<&mut Element> {
        match &mut self.node_mut(id).kind {
            NodeKind::Element(element) => Some(element),
            _ => None,
        }
    }

    /// The name of an element, None for other nodes.
    pub fn name(&self, id: NodeId) -> Option<&QName> {
        self.element(id).map(|element| &element.name)
    }

    /// The local name of an element, empty for other nodes.
    pub fn local_name(&self, id: NodeId) -> &str {
        self.name(id).map(|name| name.local.as_str()).unwrap_or("")
    }

    pub fn is_named(&self, id: NodeId, namespace: Option<&str>, local: &str) -> bool {
        self.name(id).is_some_and(|name| name.is(namespace, local))
    }

    /// The parent node; the document node for the root element; None for the document node and detached nodes.
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).parent
    }

    /// The parent element, None for the root element.
    pub fn parent_element(&self, id: NodeId) -> Option<NodeId> {
        self.parent(id).filter(|&parent| self.is_element(parent))
    }

    /// All child nodes (elements, text, comments, ...).
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        &self.node(id).children
    }

    /// The child elements.
    pub fn child_elements(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.node(id).children.iter().copied().filter(|&child| self.is_element(child))
    }

    /// The child elements with the name.
    pub fn find_all<'a>(
        &'a self,
        id: NodeId,
        namespace: Option<&'a str>,
        local: &'a str,
    ) -> impl Iterator<Item = NodeId> + 'a {
        self.child_elements(id).filter(move |&child| self.is_named(child, namespace, local))
    }

    /// The first child element with the name.
    pub fn find(&self, id: NodeId, namespace: Option<&str>, local: &str) -> Option<NodeId> {
        self.child_elements(id).find(|&child| self.is_named(child, namespace, local))
    }

    /// The node itself and all its descendants in document order.
    pub fn descendants(&self, id: NodeId) -> Descendants<'_> {
        Descendants { document: self, stack: vec![id] }
    }

    /// The element itself and all descendant elements with the name, in document order (lxml `iter(tag)`).
    pub fn iter_named<'a>(
        &'a self,
        id: NodeId,
        namespace: Option<&'a str>,
        local: &'a str,
    ) -> impl Iterator<Item = NodeId> + 'a {
        self.descendants(id).filter(move |&node| self.is_named(node, namespace, local))
    }

    /// The node itself and its ancestors up to the document node.
    pub fn ancestors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(Some(id), move |&node| self.parent(node))
    }

    /// The nearest element (the element itself included) with the name.
    pub fn ancestor_named(&self, id: NodeId, namespace: Option<&str>, local: &str) -> Option<NodeId> {
        self.ancestors(id).find(|&node| self.is_named(node, namespace, local))
    }

    /// Whether the node is in the document (not removed).
    pub fn is_attached(&self, id: NodeId) -> bool {
        self.ancestors(id).last() == Some(self.document_node())
    }

    /// The previous sibling element.
    pub fn previous_element(&self, id: NodeId) -> Option<NodeId> {
        let parent = self.parent(id)?;
        let siblings = &self.node(parent).children;
        let position = siblings.iter().position(|&s| s == id)?;
        siblings[..position].iter().rev().copied().find(|&s| self.is_element(s))
    }

    /// The next sibling element.
    pub fn next_element(&self, id: NodeId) -> Option<NodeId> {
        let parent = self.parent(id)?;
        let siblings = &self.node(parent).children;
        let position = siblings.iter().position(|&s| s == id)?;
        siblings[position + 1..].iter().copied().find(|&s| self.is_element(s))
    }

    /// The position of a node among all child nodes of its parent.
    pub fn index_in_parent(&self, id: NodeId) -> Option<usize> {
        let parent = self.parent(id)?;
        self.node(parent).children.iter().position(|&s| s == id)
    }

    // ----------------------------------------------------------------------------------------------------------
    // Attributes
    // ----------------------------------------------------------------------------------------------------------

    pub fn attribute(&self, id: NodeId, namespace: Option<&str>, local: &str) -> Option<&str> {
        self.element(id)?.attributes.iter().find(|a| a.name.is(namespace, local)).map(|a| a.value.as_str())
    }

    pub fn attributes(&self, id: NodeId) -> &[Attribute] {
        self.element(id).map(|element| element.attributes.as_slice()).unwrap_or(&[])
    }

    /// Sets an attribute; an existing attribute keeps its position, a new one is added at the end.
    pub fn set_attribute(&mut self, id: NodeId, namespace: Option<&str>, local: &str, value: &str) {
        let Some(element) = self.element_mut(id) else { return };
        if let Some(attribute) = element.attributes.iter_mut().find(|a| a.name.is(namespace, local)) {
            attribute.value = value.to_string();
        } else {
            element.attributes.push(Attribute { name: QName::new(namespace, local), value: value.to_string() });
        }
    }

    /// Removes an attribute; returns whether it existed.
    pub fn remove_attribute(&mut self, id: NodeId, namespace: Option<&str>, local: &str) -> bool {
        let Some(element) = self.element_mut(id) else { return false };
        let before = element.attributes.len();
        element.attributes.retain(|a| !a.name.is(namespace, local));
        before != element.attributes.len()
    }

    /// Declares a namespace prefix on an element (lxml `nsmap` of a new element).
    pub fn declare_namespace(&mut self, id: NodeId, prefix: Option<&str>, uri: &str) {
        let Some(element) = self.element_mut(id) else { return };
        element.namespaces.retain(|d| d.prefix.as_deref() != prefix);
        element.namespaces.push(NamespaceDecl { prefix: prefix.map(str::to_string), uri: uri.to_string() });
    }

    // ----------------------------------------------------------------------------------------------------------
    // Text
    // ----------------------------------------------------------------------------------------------------------

    /// The text before the first child node that is not text (lxml `element.text`), None when there is none. For
    /// a text or comment node: its content.
    pub fn text(&self, id: NodeId) -> Option<String> {
        match &self.node(id).kind {
            NodeKind::Text(text) | NodeKind::Comment(text) => return Some(text.clone()),
            _ => {}
        }
        let mut result: Option<String> = None;
        for &child in &self.node(id).children {
            match &self.node(child).kind {
                NodeKind::Text(text) => result.get_or_insert_with(String::new).push_str(text),
                _ => break,
            }
        }
        result
    }

    /// The text of an element without surrounding whitespace; empty when there is none.
    pub fn text_trimmed(&self, id: NodeId) -> String {
        self.text(id).map(|t| t.trim().to_string()).unwrap_or_default()
    }

    /// Replaces the text before the first child element (lxml `element.text = value`). None removes it.
    pub fn set_text(&mut self, id: NodeId, value: Option<&str>) {
        if let NodeKind::Comment(text) = &mut self.node_mut(id).kind {
            *text = value.unwrap_or_default().to_string();
            return;
        }
        if !self.is_element(id) {
            return;
        }
        let leading: Vec<NodeId> = self
            .node(id)
            .children
            .iter()
            .copied()
            .take_while(|&child| matches!(self.node(child).kind, NodeKind::Text(_)))
            .collect();
        for child in leading {
            self.detach(child);
        }
        if let Some(value) = value {
            let text = self.push_node(NodeKind::Text(value.to_string()), None);
            self.node_mut(text).parent = Some(id);
            self.node_mut(id).children.insert(0, text);
        }
    }

    /// The text after the node up to the next sibling that is not text (lxml `element.tail`).
    pub fn tail(&self, id: NodeId) -> Option<String> {
        let parent = self.parent(id)?;
        let siblings = &self.node(parent).children;
        let position = siblings.iter().position(|&s| s == id)?;
        let mut result: Option<String> = None;
        for &sibling in &siblings[position + 1..] {
            match &self.node(sibling).kind {
                NodeKind::Text(text) => result.get_or_insert_with(String::new).push_str(text),
                _ => break,
            }
        }
        result
    }

    /// Replaces the text after the node (lxml `element.tail = value`). None removes it.
    pub fn set_tail(&mut self, id: NodeId, value: Option<&str>) {
        let Some(parent) = self.parent(id) else { return };
        let position = self.index_in_parent(id).expect("a child of its parent");
        let following: Vec<NodeId> = self.node(parent).children[position + 1..]
            .iter()
            .copied()
            .take_while(|&s| matches!(self.node(s).kind, NodeKind::Text(_)))
            .collect();
        for sibling in following {
            self.detach(sibling);
        }
        if let Some(value) = value {
            let text = self.push_node(NodeKind::Text(value.to_string()), None);
            self.node_mut(text).parent = Some(parent);
            self.node_mut(parent).children.insert(position + 1, text);
        }
    }

    fn append_text(&mut self, parent: NodeId, value: &str) {
        if let Some(&last) = self.node(parent).children.last() {
            if let NodeKind::Text(text) = &mut self.node_mut(last).kind {
                text.push_str(value);
                return;
            }
        }
        self.push_node(NodeKind::Text(value.to_string()), Some(parent));
    }

    // ----------------------------------------------------------------------------------------------------------
    // Changing the tree
    // ----------------------------------------------------------------------------------------------------------

    fn push_node(&mut self, kind: NodeKind, parent: Option<NodeId>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node { kind, parent, children: Vec::new() });
        if let Some(parent) = parent {
            self.node_mut(parent).children.push(id);
        }
        id
    }

    /// Creates an element that is not in the document yet.
    pub fn create_element(&mut self, namespace: Option<&str>, local: &str) -> NodeId {
        self.push_node(
            NodeKind::Element(Element { name: QName::new(namespace, local), namespaces: vec![], attributes: vec![] }),
            None,
        )
    }

    /// Creates an element at the end of the parent (lxml `SubElement`).
    pub fn append_element(&mut self, parent: NodeId, namespace: Option<&str>, local: &str) -> NodeId {
        let id = self.create_element(namespace, local);
        self.append_child(parent, id);
        id
    }

    /// Creates a comment node that is not in the document yet.
    pub fn create_comment(&mut self, text: &str) -> NodeId {
        self.push_node(NodeKind::Comment(text.to_string()), None)
    }

    /// Creates a text node that is not in the document yet.
    pub fn create_text(&mut self, text: &str) -> NodeId {
        self.push_node(NodeKind::Text(text.to_string()), None)
    }

    /// Moves a node to the end of the parent's children.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        self.detach(child);
        self.node_mut(child).parent = Some(parent);
        self.node_mut(parent).children.push(child);
    }

    /// Moves a node to a position among all child nodes of the parent.
    pub fn insert_child(&mut self, parent: NodeId, index: usize, child: NodeId) {
        self.detach(child);
        let index = index.min(self.node(parent).children.len());
        self.node_mut(child).parent = Some(parent);
        self.node_mut(parent).children.insert(index, child);
    }

    /// Moves a node directly before the reference node.
    pub fn insert_before(&mut self, reference: NodeId, child: NodeId) {
        self.detach(child);
        let parent = self.parent(reference).expect("the reference node has a parent");
        let index = self.index_in_parent(reference).expect("a child of its parent");
        self.node_mut(child).parent = Some(parent);
        self.node_mut(parent).children.insert(index, child);
    }

    /// Moves a node directly after the reference node.
    pub fn insert_after(&mut self, reference: NodeId, child: NodeId) {
        self.detach(child);
        let parent = self.parent(reference).expect("the reference node has a parent");
        let index = self.index_in_parent(reference).expect("a child of its parent");
        self.node_mut(child).parent = Some(parent);
        self.node_mut(parent).children.insert(index + 1, child);
    }

    /// Takes a node out of the document. The node and its children stay valid and can be inserted again.
    pub fn detach(&mut self, id: NodeId) {
        if let Some(parent) = self.node(id).parent {
            self.node_mut(parent).children.retain(|&child| child != id);
            self.node_mut(id).parent = None;
        }
    }

    /// Removes a node like lxml `parent.remove(element)`: its tail text goes with it.
    pub fn remove_with_tail(&mut self, id: NodeId) {
        self.set_tail(id, None);
        self.detach(id);
    }

    /// Copies a node with all its descendants; the copy is not in the document.
    pub fn deep_copy(&mut self, id: NodeId) -> NodeId {
        let copy = self.push_node(self.node(id).kind.clone(), None);
        let children = self.node(id).children.clone();
        for child in children {
            let child_copy = self.deep_copy(child);
            self.node_mut(child_copy).parent = Some(copy);
            self.node_mut(copy).children.push(child_copy);
        }
        copy
    }

    /// Copies a node of another document (with its descendants) into this document; the copy is not attached.
    pub fn import(&mut self, other: &Document, id: NodeId) -> NodeId {
        let copy = self.push_node(other.node(id).kind.clone(), None);
        for &child in &other.node(id).children {
            let child_copy = self.import(other, child);
            self.node_mut(child_copy).parent = Some(copy);
            self.node_mut(copy).children.push(child_copy);
        }
        copy
    }

    /// Re-indents the content of an element like lxml `etree.indent(element, space, level)`: whitespace-only text
    /// and tails inside the element are replaced, other text is kept. The tail of the element itself is untouched.
    pub fn indent(&mut self, id: NodeId, space: &str, level: usize) {
        if self.is_element(id) {
            self.indent_children(id, space, level);
        }
    }

    fn indent_children(&mut self, id: NodeId, space: &str, level: usize) {
        let elements: Vec<NodeId> = self
            .node(id)
            .children
            .iter()
            .copied()
            .filter(|&c| !matches!(self.node(c).kind, NodeKind::Text(_)))
            .collect();
        if elements.is_empty() {
            return;
        }
        let child_indent = format!("\n{}", space.repeat(level + 1));
        let closing_indent = format!("\n{}", space.repeat(level));
        if self.text(id).is_none_or(|t| t.trim().is_empty()) {
            self.set_text(id, Some(&child_indent));
        }
        for (position, &child) in elements.iter().enumerate() {
            if self.is_element(child) {
                self.indent_children(child, space, level + 1);
            }
            let whitespace = if position + 1 == elements.len() { &closing_indent } else { &child_indent };
            if self.tail(child).is_none_or(|t| t.trim().is_empty()) {
                self.set_tail(child, Some(whitespace));
            }
        }
    }
}

/// Depth-first iterator over a node and its descendants.
pub struct Descendants<'a> {
    document: &'a Document,
    stack: Vec<NodeId>,
}

impl Iterator for Descendants<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let id = self.stack.pop()?;
        self.stack.extend(self.document.node(id).children.iter().rev());
        Some(id)
    }
}

/// The encoding named in the XML declaration (`<?xml version="1.0" encoding="ISO-8859-1"?>`), if any.
fn declared_encoding(bytes: &[u8]) -> Option<&'static encoding_rs::Encoding> {
    let head = &bytes[..bytes.len().min(200)];
    let head = String::from_utf8_lossy(head);
    let declaration = head.strip_prefix("<?xml")?;
    let declaration = &declaration[..declaration.find("?>")?];
    let start = declaration.find("encoding")? + "encoding".len();
    let rest = declaration[start..].trim_start().strip_prefix('=')?.trim_start();
    let quote = rest.chars().next()?;
    let label = rest[1..].split(quote).next()?;
    encoding_rs::Encoding::for_label(label.trim().as_bytes())
}

fn prefix_of(raw_name: &str) -> Option<String> {
    raw_name.split_once(':').map(|(prefix, _)| prefix.to_string())
}

fn resolve_namespace(resolved: ResolveResult, raw_name: &str) -> Result<Option<String>, XmlError> {
    match resolved {
        ResolveResult::Bound(ns) => Ok(Some(String::from_utf8_lossy(ns.as_ref()).into_owned())),
        ResolveResult::Unbound => {
            if raw_name.starts_with("xml:") {
                Ok(Some(XML_NAMESPACE.to_string()))
            } else {
                Ok(None)
            }
        }
        ResolveResult::Unknown(prefix) => {
            let prefix = String::from_utf8_lossy(&prefix).into_owned();
            if prefix == "xml" {
                Ok(Some(XML_NAMESPACE.to_string()))
            } else {
                Err(XmlError::UnknownPrefix(prefix))
            }
        }
    }
}

/// Escapes text content like libxml2: `&`, `<`, `>` and carriage return.
fn escape_text(text: &str, out: &mut String) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\r' => out.push_str("&#13;"),
            _ => out.push(c),
        }
    }
}

/// Escapes an attribute value like libxml2.
fn escape_attribute(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            _ => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NS: &str = "urn:test";
    const DECL: &str = "<?xml version='1.0' encoding='UTF-8'?>\n";

    #[test]
    fn parse_and_write_keeps_prefixes_comments_and_whitespace() {
        let xml = format!(
            "{DECL}<!-- before --><t:a xmlns:t=\"urn:test\" t:x=\"1\">\n  <t:b>text &amp; more</t:b>\n  <!-- c --><t:c/>\n</t:a>"
        );
        let document = Document::parse(&xml).unwrap();
        assert_eq!(document.to_xml(), xml);
    }

    #[test]
    fn text_and_tail_follow_lxml() {
        let mut document = Document::parse("<a xmlns=\"urn:test\">x<b/>y<c/></a>").unwrap();
        let root = document.root().unwrap();
        let b = document.find(root, Some(NS), "b").unwrap();
        assert_eq!(document.text(root).as_deref(), Some("x"));
        assert_eq!(document.tail(b).as_deref(), Some("y"));
        document.set_text(root, Some("new"));
        document.set_tail(b, None);
        assert_eq!(document.to_xml(), format!("{DECL}<a xmlns=\"urn:test\">new<b/><c/></a>"));
    }

    #[test]
    fn new_elements_use_prefix_in_scope_or_get_a_declaration() {
        let mut document = Document::parse("<t:a xmlns:t=\"urn:test\"/>").unwrap();
        let root = document.root().unwrap();
        let b = document.append_element(root, Some(NS), "b");
        document.set_attribute(b, Some(NS), "id", "x");
        let c = document.append_element(root, Some("urn:other"), "c");
        document.set_text(c, Some(""));
        assert_eq!(
            document.to_xml(),
            format!("{DECL}<t:a xmlns:t=\"urn:test\"><t:b t:id=\"x\"/><ns0:c xmlns:ns0=\"urn:other\"></ns0:c></t:a>")
        );
    }

    #[test]
    fn element_to_xml_declares_used_namespaces_of_ancestors() {
        let document =
            Document::parse("<t:a xmlns:t=\"urn:test\" xmlns:u=\"urn:u\"><t:b><t:c t:x=\"1\"/></t:b></t:a>").unwrap();
        let b = document.find(document.root().unwrap(), Some(NS), "b").unwrap();
        assert_eq!(document.element_to_xml(b), "<t:b xmlns:t=\"urn:test\"><t:c t:x=\"1\"/></t:b>");
    }

    #[test]
    fn detached_node_keeps_its_id() {
        let mut document = Document::parse("<a><b/><c/></a>").unwrap();
        let root = document.root().unwrap();
        let b = document.find(root, None, "b").unwrap();
        document.detach(b);
        assert!(!document.is_attached(b));
        assert_eq!(document.child_elements(root).count(), 1);
        document.append_child(root, b);
        assert_eq!(document.to_xml(), format!("{DECL}<a><c/><b/></a>"));
    }

    #[test]
    fn indent_matches_lxml() {
        let mut document = Document::parse("<a><b><c/></b></a>").unwrap();
        let root = document.root().unwrap();
        let b = document.find(root, None, "b").unwrap();
        document.indent(b, "  ", 1);
        assert_eq!(document.element_to_xml(b), "<b>\n    <c/>\n  </b>");
    }

    #[test]
    fn escapes_attributes_and_text() {
        let mut document = Document::parse("<a/>").unwrap();
        let root = document.root().unwrap();
        document.set_attribute(root, None, "v", "a\"b\n<&>");
        document.set_text(root, Some("1 < 2 & 3 > 2\r"));
        assert_eq!(
            document.to_xml(),
            format!("{DECL}<a v=\"a&quot;b&#10;&lt;&amp;&gt;\">1 &lt; 2 &amp; 3 &gt; 2&#13;</a>")
        );
    }
}
