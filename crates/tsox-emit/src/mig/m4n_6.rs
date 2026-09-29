#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{is_identifier, is_private_identifier};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::scanner::TokenFlags;

use crate::mig::m4n_5::r34k4_helpers::{es_decorate_helper, export_star_helper};
use crate::printer::NodeFactory;
use tsox_frontend::format::mig::m4o::EmitFlags;

impl<'a> NodeFactory<'a> {
    pub fn new_es_decorate_class_context_object(
        &self,
        name_expr: &Arc<Node>,
        metadata: &Arc<Node>,
    ) -> Arc<Node> {
        let props = vec![
            self.new_property_assignment(
                None,
                &self.new_identifier("kind"),
                None,
                None,
                &self.new_string_literal("class", 0),
            ),
            self.new_property_assignment(
                None,
                &self.new_identifier("name"),
                None,
                None,
                name_expr,
            ),
            self.new_property_assignment(
                None,
                &self.new_identifier("metadata"),
                None,
                None,
                metadata,
            ),
        ];
        self.new_object_literal_expression(&self.new_node_list(props), false)
    }

    pub fn new_es_decorate_class_element_access_get_method(
        &self,
        name_computed: bool,
        name_expr: &Arc<Node>,
    ) -> Arc<Node> {
        let accessor = if name_computed {
            self.new_element_access_expression(
                &self.new_identifier("obj"),
                None,
                name_expr,
                NodeFlags::empty(),
            )
        } else {
            self.new_property_access_expression(
                &self.new_identifier("obj"),
                None,
                name_expr,
                NodeFlags::empty(),
            )
        };

        let obj_param =
            self.new_parameter_declaration(None, None, &self.new_identifier("obj"), None, None, None);

        let arrow = self.new_arrow_function(
            None,
            None,
            &self.new_node_list(vec![obj_param]),
            None,
            None,
            &self.new_token(SyntaxKind::EqualsGreaterThanToken),
            &accessor,
        );

        self.new_property_assignment(
            None,
            &self.new_identifier("get"),
            None,
            None,
            &arrow,
        )
    }

    pub fn new_es_decorate_class_element_access_has_method(
        &self,
        name_computed: bool,
        name_expr: &Arc<Node>,
    ) -> Arc<Node> {
        let property_name = if !name_computed && is_identifier(name_expr) {
            self.new_string_literal_from_node(name_expr)
        } else {
            Arc::clone(name_expr)
        };

        let obj_param =
            self.new_parameter_declaration(None, None, &self.new_identifier("obj"), None, None, None);
        let in_expr = self.new_binary_expression(
            None,
            &property_name,
            None,
            &self.new_token(SyntaxKind::InKeyword),
            &self.new_identifier("obj"),
        );

        let arrow = self.new_arrow_function(
            None,
            None,
            &self.new_node_list(vec![obj_param]),
            None,
            None,
            &self.new_token(SyntaxKind::EqualsGreaterThanToken),
            &in_expr,
        );

        self.new_property_assignment(
            None,
            &self.new_identifier("has"),
            None,
            None,
            &arrow,
        )
    }

    pub fn new_es_decorate_class_element_access_object(
        &self,
        name_computed: bool,
        name_expr: &Arc<Node>,
        has_get: bool,
        has_set: bool,
    ) -> Arc<Node> {
        let mut access_props = vec![self.new_es_decorate_class_element_access_has_method(
            name_computed,
            name_expr,
        )];

        if has_get {
            access_props.push(self.new_es_decorate_class_element_access_get_method(
                name_computed,
                name_expr,
            ));
        }

        if has_set {
            access_props.push(self.new_es_decorate_class_element_access_set_method(
                name_computed,
                name_expr,
            ));
        }

        self.new_object_literal_expression(&self.new_node_list(access_props), false)
    }

    pub fn new_es_decorate_class_element_access_set_method(
        &self,
        name_computed: bool,
        name_expr: &Arc<Node>,
    ) -> Arc<Node> {
        let accessor = if name_computed {
            self.new_element_access_expression(
                &self.new_identifier("obj"),
                None,
                name_expr,
                NodeFlags::empty(),
            )
        } else {
            self.new_property_access_expression(
                &self.new_identifier("obj"),
                None,
                name_expr,
                NodeFlags::empty(),
            )
        };

        let assignment =
            self.new_assignment_expression(&accessor, &self.new_identifier("value"));
        let stmt = self.new_expression_statement(&assignment);
        let body = self.new_block(&self.new_node_list(vec![stmt]), false);

        let obj_param =
            self.new_parameter_declaration(None, None, &self.new_identifier("obj"), None, None, None);
        let value_param = self
            .new_parameter_declaration(None, None, &self.new_identifier("value"), None, None, None);

        let arrow = self.new_arrow_function(
            None,
            None,
            &self.new_node_list(vec![obj_param, value_param]),
            None,
            None,
            &self.new_token(SyntaxKind::EqualsGreaterThanToken),
            &body,
        );

        self.new_property_assignment(
            None,
            &self.new_identifier("set"),
            None,
            None,
            &arrow,
        )
    }

    pub fn new_es_decorate_class_element_context_object(
        &self,
        kind: &str,
        name_computed: bool,
        name_expr: &Arc<Node>,
        is_static: bool,
        is_private: bool,
        has_get: bool,
        has_set: bool,
        metadata: &Arc<Node>,
    ) -> Arc<Node> {
        let name_value = if !name_computed && (is_private_identifier(name_expr) || is_identifier(name_expr)) {
            self.new_string_literal_from_node(name_expr)
        } else {
            Arc::clone(name_expr)
        };

        let access_obj =
            self.new_es_decorate_class_element_access_object(name_computed, name_expr, has_get, has_set);

        let static_expr = if is_static {
            self.new_keyword_expression(SyntaxKind::TrueKeyword)
        } else {
            self.new_false_expression()
        };

        let private_expr = if is_private {
            self.new_keyword_expression(SyntaxKind::TrueKeyword)
        } else {
            self.new_false_expression()
        };

        let props = vec![
            self.new_property_assignment(
                None,
                &self.new_identifier("kind"),
                None,
                None,
                &self.new_string_literal(kind, 0),
            ),
            self.new_property_assignment(
                None,
                &self.new_identifier("name"),
                None,
                None,
                &name_value,
            ),
            self.new_property_assignment(
                None,
                &self.new_identifier("static"),
                None,
                None,
                &static_expr,
            ),
            self.new_property_assignment(
                None,
                &self.new_identifier("private"),
                None,
                None,
                &private_expr,
            ),
            self.new_property_assignment(
                None,
                &self.new_identifier("access"),
                None,
                None,
                &access_obj,
            ),
            self.new_property_assignment(
                None,
                &self.new_identifier("metadata"),
                None,
                None,
                metadata,
            ),
        ];
        self.new_object_literal_expression(&self.new_node_list(props), false)
    }
}
