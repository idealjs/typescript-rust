use crate::ast::*;
use std::sync::Arc;

pub trait ReferenceResolver {
    fn get_referenced_export_container(
        &self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>>;

    fn get_referenced_import_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;

    fn get_referenced_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;

    fn get_referenced_value_declarations(&self, node: &Arc<Node>) -> Vec<Arc<Node>>;

    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String;

    fn get_referenced_member_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
}
