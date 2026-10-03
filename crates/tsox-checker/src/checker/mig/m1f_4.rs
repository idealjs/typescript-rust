#![allow(unused_imports)]
use crate::checker::mig::m1f::r19k9_defs::R19K9NodeExt;
use super::r18k6_defs::InternalSymbolName;
use crate::checker::mig::wc3::NodeAccessExt;

#[path = "r22k5_defs.rs"]
pub mod r22k5_defs;

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
pub(crate) use super::m1f::*;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3e_4::{get_import_attributes, get_this_container};
#[allow(unused_imports)]
use crate::checker::checker_this_container::get_this_parameter;
#[allow(unused_imports)]
use crate::checker::mig::wc2_2::get_module_specifier_from_node;
#[allow(unused_imports)]
use crate::checker::mig::wc3_2::is_contained_by_namespace;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::x8ast::module_export_name_is_default;
#[allow(unused_imports)]
use crate::checker::mig::w9a::new_type_mapper;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::x4ast::get_external_module_import_equals_declaration_expression;
use crate::checker::mig::m2a::r18k8_flags::{
    MODULE_KIND_COMMON_JS, MODULE_KIND_ES_NEXT, MODULE_KIND_NODE20, MODULE_KIND_NODE_NEXT,
};
use std::sync::Arc;

