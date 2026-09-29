#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::mig::m3g::is_modifier;
use tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body;
use tsox_frontend::ast::node::{ModifierList, Node};
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{is_class_like, is_decorator};

use super::m4l::r33k9_defs::class_element_or_class_element_parameter_is_decorated;
use super::m4l::r39k20_defs::R39K20NodeFactoryExt;
use super::m4l_2::{MetadataTransformer, USE_NEW_TYPE_METADATA_FORMAT};
use super::m4m::MetadataSerializerContext;

impl<'a> MetadataTransformer<'a> {
    pub fn inject_class_type_metadata(
        &mut self,
        list: Option<Arc<ModifierList>>,
        node: &Arc<Node>,
    ) -> Option<Arc<ModifierList>> {
        let metadata = self.get_type_metadata(node, node);
        if !metadata.is_empty() {
            let list = list
                .unwrap_or_else(|| Arc::new(ModifierList::new(Vec::new(), ModifierFlags::empty())));
            let original_nodes = &list.list.nodes;
            if original_nodes.is_empty() {
                let mut res = self.factory().new_modifier_list(metadata);
                if let Some(res_node) = Arc::get_mut(&mut res) {
                    res_node.list.loc = list.list.loc;
                }
                return Some(res);
            }
            let mut modifiers_array: Vec<Arc<Node>> = Vec::new();
            if is_modifier(&original_nodes[0])
                && (original_nodes[0].kind == SyntaxKind::DefaultKeyword
                    || original_nodes[0].kind == SyntaxKind::ExportKeyword)
            {
                modifiers_array.push(Arc::clone(&original_nodes[0]));
                if original_nodes.len() > 1
                    && (original_nodes[1].kind == SyntaxKind::DefaultKeyword
                        || original_nodes[1].kind == SyntaxKind::ExportKeyword)
                {
                    modifiers_array.push(Arc::clone(&original_nodes[1]));
                }
            }
            let rest_start = modifiers_array.len();
            let decos: Vec<Arc<Node>> = original_nodes
                .iter()
                .filter(|n| is_decorator(n))
                .cloned()
                .collect();
            modifiers_array.extend(decos);
            modifiers_array.extend(metadata);
            let other_modifiers: Vec<Arc<Node>> = original_nodes[rest_start..]
                .iter()
                .filter(|n| is_modifier(n))
                .cloned()
                .collect();
            modifiers_array.extend(other_modifiers);
            let mut res = self.factory().new_modifier_list(modifiers_array);
            if let Some(res_node) = Arc::get_mut(&mut res) {
                res_node.list.loc = list.list.loc;
            }
            return Some(res);
        }
        list
    }

    pub fn inject_class_element_type_metadata(
        &mut self,
        list: Option<Arc<ModifierList>>,
        node: &Arc<Node>,
        container: Option<&Arc<Node>>,
    ) -> Option<Arc<ModifierList>> {
        let Some(container) = container else {
            return list;
        };
        if !is_class_like(container) {
            return list;
        }
        if !class_element_or_class_element_parameter_is_decorated(
            self.legacy_decorators,
            node,
            container,
        ) {
            return list;
        }
        let metadata = self.get_type_metadata(node, container);
        if !metadata.is_empty() {
            let list = list
                .unwrap_or_else(|| Arc::new(ModifierList::new(Vec::new(), ModifierFlags::empty())));
            let original_nodes = &list.list.nodes;
            if original_nodes.is_empty() {
                let mut res = self.factory().new_modifier_list(metadata);
                if let Some(res_node) = Arc::get_mut(&mut res) {
                    res_node.list.loc = list.list.loc;
                }
                return Some(res);
            }
            let mut modifiers_array: Vec<Arc<Node>> = Vec::new();
            let decos: Vec<Arc<Node>> = original_nodes
                .iter()
                .filter(|n| is_decorator(n))
                .cloned()
                .collect();
            modifiers_array.extend(decos);
            modifiers_array.extend(metadata);
            let other_modifiers: Vec<Arc<Node>> = original_nodes
                .iter()
                .filter(|n| is_modifier(n))
                .cloned()
                .collect();
            modifiers_array.extend(other_modifiers);
            let mut res = self.factory().new_modifier_list(modifiers_array);
            if let Some(res_node) = Arc::get_mut(&mut res) {
                res_node.list.loc = list.list.loc;
            }
            return Some(res);
        }
        list
    }

    pub fn get_type_metadata(
        &mut self,
        node: &Arc<Node>,
        container: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        if !self.legacy_decorators {
            return Vec::new();
        }
        if USE_NEW_TYPE_METADATA_FORMAT {
            return self.get_new_type_metadata(node, container);
        }
        self.get_old_type_metadata(node, container)
    }

