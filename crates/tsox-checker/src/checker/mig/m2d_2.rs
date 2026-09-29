#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol;
use crate::checker::mig::m2d::EmitResolver;
use crate::checker::types::{MemberOverrideStatus, ObjectFlags, SymbolFormatFlags, Type, TypeFlags};
use crate::checker::utilities_get_assignment_target::get_enclosing_container;
use std::sync::Arc;
use crate::checker::utilities_is_optional_symbol::is_type_alias;
use crate::checker::utilities_token_is_identifier_or_keyword::is_tuple_type;
use tsox_frontend::ast::{
    get_class_extends_heritage_element, has_abstract_modifier, has_static_modifier,
    has_syntactic_modifier, is_class_declaration, is_class_expression, is_global_scope_augmentation,
    is_in_js_file, is_interface_declaration, is_module_block, is_static, node_kind_is, CheckFlags,
    ModifierFlags, Node, NodeFlags, Symbol, SymbolFlags, SyntaxKind,
};

pub fn is_tuple_type_target(t: &Arc<Type>) -> bool {
    is_tuple_type(t) && t.target().is_some_and(|target| Arc::ptr_eq(&target, t))
}

impl Checker {
    pub fn get_local_type_parameters_of_class_or_interface_or_type_alias(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Type>> {
        let mut types: Vec<Arc<Type>> = Vec::new();
        for node in &symbol.declarations {
            if node_kind_is(
                node,
                &[
                    SyntaxKind::InterfaceDeclaration,
                    SyntaxKind::ClassDeclaration,
                    SyntaxKind::ClassExpression,
                ],
            ) || is_type_alias(node)
            {
                types = self.append_type_parameters(
                    types,
                    tsox_frontend::ast::mig::m3c::type_parameters(node).to_vec(),
                );
            }
        }
        types
    }

    pub fn get_member_override_modifier_status(
        &mut self,
        node: &Arc<Node>,
        member: &Arc<Node>,
        member_symbol: Option<&Arc<Symbol>>,
    ) -> MemberOverrideStatus {
        let Some(member_symbol) = member_symbol else {
            return MemberOverrideStatus::None;
        };
        if member.name().is_none() {
            return MemberOverrideStatus::None;
        }

        let Some(class_symbol) = self.get_symbol_of_declaration(node) else {
            return MemberOverrideStatus::None;
        };

        let t = self.get_declared_type_of_symbol(&class_symbol);
        let type_with_this = self.get_type_with_this_argument(&t, None, false);
        let static_type = self.get_type_of_symbol(&class_symbol);

        let mut base_with_this: Option<Arc<Type>> = None;
        if get_class_extends_heritage_element(node).is_some() {
            let base_types = self.get_base_types(&t);
            if !base_types.is_empty() {
                base_with_this = Some(self.get_type_with_this_argument(
                    &base_types[0],
                    t.as_interface_type().and_then(|it| it.this_type.clone()).as_ref(),
                    false,
                ));
            }
        }

        let base_constructor_type = self
            .get_base_constructor_type_of_class(&t)
            .unwrap_or_else(|| self.empty_generic_type());
        self.check_member_for_override_modifier_worker(
            node,
            &static_type,
            &base_constructor_type,
            base_with_this.as_ref(),
            &t,
            &type_with_this,
            has_syntactic_modifier(member, ModifierFlags::Override),
            has_abstract_modifier(member),
            is_static(member),
            false,
            member_symbol,
            None,
        )
    }

    pub fn get_reduced_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TypeFlags::Union) {
            if t.object_flags
                .intersects(ObjectFlags::ContainsIntersections)
            {
                if let Some(data) = t.as_union_type() {
                    if let Some(reduced_type) = data.resolved_reduced_type.get() {
                        return Arc::clone(reduced_type);
                    }
                }
                let reduced_type = self.get_reduced_union_type(t);
                if let Some(data) = t.as_union_type() {
                    data.resolved_reduced_type
                        .set(Arc::clone(&reduced_type))
                        .ok();
                }
                return reduced_type;
            }
        } else if t.flags.intersects(TypeFlags::Intersection) {
            if !t
                .object_flags
                .intersects(ObjectFlags::IsNeverIntersectionComputed)
            {
                let mut owned = Arc::clone(t);
                let mut is_never = owned
                    .object_flags
                    .intersects(ObjectFlags::IsNeverIntersection);
                if let Some(owned_mut) = Arc::get_mut(&mut owned) {
                    owned_mut.object_flags |= ObjectFlags::IsNeverIntersectionComputed;
                }
                let props = self.get_properties_of_union_or_intersection_type(&owned);
                if props.iter().any(|p| self.is_never_reduced_property(p)) {
                    is_never = true;
                    if let Some(owned_mut) = Arc::get_mut(&mut owned) {
                        owned_mut.object_flags |= ObjectFlags::IsNeverIntersection;
                    }
                }
                if is_never {
                    return Arc::clone(self.never_type.get().expect("never_type"));
                }
                return owned;
            }
            if t.object_flags.intersects(ObjectFlags::IsNeverIntersection) {
                return Arc::clone(self.never_type.get().expect("never_type"));
            }
        }
        Arc::clone(t)
    }

    pub fn is_readonly_symbol(&mut self, symbol: &Arc<Symbol>) -> bool {
        symbol.check_flags.intersects(CheckFlags::Readonly)
            || symbol.flags.intersects(SymbolFlags::Property)
                && get_declaration_modifier_flags_from_symbol(symbol)
                    .intersects(ModifierFlags::Readonly)
            || symbol.flags.intersects(SymbolFlags::VARIABLE)
                && self
                    .get_declaration_node_flags_from_symbol(symbol)
                    .intersects(NodeFlags::Constant)
            || symbol.flags.intersects(SymbolFlags::ACCESSOR)
                && !symbol.flags.intersects(SymbolFlags::SetAccessor)
            || symbol.flags.intersects(SymbolFlags::EnumMember)
            || symbol
                .declarations
                .iter()
                .any(|d| self.is_readonly_assignment_declaration(d))
    }
}

impl EmitResolver {
    pub fn get_effective_declaration_flags(
        &self,
        n: &Arc<Node>,
        flags_to_check: ModifierFlags,
    ) -> ModifierFlags {
        let checker = unsafe { &mut *self.checker };
        checker.get_effective_declaration_flags(n, flags_to_check)
    }
}

impl Checker {
    pub fn get_effective_declaration_flags(
        &mut self,
        n: &Arc<Node>,
        flags_to_check: ModifierFlags,
    ) -> ModifierFlags {
        let mut flags = self.get_combined_modifier_flags_cached(n);
        let parent = n.parent();
        let in_class_like_container = parent.as_deref().is_some_and(|p| {
            is_interface_declaration(p) || is_class_declaration(p) || is_class_expression(p)
        });
        if !in_class_like_container && n.flags.intersects(NodeFlags::Ambient) {
            if let Some(container) = get_enclosing_container(n) {
                let in_global_scope_augmentation_module_block = parent.as_deref()
                    .is_some_and(is_module_block)
                    && parent
                        .as_ref()
                        .and_then(|p| p.parent())
                        .as_deref()
                        .is_some_and(is_global_scope_augmentation);
                if container.flags.intersects(NodeFlags::ExportContext)
                    && !flags.intersects(ModifierFlags::Ambient)
                    && !in_global_scope_augmentation_module_block
                {
                    flags |= ModifierFlags::Export;
                }
            }
            flags |= ModifierFlags::Ambient;
        }
        flags & flags_to_check
    }
}
