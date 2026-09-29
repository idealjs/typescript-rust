#![allow(unused_imports)]
use super::m1e::r20k2_defs::get_source_file_of_node;
use tsox_core::collections::ordered_set::OrderedSet;
use tsox_core::tspath::file_extension_is_one_of;
use tsox_frontend::ast::is_module_with_string_literal_name;

#[path = "r25k10_defs.rs"]
pub mod r25k10_defs;

use self::r25k10_defs::{module_import_attributes_types_get, module_import_attributes_types_insert};

use crate::checker::checker::*;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

impl Checker {
    pub(crate) fn get_type_without_signatures(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::OBJECT) {
            let resolved = self.resolve_structured_type_members(t);
            let resolved_data = resolved.as_object_type();
            if resolved_data.map(|d| !d.structured.signatures.is_empty()).unwrap_or(false) {
                let mut result = self.new_object_type(ObjectFlags::ANONYMOUS, t.symbol.clone());
                if let Some(rt) = Arc::get_mut(&mut result) {
                    rt.object_flags |= ObjectFlags::MembersResolved;
                    if let TypeData::Object(d) = &mut rt.data {
                        if let Some(resolved_data) = resolved_data {
                            d.structured.members = resolved_data.structured.members.clone();
                            d.structured.properties =
                                resolved_data.structured.properties.clone();
                        }
                    }
                }
                return result;
            }
        }
        if t.flags.contains(TypeFlags::INTERSECTION) {
            let Some(data) = t.as_intersection_type() else {
                return Arc::clone(t);
            };
            let types = data
                .union_or_intersection
                .types
                .iter()
                .map(|c| self.get_type_without_signatures(c))
                .collect();
            return self.get_intersection_type(types);
        }
        Arc::clone(t)
    }

    pub(crate) fn get_type_of_module_declaration_import_attributes(
        &mut self,
        attributes: Option<&Arc<Node>>,
    ) -> Arc<Type> {
        let Some(attributes) = attributes else {
            return self.empty_object_type();
        };
        self.get_type_from_type_node(attributes)
    }

    pub(crate) fn get_type_of_module_import_attributes(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if let Some(t) = module_import_attributes_types_get(symbol) {
            return t;
        }
        let module_decl = symbol
            .declarations
            .iter()
            .find(|d| is_module_with_string_literal_name(d));
        let result = match module_decl {
            None => self.empty_object_type(),
            Some(decl) => match &decl.data {
                tsox_frontend::ast::NodeData::ModuleDeclaration(d) => self
                    .get_type_of_module_declaration_import_attributes(d.attributes.as_ref()),
                _ => self.empty_object_type(),
            },
        };
        module_import_attributes_types_insert(symbol, Arc::clone(&result));
        result
    }

    pub(crate) fn has_exported_members_of_kind(
        &mut self,
        module_symbol: &Arc<Symbol>,
        kind: SymbolFlags,
    ) -> bool {
        module_symbol.exports.entries.values().any(|symbol| {
            symbol.name != tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
                && self.get_symbol_flags(symbol).intersects(kind)
        })
    }

    pub(crate) fn has_shadowed_namespace(&mut self, symbol: &Arc<Symbol>) -> bool {
        if symbol
            .flags
            .intersects(SymbolFlags::NamespaceModule | SymbolFlags::Alias)
        {
            let target = self.resolve_alias(symbol);
            return target.flags.contains(SymbolFlags::NAMESPACE)
                && self.has_exported_members_of_kind(
                    &target,
                    SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
                );
        }
        false
    }

    pub(crate) fn get_type_of_symbol_with_deferred_type(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if let Some(t) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
        {
            return t;
        }
        let constituents = self
            .deferred_symbol_links
            .get(symbol)
            .map(|d| d.constituents.clone())
            .unwrap_or_default();
        let parent_is_union = self
            .deferred_symbol_links
            .get(symbol)
            .is_some_and(|d| d.parent.as_ref().is_some_and(|p| p.flags.contains(TypeFlags::UNION)));
        let resolved = if parent_is_union {
            self.get_union_type(constituents)
        } else {
            self.get_intersection_type(constituents)
        };
        self.value_symbol_links
            .get_or_default(symbol)
            .resolved_type = Some(Arc::clone(&resolved));
        resolved
    }

    pub(crate) fn get_write_type_of_symbol_with_deferred_type(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if let Some(t) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.write_type.clone())
        {
            return t;
        }
        let deferred = self
            .deferred_symbol_links
            .get(symbol)
            .map(|d| {
                (
                    d.write_constituents.clone(),
                    d.parent.as_ref().is_some_and(|p| p.flags.contains(TypeFlags::UNION)),
                )
            });
        let write_type = match deferred {
            Some((write_constituents, parent_is_union)) if !write_constituents.is_empty() => {
                if parent_is_union {
                    self.get_union_type(write_constituents)
                } else {
                    self.get_intersection_type(write_constituents)
                }
            }
            _ => self.get_type_of_symbol_with_deferred_type(symbol),
        };
        self.value_symbol_links
            .get_or_default(symbol)
            .write_type = Some(Arc::clone(&write_type));
        write_type
    }

    pub(crate) fn get_type_of_instantiated_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if let Some(t) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
        {
            return t;
        }
        let (target, mapper) = self
            .value_symbol_links
            .get(symbol)
            .map(|l| (l.target.clone(), l.mapper.clone()))
            .expect("instantiated symbol has links");
        let target = target.expect("instantiated symbol has target");
        let t = self.get_type_of_symbol(&target);
        let t = self.instantiate_type(&t, mapper.as_ref());
        self.value_symbol_links
            .get_or_default(symbol)
            .resolved_type = Some(Arc::clone(&t));
        t
    }

    pub(crate) fn get_write_type_of_instantiated_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if let Some(t) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.write_type.clone())
        {
            return t;
        }
        let (target, mapper) = self
            .value_symbol_links
            .get(symbol)
            .map(|l| (l.target.clone(), l.mapper.clone()))
            .expect("instantiated symbol has links");
        let target = target.expect("instantiated symbol has target");
        let t = self.get_write_type_of_symbol(&target);
        let t = self.instantiate_type(&t, mapper.as_ref());
        self.value_symbol_links
            .get_or_default(symbol)
            .write_type = Some(Arc::clone(&t));
        t
    }

    pub(crate) fn get_widened_type_for_variable_like_declaration(
        &mut self,
        declaration: &Arc<Node>,
        report_errors: bool,
    ) -> Option<Arc<Type>> {
        let t = self.get_type_for_variable_like_declaration(declaration, true, CheckMode::Normal);
        Some(self.widen_type_for_variable_like_declaration(t, declaration, report_errors))
    }

    pub(crate) fn get_type_of_func_class_enum_module(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if let Some(t) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
        {
            return t;
        }
        let t = self.get_type_of_func_class_enum_module_worker(symbol);
        self.value_symbol_links
            .get_or_default(symbol)
            .resolved_type = Some(Arc::clone(&t));
        t
    }

    pub(crate) fn get_type_of_enum_member(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        if let Some(t) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
        {
            return t;
        }
        let t = self.get_declared_type_of_enum_member(symbol);
        self.value_symbol_links
            .get_or_default(symbol)
            .resolved_type = Some(Arc::clone(&t));
        t
    }

    pub(crate) fn has_common_declaration(
        &mut self,
        symbols: &[Arc<Symbol>],
    ) -> bool {
        let mut common_declarations: Vec<Arc<Node>> = Vec::new();
        for symbol in symbols {
            if symbol.declarations.is_empty() {
                return false;
            }
            if common_declarations.is_empty() {
                for d in &symbol.declarations {
                    common_declarations.push(Arc::clone(d));
                }
                continue;
            }
            common_declarations.retain(|d| symbol.declarations.iter().any(|x| Arc::ptr_eq(x, d)));
        }
        !common_declarations.is_empty()
    }

    pub(crate) fn get_type_reference_type(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if Arc::ptr_eq(symbol, &self.unknown_symbol()) {
            return self.error_type();
        }
        if symbol
            .flags
            .intersects(SymbolFlags::Class | SymbolFlags::Interface)
        {
            return self.get_type_from_class_or_interface_reference(node, symbol);
        }
        if symbol.flags.contains(SymbolFlags::TypeAlias) {
            return self.get_type_from_type_alias_reference(node, symbol);
        }
        if let Some(res) = self.try_get_declared_type_of_symbol(symbol) {
            if self.check_no_type_arguments(node, Some(symbol)) {
                return self.get_regular_type_of_literal_type(&res);
            }
        }
        self.error_type()
    }
}