    pub fn get_old_type_metadata(
        &mut self,
        node: &Arc<Node>,
        container: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        let mut decorators: Vec<Arc<Node>> = Vec::new();
        if self.should_add_type_metadata(node) {
            let serialized = self
                .serializer
                .as_mut()
                .unwrap()
                .serialize_type_of_node(
                    MetadataSerializerContext {
                        current_lexical_scope: self.current_lexical_scope.clone(),
                        current_name_scope: Some(Arc::clone(container)),
                        serializing_conditional_type_branch: false,
                    },
                    node,
                    Some(container),
                )
                .unwrap();
            let type_metadata = self.factory().new_metadata_helper("design:type", &serialized);
            decorators.push(self.factory().new_decorator(type_metadata));
        }
        if self.should_add_param_types_metadata(node) {
            let serialized = self
                .serializer
                .as_mut()
                .unwrap()
                .serialize_parameter_types_of_node(
                    MetadataSerializerContext {
                        current_lexical_scope: self.current_lexical_scope.clone(),
                        current_name_scope: Some(Arc::clone(container)),
                        serializing_conditional_type_branch: false,
                    },
                    node,
                    Some(container),
                )
                .unwrap();
            let param_types_metadata = self
                .factory()
                .new_metadata_helper("design:paramtypes", &serialized);
            decorators.push(self.factory().new_decorator(param_types_metadata));
        }
        if self.should_add_return_type_metadata(node) {
            let serialized = self
                .serializer
                .as_mut()
                .unwrap()
                .serialize_return_type_of_node(
                    MetadataSerializerContext {
                        current_lexical_scope: self.current_lexical_scope.clone(),
                        current_name_scope: Some(Arc::clone(container)),
                        serializing_conditional_type_branch: false,
                    },
                    node,
                )
                .unwrap();
            let return_type_metadata = self
                .factory()
                .new_metadata_helper("design:returntype", &serialized);
            decorators.push(self.factory().new_decorator(return_type_metadata));
        }
        decorators
    }

    pub fn get_new_type_metadata(
        &mut self,
        node: &Arc<Node>,
        container: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        let mut properties: Vec<Arc<Node>> = Vec::new();
        if self.should_add_type_metadata(node) {
            let serialized = self
                .serializer
                .as_mut()
                .unwrap()
                .serialize_type_of_node(
                    MetadataSerializerContext {
                        current_lexical_scope: self.current_lexical_scope.clone(),
                        current_name_scope: Some(Arc::clone(container)),
                        serializing_conditional_type_branch: false,
                    },
                    node,
                    Some(container),
                )
                .unwrap();
            let name = self.factory().new_identifier("type");
            let body = self.factory().new_arrow_function(
                None,
                None,
                &self.factory().new_node_list(Vec::new()),
                None,
                None,
                &self.factory().new_token(SyntaxKind::EqualsGreaterThanToken),
                &serialized,
            );
            properties.push(self.factory().new_property_assignment(
                None,
                &name,
                None,
                None,
                &body,
            ));
        }
        if self.should_add_param_types_metadata(node) {
            let serialized = self
                .serializer
                .as_mut()
                .unwrap()
                .serialize_parameter_types_of_node(
                    MetadataSerializerContext {
                        current_lexical_scope: self.current_lexical_scope.clone(),
                        current_name_scope: Some(Arc::clone(container)),
                        serializing_conditional_type_branch: false,
                    },
                    node,
                    Some(container),
                )
                .unwrap();
            let name = self.factory().new_identifier("paramTypes");
            let body = self.factory().new_arrow_function(
                None,
                None,
                &self.factory().new_node_list(Vec::new()),
                None,
                None,
                &self.factory().new_token(SyntaxKind::EqualsGreaterThanToken),
                &serialized,
            );
            properties.push(self.factory().new_property_assignment(
                None,
                &name,
                None,
                None,
                &body,
            ));
        }
        if self.should_add_return_type_metadata(node) {
            let serialized = self
                .serializer
                .as_mut()
                .unwrap()
                .serialize_return_type_of_node(
                    MetadataSerializerContext {
                        current_lexical_scope: self.current_lexical_scope.clone(),
                        current_name_scope: Some(Arc::clone(container)),
                        serializing_conditional_type_branch: false,
                    },
                    node,
                )
                .unwrap();
            let name = self.factory().new_identifier("returnType");
            let body = self.factory().new_arrow_function(
                None,
                None,
                &self.factory().new_node_list(Vec::new()),
                None,
                None,
                &self.factory().new_token(SyntaxKind::EqualsGreaterThanToken),
                &serialized,
            );
            properties.push(self.factory().new_property_assignment(
                None,
                &name,
                None,
                None,
                &body,
            ));
        }
        if !properties.is_empty() {
            let type_info_object = self.factory().new_object_literal_expression(
                &self.factory().new_node_list(properties),
                true,
            );
            let type_info_metadata = self
                .factory()
                .new_metadata_helper("design:typeinfo", &type_info_object);
            return vec![self.factory().new_decorator(type_info_metadata)];
        }
        Vec::new()
    }

    pub fn should_add_type_metadata(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::PropertyDeclaration
        )
    }

    pub fn should_add_return_type_metadata(&self, node: &Arc<Node>) -> bool {
        node.kind == SyntaxKind::MethodDeclaration
    }

    pub fn should_add_param_types_metadata(&self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                get_first_constructor_with_body(node).is_some()
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => true,
            _ => false,
        }
    }
}