impl Checker {
    pub fn get_target_of_access_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_access_expression"); 
        let parent = node.parent_rc();
        if parent.kind == SyntaxKind::BinaryExpression {
            let expr = parent.as_binary_expression();
            if expr.left.id() == node.id() && expr.operator_token.kind == SyntaxKind::EqualsToken {
                return self.get_target_of_alias_like_expression(&expr.right);
            }
        }
        None
    }

    pub fn get_target_of_alias_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_alias_declaration"); 
        match node.kind {
            SyntaxKind::ImportEqualsDeclaration | SyntaxKind::VariableDeclaration => {
                self.get_target_of_import_equals_declaration(node)
            }
            SyntaxKind::ImportClause => self.get_target_of_import_clause(node),
            SyntaxKind::NamespaceImport => self.get_target_of_namespace_import(node),
            SyntaxKind::NamespaceExport => self.get_target_of_namespace_export(node),
            SyntaxKind::ImportSpecifier | SyntaxKind::BindingElement => {
                self.get_target_of_import_specifier(node)
            }
            SyntaxKind::ExportSpecifier => self.get_target_of_export_specifier(
                node,
                SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
                true,
            ),
            SyntaxKind::ExportAssignment => self.get_target_of_export_assignment(node),
            SyntaxKind::BinaryExpression => self.get_target_of_binary_expression(node),
            SyntaxKind::NamespaceExportDeclaration => {
                self.get_target_of_namespace_export_declaration(node)
            }
            SyntaxKind::ShorthandPropertyAssignment => self.resolve_entity_name(
                &node.as_shorthand_property_assignment().name,
                SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
                true,
                true,
                None,
            ),
            SyntaxKind::PropertyAssignment => {
                self.get_target_of_alias_like_expression(node.initializer()?)
            }
            SyntaxKind::ElementAccessExpression | SyntaxKind::PropertyAccessExpression => {
                self.get_target_of_access_expression(node)
            }
            _ => None,
        }
    }

    pub fn get_target_of_alias_like_expression(&mut self, expression: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_alias_like_expression"); 
        if is_class_expression(expression) {
            return self.check_expression_cached(expression).symbol().cloned();
        }
        if !is_entity_name(expression) && !is_entity_name_expression(expression) {
            return None;
        }
        let alias_like = self.resolve_entity_name(
            expression,
            SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
            true,
            true,
            None,
        );
        if let Some(alias_like) = alias_like {
            return Some(alias_like);
        }
        self.check_expression_cached(expression);
        self.get_resolved_symbol_nil(expression)
    }

    pub fn get_target_of_export_assignment(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_export_assignment"); 
        if is_contained_by_namespace(node) {
            return None;
        }
        let resolved = self.get_target_of_alias_like_expression(node.expression().unwrap());
        self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
        resolved
    }

    pub fn get_target_of_export_specifier(
        &mut self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
        dont_resolve_alias: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_export_specifier"); 
        let name = node.property_name_or_name();
        if module_export_name_is_default(&name) {
            if let Some(specifier) = self.get_module_specifier_for_import_or_export(node) {
                let attributes = get_import_attributes(&node.parent_rc().parent_rc());
                let import_attributes_type = self.get_type_from_import_attributes(attributes.as_ref());
                if let Some(module_symbol) = self.resolve_external_module_name_worker(
                    node,
                    Some(&specifier),
                    None,
                    false,
                    false,
                    import_attributes_type.as_ref(),
                ) {
                    return self.get_target_of_module_default(&module_symbol, node, dont_resolve_alias);
                }
            }
        }
        let export_declaration = node.parent_rc().parent_rc();
        let resolved = if get_module_specifier_from_node(&export_declaration).is_some() {
            self.get_external_module_member(&export_declaration, node, dont_resolve_alias)
        } else if is_string_literal(&name) {
            None
        } else {
            self.resolve_entity_name(&name, meaning, false, dont_resolve_alias, None)
        };
        self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
        resolved
    }

    pub fn get_target_of_import_clause(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_import_clause"); 
        let attributes = get_import_attributes(&node.parent_rc());
        let import_attributes_type = self.get_type_from_import_attributes(attributes.as_ref());
        let module_symbol = self.resolve_external_module_name_worker(
            node,
            get_module_specifier_from_node(&node.parent_rc()).as_ref(),
            None,
            false,
            false,
            import_attributes_type.as_ref(),
        );
        if let Some(module_symbol) = module_symbol {
            return self.get_target_of_module_default(&module_symbol, node, true);
        }
        None
    }

    pub fn get_target_of_import_equals_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_import_equals_declaration"); 
        let is_external_reference = is_variable_declaration(node)
            || node.as_import_equals_declaration().module_reference.kind
                == SyntaxKind::ExternalModuleReference;
        if is_external_reference {
            let mut module_reference = get_external_module_require_argument(node);
            if module_reference.is_none() {
                module_reference =
                    Some(get_external_module_import_equals_declaration_expression(node));
            }
            let immediate = self.resolve_external_module_name_worker(
                node,
                module_reference.as_ref(),
                None,
                false,
                false,
                None,
            );
            let resolved: Option<Arc<Symbol>> = immediate
                .as_ref()
                .map(|im| self.resolve_external_module_symbol(im, true));
            if let Some(resolved) = resolved.as_ref() {
                if MODULE_KIND_NODE20 <= self.module_kind
                    && self.module_kind <= MODULE_KIND_NODE_NEXT
                {
                    if let Some(module_exports) = self.get_export_of_module(
                        resolved,
                        InternalSymbolName::ModuleExports,
                        node,
                        true,
                    ) {
                        return Some(module_exports);
                    }
                }
            }
            self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
            return resolved;
        }
        let resolved = self.get_symbol_of_part_of_right_hand_side_of_import_equals(
            &node.as_import_equals_declaration().module_reference,
        );
        if let Some(resolved) = resolved.as_ref() {
            self.check_and_report_error_for_resolving_import_alias_to_type_only_symbol(node, resolved);
        }
        resolved
    }

    pub fn get_target_of_import_specifier(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_import_specifier"); 
        let name = node.property_name_or_name();
        if node.kind == SyntaxKind::ImportSpecifier && module_export_name_is_default(&name) {
            if let Some(specifier) = self.get_module_specifier_for_import_or_export(node) {
                let attributes =
                    get_import_attributes(&node.parent_rc().parent_rc().parent_rc());
                let import_attributes_type = self.get_type_from_import_attributes(attributes.as_ref());
                if let Some(module_symbol) = self.resolve_external_module_name_worker(
                    node,
                    Some(&specifier),
                    None,
                    false,
                    false,
                    import_attributes_type.as_ref(),
                ) {
                    return self.get_target_of_module_default(&module_symbol, node, true);
                }
            }
        }
        let root = if node.kind == SyntaxKind::BindingElement {
            get_root_declaration(node)
        } else {
            node.parent_rc().parent_rc().parent_rc()
        };
        let resolved = self.get_external_module_member(&root, node, true);
        self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
        resolved
    }

    pub fn get_target_of_module_default(
        &mut self,
        module_symbol: &Arc<Symbol>,
        node: &Arc<Node>,
        dont_resolve_alias: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_module_default"); 
        let file = module_symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .cloned();
        let specifier = self.get_module_specifier_for_import_or_export(node);
        let mut export_default_symbol: Option<Arc<Symbol>> = None;
        let mut export_module_dot_exports_symbol: Option<Arc<Symbol>> = None;
        let implied_format = file
            .as_ref()
            .and_then(|f| self.get_source_file_of_node(f))
            .map(|sf| self.emit_implied_node_format_for_file(&sf.file_name));
        let cjs_ish = file.is_some()
            && specifier.is_some()
            && MODULE_KIND_NODE20 <= self.module_kind
            && self.module_kind <= MODULE_KIND_NODE_NEXT
            && self.get_emit_syntax_for_module_specifier_expression(specifier.as_ref().unwrap())
                == MODULE_KIND_COMMON_JS
            && implied_format == Some(MODULE_KIND_ES_NEXT);
        if !is_shorthand_ambient_module_symbol(module_symbol) && cjs_ish {
            export_module_dot_exports_symbol = self.resolve_export_by_name(
                module_symbol,
                InternalSymbolName::ModuleExports,
                Some(node),
                dont_resolve_alias,
            );
        }
        if let Some(export_module_dot_exports_symbol) = export_module_dot_exports_symbol {
            self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
            return Some(export_module_dot_exports_symbol);
        }
        export_default_symbol = self.resolve_export_by_name(
            module_symbol,
            InternalSymbolName::Default,
            Some(node),
            dont_resolve_alias,
        );
        let specifier = specifier?;
        let attributes = if node.kind == SyntaxKind::ImportClause {
            get_import_attributes(&node.parent_rc())
        } else if node.kind == SyntaxKind::ImportSpecifier {
            get_import_attributes(&node.parent_rc().parent_rc().parent_rc())
        } else if node.kind == SyntaxKind::ExportSpecifier {
            get_import_attributes(&node.parent_rc().parent_rc())
        } else {
            None
        };
        let import_attributes_type = self.get_type_from_import_attributes(attributes.as_ref());
        let has_default_only = self.is_only_importable_as_default(
            &specifier,
            Some(module_symbol),
            import_attributes_type.as_ref(),
        );
        let has_synthetic_default = self.can_have_synthetic_default(
            file.as_ref(),
            module_symbol,
            dont_resolve_alias,
            &specifier,
        );
        if export_default_symbol.is_none() && !has_synthetic_default && !has_default_only {
            if node.kind == SyntaxKind::ImportClause {
                self.report_non_default_export(module_symbol, node);
            } else {
                let name = if is_import_or_export_specifier(node) {
                    node.property_name_or_name()
                } else {
                    node.name().unwrap().clone()
                };
                self.error_no_module_member_symbol(module_symbol, Some(module_symbol), node, &name);
            }
        } else if has_synthetic_default || has_default_only {
            let resolved = self.resolve_external_module_symbol(module_symbol, dont_resolve_alias);
            self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
            return Some(resolved);
        }
        self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
        export_default_symbol
    }

    pub fn get_target_of_namespace_export(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_namespace_export"); 
        let module_specifier = self.get_module_specifier_for_import_or_export(node)?;
        let attributes = get_import_attributes(&node.parent_rc());
        let import_attributes_type = self.get_type_from_import_attributes(attributes.as_ref());
        let immediate = self.resolve_external_module_name_worker(
            node,
            Some(&module_specifier),
            None,
            false,
            false,
            import_attributes_type.as_ref(),
        );
        let resolved = self.resolve_es_module_symbol(immediate.as_ref(), node, &module_specifier);
        self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
        resolved
    }

    pub fn get_target_of_namespace_export_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_namespace_export_declaration"); 
        let parent = node.parent_rc();
        if can_have_symbol(&parent) {
            let symbol = self.get_symbol_of_node(&parent)?;
            let resolved = self.resolve_external_module_symbol(&symbol, true);
            self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
            return Some(resolved);
        }
        None
    }

    pub fn get_target_of_namespace_import(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_namespace_import"); 
        let module_specifier = self.get_module_specifier_for_import_or_export(node)?;
        let attributes = get_import_attributes(&node.parent_rc().parent_rc());
        let import_attributes_type = self.get_type_from_import_attributes(attributes.as_ref());
        let immediate = self.resolve_external_module_name_worker(
            node,
            Some(&module_specifier),
            None,
            false,
            false,
            import_attributes_type.as_ref(),
        );
        let resolved = self.resolve_es_module_symbol(immediate.as_ref(), node, &module_specifier);
        self.mark_symbol_of_alias_declaration_if_type_only(Some(node), None);
        resolved
    }

    pub fn get_checker_target_type(&self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_checker_target_type"); 
        get_target_type(t)
    }
}

