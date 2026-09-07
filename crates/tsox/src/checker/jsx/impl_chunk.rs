#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, NodeData, SyntaxKind, is_identifier, is_jsx_namespaced_name};

bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct JsxFlags: u32 {

        const INTRINSIC_NAMED_ELEMENT = 1 << 0;

        const INTRINSIC_INDEXED_ELEMENT = 1 << 1;
    }
}

impl JsxFlags {
    pub const INTRINSIC_ELEMENT: Self =
        Self::INTRINSIC_NAMED_ELEMENT.union(Self::INTRINSIC_INDEXED_ELEMENT);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxReferenceKind {
    Component,

    Function,

    Mixed,
}

pub struct JsxNames;
impl JsxNames {
    pub const JSX: &'static str = "JSX";
    pub const INTRINSIC_ELEMENTS: &'static str = "IntrinsicElements";
    pub const ELEMENT_CLASS: &'static str = "ElementClass";
    pub const ELEMENT_ATTRIBUTES_PROPERTY_NAME_CONTAINER: &'static str =
        "ElementAttributesProperty";
    pub const ELEMENT_CHILDREN_ATTRIBUTE_NAME_CONTAINER: &'static str = "ElementChildrenAttribute";
    pub const ELEMENT: &'static str = "Element";
    pub const ELEMENT_TYPE: &'static str = "ElementType";
    pub const INTRINSIC_ATTRIBUTES: &'static str = "IntrinsicAttributes";
    pub const INTRINSIC_CLASS_ATTRIBUTES: &'static str = "IntrinsicClassAttributes";
    pub const LIBRARY_MANAGED_ATTRIBUTES: &'static str = "LibraryManagedAttributes";
}

pub struct ReactNames;
impl ReactNames {
    pub const FRAGMENT: &'static str = "Fragment";
}

pub fn is_intrinsic_jsx_name(name: &str) -> bool {
    !name.is_empty() && (name.as_bytes()[0].is_ascii_lowercase() || name.contains('-'))
}

pub fn is_jsx_intrinsic_tag_name(tag_name: &Arc<Node>) -> bool {
    (is_identifier(tag_name) && is_intrinsic_jsx_name(tag_name.text()))
        || is_jsx_namespaced_name(tag_name)
}

pub fn jsx_tag_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::JsxOpeningElement(data) => Some(Arc::clone(&data.tag_name)),
        NodeData::JsxSelfClosingElement(data) => Some(Arc::clone(&data.tag_name)),
        NodeData::JsxClosingElement(data) => Some(Arc::clone(&data.tag_name)),
        _ => None,
    }
}

pub fn jsx_attributes(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::JsxOpeningElement(data) => Some(Arc::clone(&data.attributes)),
        NodeData::JsxSelfClosingElement(data) => Some(Arc::clone(&data.attributes)),
        _ => None,
    }
}

pub fn is_jsx_opening_like_element(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement
    )
}
