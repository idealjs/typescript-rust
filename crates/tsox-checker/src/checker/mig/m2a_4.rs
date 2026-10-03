#![allow(unused_imports)]
use crate::checker::mig::m2a::r19k11_defs::*;
use crate::checker::mig::m2a::r18k8_flags::*;
use tsox_frontend::ast::mig::m3e_4::get_first_identifier;
use tsox_frontend::ast::mig::m3f_3::is_alias_symbol_declaration;
use tsox_frontend::ast::mig::m3g_2::is_type_only_import_or_export_declaration;
use tsox_frontend::ast::mig::m3b::{is_type_only, module_specifier, property_name_or_name};
use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_INDEX;
use crate::checker::mig::m1c_3::create_diagnostic_for_node_message;
use crate::checker::mig::m2d::is_const_enum_or_const_enum_only_module;
use crate::checker::checker::Checker;

use crate::checker::checker_checker::*;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn is_type_assignable_to_kind_ex(
        &mut self,
        source: &Arc<Type>,
        kind: TypeFlags,
        strict: bool,
    ) -> bool { ::tsox_core::fntrace::enter("is_type_assignable_to_kind_ex"); 
        if source.flags.intersects(kind) {
            return true;
        }
        if strict
            && source
                .flags
                .intersects(TYPE_FLAGS_ANY_OR_UNKNOWN | TypeFlags::Void | TypeFlags::Undefined | TypeFlags::Null)
        {
            return false;
        }
        (kind.intersects(TYPE_FLAGS_NUMBER_LIKE) && {
            let number_type = self.number_type();
            self.is_type_assignable_to(source, &number_type)
        }) || (kind.intersects(TYPE_FLAGS_BIGINT_LIKE) && {
            let bigint_type = self.bigint_type();
            self.is_type_assignable_to(source, &bigint_type)
        }) || (kind.intersects(TYPE_FLAGS_STRING_LIKE) && {
            let string_type = self.string_type();
            self.is_type_assignable_to(source, &string_type)
        }) || (kind.intersects(TYPE_FLAGS_BOOLEAN_LIKE) && {
            let boolean_type = self.boolean_type();
            self.is_type_assignable_to(source, &boolean_type)
        }) || (kind.intersects(TYPE_FLAGS_VOID) && {
            let void_type = self.void_type();
            self.is_type_assignable_to(source, &void_type)
        }) || (kind.intersects(TYPE_FLAGS_NEVER) && {
            let never_type = self.never_type();
            self.is_type_assignable_to(source, &never_type)
        }) || (kind.intersects(TYPE_FLAGS_NULL) && {
            let null_type = self.null_type();
            self.is_type_assignable_to(source, &null_type)
        }) || (kind.intersects(TYPE_FLAGS_UNDEFINED) && {
            let undefined_type = self.undefined_type();
            self.is_type_assignable_to(source, &undefined_type)
        }) || (kind.intersects(TYPE_FLAGS_ES_SYMBOL) && {
            let es_symbol_type = self.es_symbol_type();
            self.is_type_assignable_to(source, &es_symbol_type)
        }) || (kind.intersects(TYPE_FLAGS_NON_PRIMITIVE) && {
            let non_primitive_type = self.non_primitive_type();
            self.is_type_assignable_to(source, &non_primitive_type)
        })
    }

    pub fn late_bind_index_signature(
        &mut self,
        _parent: &Arc<Symbol>,
        early_symbols: &SymbolTable,
        late_symbols: &mut SymbolTable,
        decl: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("late_bind_index_signature"); 
        let mut index_symbol = late_symbols
            .get(INTERNAL_SYMBOL_NAME_INDEX)
            .cloned();
        if index_symbol.is_none() {
            let early = early_symbols.get(INTERNAL_SYMBOL_NAME_INDEX);
            let symbol = match early {
                Some(early) => {
                    let mut cloned = self.clone_symbol(early).unwrap();
                    if let Some(cloned_mut) = Arc::get_mut(&mut cloned) {
                        cloned_mut.check_flags |= CHECK_FLAGS_LATE;
                    }
                    cloned
                }
                None => self.new_symbol_ex(SymbolFlags::None, INTERNAL_SYMBOL_NAME_INDEX, CHECK_FLAGS_LATE),
            };
            index_symbol = Some(symbol);
            late_symbols.insert(
                INTERNAL_SYMBOL_NAME_INDEX.to_string(),
                index_symbol.clone().unwrap(),
            );
        }
        let index_symbol = index_symbol.unwrap();
        let decl_replaceable_by_method = self
            .get_symbol_of_declaration(decl)
            .map(|s| s.flags.intersects(SymbolFlags::ReplaceableByMethod))
            .unwrap_or(false);
        if index_symbol.declarations.is_empty() || !decl_replaceable_by_method {
            let index_symbol_mut = Arc::as_ptr(&index_symbol) as *mut Symbol;
            unsafe {
                (*index_symbol_mut).declarations.push(Arc::clone(decl));
            }
        }
    }

    pub fn mark_symbol_of_alias_declaration_if_type_only(
        &mut self,
        alias_declaration: Option<&Arc<Node>>,
        export_star_declaration: Option<&Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("mark_symbol_of_alias_declaration_if_type_only"); 
        let Some(alias_declaration) = alias_declaration else {
            return false;
        };
        if !ast::is_declaration_node(alias_declaration) {
            return false;
        }
        let source_symbol = self.get_symbol_of_declaration(alias_declaration).unwrap();
        let links = self.alias_symbol_links.get_mut(&source_symbol).unwrap();
        if links.type_only_declaration.is_none()
            && is_type_only_import_or_export_declaration(alias_declaration)
        {
            links.type_only_declaration = Some(Arc::clone(alias_declaration));
            return true;
        }
        if links.type_only_declaration.is_none() {
            if let Some(export_star_declaration) = export_star_declaration {
                links.type_only_declaration = Some(Arc::clone(export_star_declaration));
                return true;
            }
        }
        links.type_only_declaration.is_some()
    }

    pub fn mark_entity_name_or_entity_expression_as_reference(
        &mut self,
        typeName: &Arc<Node>,
        for_decorator_metadata: bool,
    ) { ::tsox_core::fntrace::enter("mark_entity_name_or_entity_expression_as_reference"); 
        let root_name = get_first_identifier(typeName);
        let meaning = if typeName.kind == SyntaxKind::Identifier {
            SymbolFlags::TYPE
        } else {
            SymbolFlags::NAMESPACE
        } | SymbolFlags::Alias;
        let root_symbol = self.resolve_name(
            root_name.text(),
            &root_name,
            meaning,
            false,
        );
        if let Some(root_symbol) = root_symbol {
            if root_symbol.flags.intersects(SymbolFlags::Alias) {
                let not_const_enum =
                    !is_const_enum_or_const_enum_only_module(&self.resolve_alias(&root_symbol));
                if self.can_collect_symbol_alias_accessibility_data
                    && self.symbol_is_value(&root_symbol)
                    && not_const_enum
                    && self.get_type_only_alias_declaration(&root_symbol).is_none()
                {
                    self.mark_alias_symbol_as_referenced(&root_symbol);
                } else if for_decorator_metadata
                    && self.compiler_options.get_isolated_modules()
                    && self.compiler_options.get_emit_module_kind() >= MODULE_KIND_ES2015
                    && !self.symbol_is_value(&root_symbol)
                    && !root_symbol
                        .declarations
                        .iter()
                        .any(|d| is_type_only_import_or_export_declaration(d))
                {
                    let diag = self.error_message(
                        typeName,messages_generated::A_TYPE_REFERENCED_IN_A_DECORATED_SIGNATURE_MUST_BE_IMPORTED_WITH_IMPORT_TYPE_OR_A_NAMESPACE_IMPORT_WHEN_ISOLATEDMODULES_AND_EMITDECORATORMETADATA_ARE_ENABLED,
                        &[],
                    );
                    let alias_declaration = root_symbol
                        .declarations
                        .iter()
                        .find(|d| is_alias_symbol_declaration(d));
                    if let (Some(mut diag), Some(alias_declaration)) = (diag, alias_declaration) {
                        let related = create_diagnostic_for_node_message(
                            alias_declaration,messages_generated::X_0_WAS_IMPORTED_HERE,
                            &[root_name.text().to_string()],
                        );
                        if let Some(diag) = Arc::get_mut(&mut diag) {
                            diag.set_related_info(vec![(*related).clone()]);
                        }
                    }
                }
            }
        }
    }

    pub fn mark_export_specifier_alias_referenced(&mut self, location: &Arc<Node>) { ::tsox_core::fntrace::enter("mark_export_specifier_alias_referenced"); 
        let parent = location.parent().unwrap();
        let grandparent = parent.parent().unwrap();
        if module_specifier(&grandparent).is_none()
            && !is_type_only(location)
            && !is_type_only(&grandparent)
        {
            let Some(exported_name) = property_name_or_name(location) else {
                return;
            };
            if exported_name.kind == SyntaxKind::StringLiteral {
                return;
            }
            let symbol = self.resolve_name(
                exported_name.text(),
                exported_name,
                SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE | SymbolFlags::Alias,
                false,
            );
            let is_non_local = symbol.as_ref().is_some_and(|symbol| {
                self.undefined_symbol
                    .as_ref()
                    .is_some_and(|u| Arc::ptr_eq(symbol, u))
                    || self
                        .global_this_symbol
                        .as_ref()
                        .is_some_and(|g| Arc::ptr_eq(symbol, g))
                    || (!symbol.declarations.is_empty()
                        && Checker::get_declaration_container(&symbol.declarations[0])
                            .is_some_and(|c| Checker::is_global_source_file(&c)))
            });
            if !is_non_local {
                let mut target = symbol.clone();
                if target
                    .as_ref()
                    .is_some_and(|t| t.flags.intersects(SymbolFlags::Alias))
                {
                    target = Some(self.resolve_alias(target.as_ref().unwrap()));
                }
                if target.is_none()
                    || self
                        .get_symbol_flags(target.as_ref().unwrap())
                        .intersects(SymbolFlags::VALUE)
                {
                    self.mark_export_as_referenced(location);
                    self.mark_identifier_alias_referenced(exported_name);
                }
            }
        }
    }
}
