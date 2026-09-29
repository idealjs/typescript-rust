use crate::checker::checker::*;
use crate::checker::mig::m2a::r19k11_defs::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags};

impl Checker {
    pub fn resolve_alias_with_deprecation_check(
        &mut self,
        symbol: &Arc<Symbol>,
        location: &Arc<Node>,
    ) -> Arc<Symbol> {
        if !symbol.flags.intersects(SymbolFlags::Alias)
            || self.is_deprecated_symbol(symbol)
            || self.get_declaration_of_alias_symbol(symbol).is_none()
        {
            return Arc::clone(symbol);
        }
        let target_symbol = self.resolve_alias(symbol);
        let unknown_symbol = self.unknown_symbol();
        if Arc::ptr_eq(&target_symbol, &unknown_symbol) {
            return target_symbol;
        }
        let mut symbol = Arc::clone(symbol);
        while symbol.flags.intersects(SymbolFlags::Alias) {
            let target = self.get_immediate_aliased_symbol(&symbol);
            if let Some(target) = target {
                if Arc::ptr_eq(&target, &target_symbol) {
                    break;
                }
                if !target.declarations.is_empty() {
                    if self.is_deprecated_symbol(&target) {
                        self.add_deprecated_suggestion(location, &target.declarations, &target.name);
                        break;
                    } else {
                        if Arc::ptr_eq(&symbol, &target_symbol) {
                            break;
                        }
                        symbol = target;
                    }
                }
            } else {
                break;
            }
        }
        target_symbol
    }

    pub fn resolve_base_types_of_interface(&mut self, t: &Arc<Type>) {
        let symbol = t.symbol.clone().unwrap();
        for declaration in &symbol.declarations {
            if tsox_frontend::ast::is_interface_declaration(declaration) {
                for node in tsox_frontend::ast::get_extends_heritage_clause_elements(declaration) {
                    let node_type = self.get_type_from_type_node(&node);
                    let base_type = self.get_reduced_type(&node_type);
                    if !self.is_error_type(&base_type) {
                        if self.is_valid_base_type(&base_type) {
                            if !Arc::ptr_eq(t, &base_type) && !self.has_base_type(&base_type, t) {
                                let mut t = Arc::clone(t);
                                if let Some(t_mut) = Arc::get_mut(&mut t) {
                                    if let TypeData::Interface(interface) = &mut t_mut.data {
                                        interface.resolved_base_types.push(base_type);
                                    }
                                }
                            } else {
                                self.report_circular_base_type(declaration, t);
                            }
                        } else {
                            self.error_message(
                                &node,tsox_core::diagnostics::messages_generated::AN_INTERFACE_CAN_ONLY_EXTEND_AN_OBJECT_TYPE_OR_INTERSECTION_OF_OBJECT_TYPES_WITH_STATICALLY_KNOWN_MEMBERS,
                                &[],
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn resolve_class_or_interface_members(&mut self, t: &Arc<Type>) {
        self.resolve_object_type_members(t, t, &[], &[]);
    }

    pub fn resolve_declared_members(&mut self, t: &Arc<Type>) {
        let mut t = Arc::clone(t);
        let Some(t_mut) = Arc::get_mut(&mut t) else {
            return;
        };
        let TypeData::Interface(interface) = &mut t_mut.data else {
            return;
        };
        if interface.declared_members_resolved {
            return;
        }
        let symbol = t_mut.symbol.clone().unwrap();
        let members = if symbol
            .flags
            .intersects(tsox_frontend::ast::SymbolFlags::Class
                .union(tsox_frontend::ast::SymbolFlags::Interface)
                .union(tsox_frontend::ast::SymbolFlags::TypeLiteral)
                .union(tsox_frontend::ast::SymbolFlags::ObjectLiteral)
                .union(tsox_frontend::ast::SymbolFlags::Function))
        {
            self.get_resolved_members_or_exports_of_symbol(
                &symbol,
                crate::checker::types_alias_symbol_links::MembersOrExportsResolutionKind::ResolvedMembers,
            )
        } else {
            symbol.members.clone()
        };
        let declared_call_signatures =
            self.get_signatures_of_symbol(members.get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_CALL));
        let declared_construct_signatures =
            self.get_signatures_of_symbol(members.get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_NEW));
        let declared_index_infos = self.get_index_infos_of_symbol(&symbol);
        interface.declared_members_resolved = true;
        interface.declared_members = members;
        interface.declared_call_signatures = declared_call_signatures;
        interface.declared_construct_signatures = declared_construct_signatures;
        interface.declared_index_infos = declared_index_infos;
    }

    pub fn resolve_error_call(&mut self, node: &Arc<Node>) -> Arc<Signature> {
        self.resolve_untyped_call(node);
        self.unknown_signature()
    }
}
