#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_jsx_element_properties_name(
        &self,
        _jsx_namespace: &Arc<crate::ast::Symbol>,
    ) -> Option<String> {
        None
    }

    pub fn get_jsx_element_children_property_name(
        &self,
        _jsx_namespace: &Arc<crate::ast::Symbol>,
    ) -> Option<String> {
        None
    }

    pub fn get_name_from_jsx_element_attributes_container(
        &self,
        _name_of_attrib_prop_container: &str,
        _jsx_namespace: &Arc<crate::ast::Symbol>,
    ) -> Option<String> {
        None
    }

    pub fn get_static_type_of_referenced_jsx_constructor(
        &mut self,
        _context: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_intrinsic_attributes_type_from_string_literal_type(
        &mut self,
        _t: &Arc<crate::checker::types::Type>,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_reference_kind(&self, _node: &Arc<Node>) -> JsxReferenceKind {
        JsxReferenceKind::Function
    }

    pub fn create_signature_for_jsx_intrinsic(
        &mut self,
        _node: &Arc<Node>,
        _result: &Arc<crate::checker::types::Type>,
    ) -> Option<Arc<crate::checker::types::Signature>> {
        None
    }

    pub fn get_intrinsic_attributes_type_from_jsx_opening_like_element(
        &mut self,
        _node: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_intrinsic_tag_symbol(&self, _node: &Arc<Node>) -> Option<Arc<crate::ast::Symbol>> {
        None
    }

    pub fn get_jsx_stateless_element_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_element_class_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_element_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_element_type_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_namespace_str(&self, _location: &Arc<Node>) -> String {
        "jsx".to_string()
    }

    pub fn get_local_jsx_namespace(&self, _file: &Arc<crate::ast::SourceFile>) -> String {
        "jsx".to_string()
    }

    pub fn get_jsx_factory_entity(&self, _location: &Arc<Node>) -> Option<Arc<Node>> {
        None
    }

    pub fn get_jsx_fragment_factory_entity(&self, _location: &Arc<Node>) -> Option<Arc<Node>> {
        None
    }

    pub fn get_jsx_namespace_container_for_implicit_import(
        &self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::ast::Symbol>> {
        None
    }

    pub fn get_jsx_runtime_import_specifier(
        &self,
        _file: &Arc<crate::ast::SourceFile>,
    ) -> (String, Option<Arc<Node>>) {
        (String::new(), None)
    }
}