pub(crate) fn get_target_type(t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_target_type"); 
    if t.object_flags.intersects(ObjectFlags::Reference) {
        if let Some(target) = t.target() {
            return target.clone();
        }
    }
    t.clone()
}

pub(crate) fn get_template_type_key(texts: &[String], types: &[Arc<Type>]) -> CacheHashKey { ::tsox_core::fntrace::enter("get_template_type_key"); 
    let mut b = KeyBuilder::new();
    b.write_types(types);
    b.write_byte(b'|');
    for s in texts {
        b.write_int(s.len() as i32);
    }
    b.write_byte(b'|');
    for s in texts {
        b.write_string(s);
    }
    b.hash()
}

pub(crate) fn get_this_parameter_from_node_context(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_this_parameter_from_node_context"); 
    let this_container = get_this_container(node, false, false);
    if is_function_like(&this_container) {
        return get_this_parameter(&this_container);
    }
    None
}

pub(crate) fn get_start_element_count(t: &Arc<Type>, flags: ElementFlags) -> usize { ::tsox_core::fntrace::enter("get_start_element_count"); 
    let element_infos = match t.target_tuple_type() {
        Some(d) => &d.element_infos,
        None => return 0,
    };
    for (i, info) in element_infos.iter().enumerate() {
        if !info.flags.intersects(flags) {
            return i;
        }
    }
    element_infos.len()
}