pub(crate) fn get_verbatim_module_syntax_error_message(
    checker: &Checker,
    node: &Arc<Node>,
) -> Message {
    let file_name = checker
        .get_source_file_of_node(node)
        .map(|sf| sf.file_name.clone())
        .unwrap_or_default();
    if file_extension_is_one_of(&file_name, &[".cts", ".cjs"]) {
        return tsox_core::diagnostics::messages_generated::ECMASCRIPT_IMPORTS_AND_EXPORTS_CANNOT_BE_WRITTEN_IN_A_COMMONJS_FILE_UNDER_VERBATIMMODULESYNTAX;
    }
    tsox_core::diagnostics::messages_generated::ECMASCRIPT_IMPORTS_AND_EXPORTS_CANNOT_BE_WRITTEN_IN_A_COMMONJS_FILE_UNDER_VERBATIMMODULESYNTAX_ADJUST_THE_TYPE_FIELD_IN_THE_NEAREST_PACKAGE_JSON_TO_MAKE_THIS_FILE_AN_ECMASCRIPT_MODULE_OR_ADJUST_YOUR_VERBATIMMODULESYNTAX_MODULE_AND_MODULERESOLUTION_SETTINGS_IN_TYPESCRIPT
}

pub(crate) fn get_type_list_key(types: &[Arc<Type>]) -> CacheHashKey {
    let mut b = KeyBuilder::new();
    b.write_types(types);
    b.hash()
}

pub(crate) fn get_union_key(
    types: &[Arc<Type>],
    origin: Option<&Arc<Type>>,
    alias: Option<&TypeAlias>,
) -> CacheHashKey {
    let mut b = KeyBuilder::new();
    match origin {
        None => b.write_types(types),
        Some(origin) if origin.flags.contains(TypeFlags::UNION) => {
            b.write_byte(b'|');
            b.write_types(origin.types().unwrap_or(&[]));
        }
        Some(origin) if origin.flags.contains(TypeFlags::INTERSECTION) => {
            b.write_byte(b'&');
            b.write_types(origin.types().unwrap_or(&[]));
        }
        Some(origin) => {
            b.write_byte(b'o');
            b.write_type(origin);
            b.write_types(types);
        }
    }
    if let Some(alias) = alias {
        b.write_alias(Some(alias));
    }
    b.hash()
}
