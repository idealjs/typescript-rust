#![allow(unused_imports)]

use crate::checker::jsx_impl_chunk_2::*;

impl Checker {
    pub fn check_jsx_opening_like_element(&mut self, opening: &Arc<Node>) {
        let is_opening_like = is_jsx_opening_like_element(opening);
        if is_opening_like {
            self.check_grammar_jsx_element(opening);
        }
        self.check_jsx_preconditions(opening);
        self.mark_jsx_alias_referenced(opening);

        if is_opening_like
            && matches!(
                self.compiler_options.jsx,
                tsox_core::core::compiler_options::JsxEmit::React
            )
            && let Some(tag) = jsx_tag_name(opening)
        {
            let default_name = self.compiler_options.react_namespace.as_str();
            let default_name = if default_name.is_empty() {
                "React"
            } else {
                default_name
            };
            let option_factory = self
                .compiler_options
                .jsx_factory
                .split('.')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or(default_name)
                .to_string();
            let factory_name = self
                .local_jsx_pragma_factory("jsx")
                .and_then(|f| f.split('.').next().map(str::to_string))
                .filter(|s| !s.is_empty())
                .unwrap_or(option_factory);
            if !self.jsx_factory_namespace_in_scope(&factory_name) {
                self.grammar_error_on_node_with_args(
                    &tag,
                    &THIS_JSX_TAG_REQUIRES_0_TO_BE_IN_SCOPE_BUT_IT_COULD_NOT_BE_FOUND,
                    &[factory_name.to_string()],
                );
            }
        }
        if !is_opening_like {
            return;
        }
        // Go jsxElementLinks 惰性解析命名空间：先建隐式容器缓存，标签检查
        // 才能命中 jsxFactory 派生的 JSX 命名空间
        self.ensure_jsx_implicit_container(opening);
        let tag_name = match jsx_tag_name(opening) {
            Some(t) => t,
            None => {
                if matches!(opening.kind, tsox_frontend::ast::SyntaxKind::JsxOpeningFragment) {
                    self.check_jsx_element_props(opening);
                }
                return;
            }
        };
        if is_jsx_intrinsic_tag_name(&tag_name) {
            self.check_jsx_intrinsic_element(opening);
        } else {
            self.check_jsx_component(opening);
        }
        self.check_jsx_element_props(opening);

        if let Some((loc, module_ref)) = self.pending_jsx_2875.take() {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                THIS_JSX_TAG_REQUIRES_THE_MODULE_PATH_0_TO_EXIST_BUT_NONE_COULD_BE_FOUND_MAKE_SURE_YOU_HAVE_TYPES_FOR_THE_APPROPRIATE_PACKAGE_INSTALLED,
                vec![module_ref],
            ));
        }
    }

    pub fn check_jsx_element_deferred(&mut self, _node: &Arc<Node>) {}

    pub fn check_jsx_expression(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: u32,
    ) -> Arc<crate::checker::types::Type> {
        self.any_type()
    }

    pub fn check_jsx_self_closing_element(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: u32,
    ) -> Arc<crate::checker::types::Type> {
        self.any_type()
    }

    pub fn check_jsx_self_closing_element_deferred(&mut self, _node: &Arc<Node>) {}

    pub fn check_jsx_fragment(&mut self, _node: &Arc<Node>) -> Arc<crate::checker::types::Type> {
        self.any_type()
    }

    pub fn check_jsx_attributes(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: u32,
    ) -> Arc<crate::checker::types::Type> {
        self.any_type()
    }

    pub fn check_jsx_return_assignable_to_appropriate_bound(
        &mut self,
        _ref_kind: JsxReferenceKind,
        _elem_instance_type: &Arc<crate::checker::types::Type>,
        _opening_like_element: &Arc<Node>,
    ) {
    }

    pub fn infer_jsx_type_arguments(
        &mut self,
        _node: &Arc<Node>,
        _signature: &Arc<crate::checker::types::Signature>,
        _check_mode: u32,
        _context: &crate::checker::inference::InferenceContext,
    ) -> Vec<Arc<crate::checker::types::Type>> {
        Vec::new()
    }

    pub fn get_contextual_type_for_jsx_expression(
        &mut self,
        node: &Arc<Node>,
        _context_flags: crate::checker::types::ContextFlags,
    ) -> Option<Arc<crate::checker::types::Type>> {
        let jsx_expr = node.parent()?;
        if jsx_expr.kind != SyntaxKind::JsxExpression {
            return None;
        }
        let parent = jsx_expr.parent()?;
        if parent.kind == SyntaxKind::JsxElement {
            return self.get_contextual_type_for_child_jsx_expression(
                &parent,
                &jsx_expr,
                _context_flags,
            );
        }
        let attr = parent;
        if attr.kind != SyntaxKind::JsxAttribute {
            return None;
        }
        let attrs = attr.parent()?;
        let elem = attrs.parent()?;
        let name = attr.name().map(|n| n.text().to_string())?;
        let ct = self.jsx_element_attributes_contextual_type(&elem)?;
        self.get_type_of_property_of_contextual_type(&ct, &name)
    }

    pub fn get_contextual_type_for_jsx_attribute(
        &mut self,
        _attribute: &Arc<Node>,
        _context_flags: crate::checker::types::ContextFlags,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_contextual_jsx_element_attributes_type(
        &mut self,
        _node: &Arc<Node>,
        _context_flags: crate::checker::types::ContextFlags,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_contextual_type_for_child_jsx_expression(
        &mut self,
        node: &Arc<Node>,
        child: &Arc<Node>,
        _context_flags: crate::checker::types::ContextFlags,
    ) -> Option<Arc<crate::checker::types::Type>> {
        use crate::checker::types::TypeFlags;
        // Go getContextualTypeForChildJsxExpression：属性类型的 children 成员
        // （名字来自 JSX.ElementChildrenAttribute）即 children 位上下文
        let opening = match &node.data {
            tsox_frontend::ast::NodeData::JsxElement(d) => Arc::clone(&d.opening_element),
            _ => return None,
        };
        let attributes_type = self.jsx_element_attributes_contextual_type(&opening)?;
        if attributes_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let children_name = self
            .get_jsx_namespace()
            .and_then(|ns| self.get_jsx_element_children_property_name(&ns))?;
        if children_name.is_empty() {
            return None;
        }
        let child_field_type = self
            .get_type_of_property_of_contextual_type(&attributes_type, &children_name)?;
        let children = match &node.data {
            tsox_frontend::ast::NodeData::JsxElement(d) => Arc::clone(&d.children),
            _ => return None,
        };
        let semantic: Vec<Arc<Node>> = children
            .iter()
            .filter(|c| match &c.data {
                tsox_frontend::ast::NodeData::JsxExpression(je) => je.expression.is_some(),
                tsox_frontend::ast::NodeData::JsxText(t) => !t.contains_only_trivia_white_spaces,
                _ => true,
            })
            .cloned()
            .collect();
        if semantic.len() <= 1 {
            return Some(child_field_type);
        }
        let child_index = semantic
            .iter()
            .position(|c| Arc::ptr_eq(c, child))?;
        let index_type = self.get_number_literal_type(tsox_core::jsnum::Number::from(
            child_index as f64,
        ));
        let map_one = |checker: &mut Self, t: &Arc<crate::checker::types::Type>| {
            if checker.is_array_like_type(t) {
                checker.get_indexed_access_type(t, &index_type)
            } else {
                Arc::clone(t)
            }
        };
        if let Some(types) = child_field_type.types() {
            let mapped: Vec<Arc<crate::checker::types::Type>> =
                types.iter().map(|t| map_one(self, t)).collect();
            return Some(self.get_union_type(mapped));
        }
        Some(map_one(self, &child_field_type))
    }

    pub fn discriminate_contextual_type_by_jsx_attributes(
        &mut self,
        _node: &Arc<Node>,
        contextual_type: &Arc<crate::checker::types::Type>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        let _ = contextual_type;
        None
    }

    pub fn elaborate_jsx_components(
        &mut self,
        _node: &Arc<Node>,
        _source: &Arc<crate::checker::types::Type>,
        _target: &Arc<crate::checker::types::Type>,
        _relation: crate::checker::relater::RelationKind,
        _diagnostic_output: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        false
    }

    pub fn get_suggested_symbol_for_nonexistent_jsx_attribute(
        &mut self,
        _name: &str,
        _containing_type: &Arc<crate::checker::types::Type>,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> {
        None
    }

    pub fn get_jsx_fragment_type(&mut self, _node: &Arc<Node>) -> Arc<crate::checker::types::Type> {
        self.any_type()
    }

    pub fn resolve_jsx_opening_like_element(
        &mut self,
        _node: &Arc<Node>,
        _candidates_out_array: Option<&mut Vec<Arc<crate::checker::types::Signature>>>,
        _check_mode: u32,
    ) -> Option<Arc<crate::checker::types::Signature>> {
        None
    }

    pub fn check_applicable_signature_for_jsx_call_like_element(
        &mut self,
        _node: &Arc<Node>,
        _signature: &Arc<crate::checker::types::Signature>,
        _relation: crate::checker::relater::RelationKind,
        _check_mode: u32,
        _report_errors: bool,
        _diagnostic_output: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        false
    }

    pub fn create_jsx_attributes_type_from_attributes_property(
        &mut self,
        _opening_like_element: &Arc<Node>,
        _check_mode: u32,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn check_jsx_children(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: u32,
    ) -> Vec<Arc<crate::checker::types::Type>> {
        Vec::new()
    }

    pub fn get_uninstantiated_jsx_signatures_of_type(
        &mut self,
        _element_type: &Arc<crate::checker::types::Type>,
        _caller: &Arc<Node>,
    ) -> Vec<Arc<crate::checker::types::Signature>> {
        Vec::new()
    }

    pub fn get_effective_first_argument_for_jsx_signature(
        &mut self,
        _signature: &Arc<crate::checker::types::Signature>,
        _node: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_props_type_from_call_signature(
        &mut self,
        _sig: &Arc<crate::checker::types::Signature>,
        _context: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_props_type_from_class_type(
        &mut self,
        _sig: &Arc<crate::checker::types::Signature>,
        _context: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_props_type_for_signature_from_member(
        &mut self,
        _sig: &Arc<crate::checker::types::Signature>,
        _forced_lookup_location: &str,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_managed_attributes_from_located_attributes(
        &mut self,
        _context: &Arc<Node>,
        _ns: &Arc<tsox_frontend::ast::Symbol>,
        _attributes_type: &Arc<crate::checker::types::Type>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn instantiate_alias_or_interface_with_defaults(
        &mut self,
        _managed_sym: &Arc<tsox_frontend::ast::Symbol>,
        _type_arguments: &[Arc<crate::checker::types::Type>],
        _in_java_script: bool,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_library_managed_attributes(
        &self,
        _jsx_namespace: &Arc<tsox_frontend::ast::Symbol>,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> {
        None
    }

    pub fn get_jsx_element_type_symbol(
        &self,
        _jsx_namespace: &Arc<tsox_frontend::ast::Symbol>,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> {
        None
    }
}