pub(crate) fn get_end_element_count(t: &Arc<Type>, flags: ElementFlags) -> usize { ::tsox_core::fntrace::enter("get_end_element_count"); 
    let element_infos = match t.target_tuple_type() {
        Some(d) => &d.element_infos,
        None => return 0,
    };
    for i in (1..=element_infos.len()).rev() {
        if !element_infos[i - 1].flags.intersects(flags) {
            return element_infos.len() - i;
        }
    }
    element_infos.len()
}

pub(crate) fn get_total_fixed_element_count(t: &Arc<Type>) -> usize { ::tsox_core::fntrace::enter("get_total_fixed_element_count"); 
    let fixed_length = t.target_tuple_type().map(|d| d.fixed_length).unwrap_or(0);
    fixed_length + get_end_element_count(t, ELEMENT_FLAGS_FIXED)
}

pub(crate) fn get_tuple_key(element_infos: &[TupleElementInfo], readonly: bool) -> CacheHashKey { ::tsox_core::fntrace::enter("get_tuple_key"); 
    let mut b = KeyBuilder::new();
    for e in element_infos {
        if e.flags.intersects(ElementFlags::Required) {
            b.write_byte(b'#');
        } else if e.flags.intersects(ElementFlags::Optional) {
            b.write_byte(b'?');
        } else if e.flags.intersects(ElementFlags::Rest) {
            b.write_byte(b'.');
        } else {
            b.write_byte(b'*');
        }
        if let Some(labeled) = &e.labeled_declaration {
            b.write_node(Some(labeled));
        }
    }
    if readonly {
        b.write_byte(b'!');
    }
    b.hash()
}

pub(crate) fn get_symbol_of_node_impl(c: &mut Checker, node: &Node) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_of_node_impl"); 
    let symbol = c.program.symbol_map().symbol_of(node).cloned()?;
    let late = c.get_late_bound_symbol(&symbol);
    Some(c.get_merged_symbol(&late))
}
