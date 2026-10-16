
use std::sync::Arc;

use tsox_core::diagnostics::messages_generated::{
    CANNOT_FIND_GLOBAL_TYPE_0, CANNOT_FIND_GLOBAL_VALUE_0, CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS,
    GLOBAL_TYPE_0_MUST_BE_A_CLASS_OR_INTERFACE_TYPE, GLOBAL_TYPE_0_MUST_HAVE_1_TYPE_PARAMETER_S,
    NAMED_IMPORTS_FROM_A_JSON_FILE_INTO_AN_ECMASCRIPT_MODULE_ARE_NOT_ALLOWED_WHEN_MODULE_IS_SET_TO_0,
};
use tsox_frontend::ast::mig::m3e_3::symbol_name;
use tsox_frontend::ast::mig::m3e_4::{get_declaration_of_kind, get_import_attributes};
use tsox_frontend::ast::mig::m3f_2::has_import_attributes;
use tsox_frontend::ast::{CheckFlags, ModifierFlags, Node, NodeData, NodeFlags, Symbol, SymbolFlags, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::{
    IntersectionFlags, ObjectFlags, SymbolFormatFlags, Type, TypeData, TypeFlags,
};
use crate::checker::types_impl_chunk::LiteralValue;
use crate::checker::utilities_get_assignment_target::{
    get_external_module_require_argument, is_shorthand_ambient_module_symbol,
};
use crate::checker::utilities_has_only_expression_initialization::create_symbol_table;

use super::wc1b::is_tuple_type;
use super::wc3::NodeAccessExt;
use super::wc3_3::is_generic_tuple_type;

#[path = "r25k4_defs.rs"]
pub(crate) mod r25k4_defs;
pub(crate) use r25k4_defs::*;

#[path = "r28k9_defs.rs"]
pub(crate) mod r28k9_defs;
pub(crate) use r28k9_defs::*;

#[path = "r26k3_defs.rs"]
pub(crate) mod r26k3_defs;
pub(crate) use r26k3_defs::*;

#[path = "r17k3_flags.rs"]
pub(crate) mod r17k3_flags;
pub(crate) use r17k3_flags::*;

#[path = "r18k5_helpers.rs"]
pub(crate) mod r18k5_helpers;
pub(crate) use r18k5_helpers::*;

#[path = "r20k2_defs.rs"]
pub(crate) mod r20k2_defs;
pub(crate) use r20k2_defs::*;

#[path = "r23k2_defs.rs"]
pub(crate) mod r23k2_defs;

#[path = "r24k6_defs.rs"]
pub(crate) mod r24k6_defs;
pub(crate) use r24k6_defs::*;

use tsox_frontend::ast::node_data_generated::{
    is_binding_pattern, is_call_expression, is_class_declaration, is_class_static_block_declaration,
    is_function_declaration, is_identifier, is_property_access_expression, is_source_file,
    is_string_literal, is_type_alias_declaration, is_variable_declaration_list,
};

pub enum EnumLiteralValue {
    Str(String),
    Num(f64),
}

impl Clone for EnumLiteralValue {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        match self {
            EnumLiteralValue::Str(s) => EnumLiteralValue::Str(s.clone()),
            EnumLiteralValue::Num(n) => EnumLiteralValue::Num(*n),
        }
    }
}

impl PartialEq for EnumLiteralValue {
    fn eq(&self, other: &Self) -> bool { ::tsox_core::fntrace::enter("eq"); 
        match (self, other) {
            (EnumLiteralValue::Str(a), EnumLiteralValue::Str(b)) => a == b,
            (EnumLiteralValue::Num(a), EnumLiteralValue::Num(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}

impl Eq for EnumLiteralValue {}

impl std::hash::Hash for EnumLiteralValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { ::tsox_core::fntrace::enter("hash"); 
        match self {
            EnumLiteralValue::Str(s) => {
                std::hash::Hash::hash(&0u8, state);
                std::hash::Hash::hash(s, state);
            }
            EnumLiteralValue::Num(n) => {
                std::hash::Hash::hash(&1u8, state);
                std::hash::Hash::hash(&n.to_bits(), state);
            }
        }
    }
}

fn literal_value_of(value: &EnumLiteralValue) -> LiteralValue { ::tsox_core::fntrace::enter("literal_value_of"); 
    match value {
        EnumLiteralValue::Str(s) => LiteralValue::String(s.clone()),
        EnumLiteralValue::Num(n) => LiteralValue::Number(tsox_core::jsnum::Number::from(*n)),
    }
}

impl Checker {
    pub fn get_entity_name_for_extending_interface(&self, node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_entity_name_for_extending_interface"); 
        match node.kind {
            SyntaxKind::Identifier | SyntaxKind::QualifiedName | SyntaxKind::PropertyAccessExpression => {
                let parent = node.parent()?;
                return self.get_entity_name_for_extending_interface(&parent);
            }
            SyntaxKind::TypeReference => {
                if let NodeData::TypeReferenceNode(tr) = &node.data {
                    return Some(Arc::clone(&tr.type_name));
                }
            }
            SyntaxKind::ExpressionWithTypeArguments => {
                if let Some(e) = node.expression()
                    && is_entity_name_expression_local(e.as_ref())
                {
                    return Some(Arc::clone(e));
                }
            }
            _ => {}
        }
        None
    }

    pub fn get_extract_string_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_extract_string_type"); 
        if let Some(extract_type_alias) = self.get_global_extract_symbol() {
            return self.get_type_alias_instantiation(&extract_type_alias, &[Arc::clone(t), self.string_type()], None);
        }
        self.string_type()
    }

    pub fn get_enum_literal_type(&mut self, value: EnumLiteralValue, enum_symbol: &Arc<Symbol>, symbol: Option<Arc<Symbol>>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_enum_literal_type"); 
        let flags = match &value {
            EnumLiteralValue::Str(_) => TypeFlags::ENUM_LITERAL | TypeFlags::STRING_LITERAL,
            EnumLiteralValue::Num(n) => {
                // NaN cannot be used as a map key (NaN != NaN), cache it separately by enum symbol
                if n.is_nan() {
                    let key = Arc::as_ptr(enum_symbol) as *const () as usize;
                    if let Some(t) = enum_nan_literal_types_get(&key) {
                        return t;
                    }
                    let mut t = self.new_literal_type(TypeFlags::ENUM_LITERAL | TypeFlags::NUMBER_LITERAL, literal_value_of(&value), None);
                    if let Some(t_mut) = Arc::get_mut(&mut t) {
                        t_mut.symbol = symbol.clone();
                    }
                    enum_nan_literal_types_insert(key, Arc::clone(&t));
                    return t;
                }
                TypeFlags::ENUM_LITERAL | TypeFlags::NUMBER_LITERAL
            }
        };
        let key = (Arc::as_ptr(enum_symbol) as *const () as usize, value.clone());
        if let Some(t) = enum_literal_types_get(&key) {
            return t;
        }
        let mut t = self.new_literal_type(flags, literal_value_of(&value), None);
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            t_mut.symbol = symbol.clone();
        }
        enum_literal_types_insert(key, Arc::clone(&t));
        t
    }

    pub fn get_error_node_for_call_node(node: &Node) -> &Node { ::tsox_core::fntrace::enter("get_error_node_for_call_node"); 
        if is_call_expression(node) {
            if let Some(expr) = node.expression() {
                if is_property_access_expression(expr) {
                    if let Some(name) = expr.name() {
                        return name;
                    }
                }
                return expr;
            }
        }
        node
    }

    pub fn get_exact_optional_unassignable_properties(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_exact_optional_unassignable_properties"); 
        if is_tuple_type(source) && is_tuple_type(target) {
            return Vec::new();
        }
        self.get_properties_of_type(target)
            .into_iter()
            .filter(|target_prop| {
                let Some(prop_type) = self.get_type_of_property_of_type(source, &target_prop.name) else {
                    return false;
                };
                let expected_type = self.get_type_of_symbol(target_prop);
                self.is_exact_optional_property_mismatch(Some(&prop_type), Some(&expected_type))
            })
            .collect()
    }

    pub fn get_external_module_file_from_declaration(&mut self, declaration: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_external_module_file_from_declaration"); 
        let module_declaration_name = if declaration.kind == SyntaxKind::ModuleDeclaration {
            declaration.name().filter(|n| is_string_literal(n)).cloned()
        } else {
            get_external_module_name(declaration)
        };
        let specifier = module_declaration_name.as_ref()?;
        let mut import_attributes_type: Option<Arc<Type>> = None;
        if has_import_attributes(declaration) {
            import_attributes_type = self.get_type_from_import_attributes(get_import_attributes(declaration).as_ref());
        }
        let module_symbol = self.resolve_external_module_name_worker(
            specifier,
            Some(specifier),
            None,
            false,
            false,
            import_attributes_type.as_ref(),
        )?;
        let decl = get_declaration_of_kind(&module_symbol, SyntaxKind::SourceFile)?;
        Some(decl)
    }

    pub fn get_external_module_member(&mut self, node: &Arc<Node>, specifier: &Arc<Node>, dont_resolve_alias: bool) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_external_module_member"); 
        let mut module_specifier = get_external_module_require_argument(node);
        if module_specifier.is_none() {
            module_specifier = get_external_module_name(node);
        }
        let attributes: Option<Arc<Node>> = if has_import_attributes(node) {
            get_import_attributes(node)
        } else {
            None
        };
        let import_attributes_type = self.get_type_from_import_attributes(attributes.as_ref());
        let module_specifier = module_specifier?;
        let ignore_errors = self.compiler_options.no_check.is_true();
        let module_not_found_error = if ignore_errors {
            None
        } else {
            Some(&CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS)
        };
        let module_symbol = self.resolve_external_module_name_worker(
            node,
            Some(&module_specifier),
            module_not_found_error,
            ignore_errors,
            false,
            import_attributes_type.as_ref(),
        )?;
        let name: &Arc<Node> = if !is_property_access_expression(specifier) {
            match &specifier.data {
                NodeData::ImportSpecifier(d) => d.property_name.as_ref().unwrap_or(&d.name),
                NodeData::ExportSpecifier(d) => d.property_name.as_ref().unwrap_or(&d.name),
                NodeData::ShorthandPropertyAssignment(d) => &d.name,
                _ => specifier,
            }
        } else {
            specifier.name().unwrap_or(specifier)
        };
        if !is_identifier(name) && !is_string_literal(name) {
            return None;
        }
        let name_text = name.text();
        let target_symbol = self.resolve_es_module_symbol(Some(&module_symbol), specifier, &module_specifier)?;
        // Note: The empty string is a valid module export name:
        //
        //   import { "" as foo } from "./foo";
        //   export { foo as "" };
        //
        if !name_text.is_empty() || name.kind == SyntaxKind::StringLiteral {
            if is_shorthand_ambient_module_symbol(&module_symbol) {
                return Some(module_symbol);
            }
            let symbol_from_variable = if module_symbol.exports.get(internal_symbol_name_export_equals).is_some() {
                let target_symbol_type = self.get_type_of_symbol(&target_symbol);
                self.get_property_of_type_ex(&target_symbol_type, &name_text, true, false)
            } else {
                self.get_property_of_variable(&target_symbol, &name_text)
            };
            // if symbolFromVariable is export - get its final target
            let symbol_from_variable = match symbol_from_variable {
                Some(sv) => Some(self.resolve_symbol_ex(&sv, dont_resolve_alias)),
                None => None,
            };
            let mut export_container = Arc::clone(&target_symbol);
            if module_symbol.exports.get(internal_symbol_name_export_equals).is_some() {
                // For `export =` modules, supplemental type/namespace exports live on the original module symbol.
                export_container = Arc::clone(&module_symbol);
            }
            let mut symbol_from_module = self.get_export_of_module(&export_container, &name_text, specifier, dont_resolve_alias);
            if symbol_from_module.is_none() && name_text == internal_symbol_name_default {
                let file = module_symbol
                    .declarations
                    .iter()
                    .find(|d| is_source_file(d))
                    .cloned();
                if self.is_only_importable_as_default(&module_specifier, Some(&module_symbol), import_attributes_type.as_ref())
                    || self.can_have_synthetic_default(file.as_ref(), &module_symbol, dont_resolve_alias, &module_specifier)
                {
                    symbol_from_module = Some(self.resolve_external_module_symbol(&module_symbol, dont_resolve_alias));
                    if symbol_from_module.is_none() {
                        symbol_from_module = Some(self.resolve_symbol_ex(&module_symbol, dont_resolve_alias));
                    }
                }
            }
            let mut symbol = symbol_from_variable.clone();
            if let Some(sm) = symbol_from_module {
                symbol = Some(match symbol_from_variable {
                    Some(sv) => self.combine_value_and_type_symbols(&sv, &sm),
                    None => sm,
                });
            }
            if specifier.is_import_or_export_specifier()
                && self.is_only_importable_as_default(&module_specifier, Some(&module_symbol), import_attributes_type.as_ref())
                && name_text != internal_symbol_name_default
            {
                self.error_message(
                    name,NAMED_IMPORTS_FROM_A_JSON_FILE_INTO_AN_ECMASCRIPT_MODULE_ARE_NOT_ALLOWED_WHEN_MODULE_IS_SET_TO_0,
                    &[format!("{:?}", self.module_kind)],
                );
            } else if symbol.is_none() {
                self.error_no_module_member_symbol(&module_symbol, Some(&target_symbol), node, name);
            }
            return symbol;
        }
        None
    }

    pub fn get_flow_type_of_destructuring(&mut self, node: &Arc<Node>, declared_type: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_flow_type_of_destructuring"); 
        match self.get_synthetic_element_access(node) {
            Some(reference) => self.get_flow_type_of_reference(&reference, declared_type),
            None => Arc::clone(declared_type),
        }
    }

    pub fn get_flow_type_of_property(&mut self, reference: &Arc<Node>, prop: Option<&Arc<Symbol>>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_flow_type_of_property"); 
        let mut initial_type = self.undefined_type();
        if let Some(prop) = prop
            && let Some(value_declaration) = prop.value_declaration.as_ref()
            && (!self.is_auto_typed_property(prop)
                || value_declaration.syntactic_modifier_flags().contains(ModifierFlags::Ambient))
        {
            if let Some(base_type) = self.get_type_of_property_in_base_class(prop) {
                initial_type = base_type;
            }
        }
        self.get_flow_type_of_reference_ex(reference, &self.auto_type(), Some(&initial_type), None)
    }

    pub fn get_for_in_variable_symbol(&mut self, node: &Node) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_for_in_variable_symbol"); 
        let initializer = node.initializer()?;
        if is_variable_declaration_list(initializer) {
            let declarations = &initializer.as_variable_declaration_list().declarations.nodes;
            if let Some(variable) = declarations.first() {
                if !variable.name().is_some_and(|n| is_binding_pattern(n)) {
                    return self.get_symbol_of_declaration(variable);
                }
            }
        } else if is_identifier(initializer) {
            return self.get_resolved_symbol(initializer);
        }
        None
    }

    pub fn get_fully_qualified_name(&mut self, symbol: &Arc<Symbol>, containing_location: Option<&Node>) -> String { ::tsox_core::fntrace::enter("get_fully_qualified_name"); 
        if let Some(parent) = symbol.parent() {
            return format!("{}.{}", self.get_fully_qualified_name(&parent, containing_location), self.symbol_to_string(symbol));
        }
        self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::DoNotIncludeSymbolChain | SymbolFormatFlags::AllowAnyNodeKind,
            SYMBOL_FLAGS_ALL,
        )
    }

    pub fn get_generic_object_flags(&mut self, t: &Arc<Type>) -> ObjectFlags { ::tsox_core::fntrace::enter("get_generic_object_flags"); 
        let mut combined_flags = ObjectFlags::empty();
        if t.flags.intersects(TypeFlags::UNION_OR_INTERSECTION | TypeFlags::SUBSTITUTION) {
            if !t.object_flags.contains(ObjectFlags::IS_GENERIC_TYPE_COMPUTED) {
                if t.flags.intersects(TypeFlags::UNION_OR_INTERSECTION) {
                    if let TypeData::Union(u) = &t.data {
                        for ty in &u.union_or_intersection.types {
                            combined_flags |= self.get_generic_object_flags(ty);
                        }
                    }
                } else if let TypeData::Substitution(s) = &t.data {
                    if let (Some(base_type), Some(constraint)) = (s.base_type.as_ref(), s.constraint.as_ref()) {
                        combined_flags = self.get_generic_object_flags(base_type) | self.get_generic_object_flags(constraint);
                    }
                }
                return (t.object_flags | ObjectFlags::IS_GENERIC_TYPE_COMPUTED | combined_flags)
                    & ObjectFlags::IS_GENERIC_TYPE;
            }
            return t.object_flags & ObjectFlags::IS_GENERIC_TYPE;
        }
        if t.flags.intersects(TypeFlags::INSTANTIABLE_NON_PRIMITIVE) || self.is_generic_mapped_type(t) || is_generic_tuple_type(t) {
            combined_flags |= ObjectFlags::IS_GENERIC_OBJECT_TYPE;
        }
        if t.flags.intersects(TypeFlags::INSTANTIABLE_NON_PRIMITIVE | TypeFlags::INDEX) || self.is_generic_string_like_type(t) {
            combined_flags |= ObjectFlags::IS_GENERIC_INDEX_TYPE;
        }
        combined_flags
    }

    pub fn get_global_import_meta_expression_type(&mut self) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_import_meta_expression_type"); 
        if deferred_global_import_meta_expression_type_get(self).is_none() {
            // Create a synthetic type `ImportMetaExpression { meta: MetaProperty }`
            let mut symbol = self.new_symbol(SymbolFlags::empty(), "ImportMetaExpression");
            let import_meta_type = self.get_global_import_meta_type();
            let meta_property_symbol = self.new_symbol_ex(SymbolFlags::Property, "meta", CheckFlags::Readonly);
            meta_property_symbol.set_parent(&symbol);
            self.value_symbol_links.get_or_default(&meta_property_symbol).resolved_type = Some(Arc::clone(&import_meta_type));
            let members = create_symbol_table(&[Arc::clone(&meta_property_symbol)]);
            if let Some(symbol_mut) = Arc::get_mut(&mut symbol) {
                symbol_mut.members = members.clone();
            }
            let t = self.new_anonymous_type(&symbol, members, Vec::new(), Vec::new(), Vec::new());
            deferred_global_import_meta_expression_type_set(self, t);
        }
        deferred_global_import_meta_expression_type_get(self).unwrap()
    }

    pub fn get_global_non_nullable_type_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_non_nullable_type_instantiation"); 
        if let Some(alias) = self.get_global_non_nullable_type_alias_or_nil() {
            return self.get_type_alias_instantiation(&alias, &[Arc::clone(t)], None);
        }
        self.get_intersection_type(vec![Arc::clone(t), self.empty_object_type()])
    }

    pub fn get_global_strict_function_type(&mut self, name: &str) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_strict_function_type"); 
        if self.strict_bind_call_apply {
            return self.get_global_type(name, 0, true);
        }
        self.global_function_type()
    }

    pub fn get_global_type(&mut self, name: &str, arity: usize, report_errors: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_type"); 
        let symbol = self.get_global_symbol(
            name,
            SymbolFlags::TYPE,
            if report_errors { Some(&CANNOT_FIND_GLOBAL_TYPE_0) } else { None },
        );
        if let Some(symbol) = symbol {
            if symbol.flags.intersects(SymbolFlags::Class.union(SymbolFlags::Interface)) {
                let t = self.get_declared_type_of_symbol(&symbol);
                let declared_type_parameter_count = match &t.data {
                    TypeData::Interface(i) if !i.all_type_parameters.is_empty() => {
                        i.type_parameters().len()
                    }
                    _ => global_symbol_type_parameter_count(&symbol),
                };
                if declared_type_parameter_count == arity {
                    return t;
                }
                if report_errors {
                    if let Some(decl) = get_global_type_declaration(&symbol) {
                        self.error_message(
                            decl,GLOBAL_TYPE_0_MUST_HAVE_1_TYPE_PARAMETER_S,
                            &[symbol_name(&symbol).to_string(), arity.to_string()],
                        );
                    }
                }
            } else if report_errors {
                if let Some(decl) = get_global_type_declaration(&symbol) {
                    self.error_message(decl,GLOBAL_TYPE_0_MUST_BE_A_CLASS_OR_INTERFACE_TYPE, &[symbol_name(&symbol).to_string()]);
                }
            }
        }
        if arity != 0 {
            return self.empty_generic_type();
        }
        self.empty_object_type()
    }

    pub fn get_global_type_alias_resolver(&self, name: &str, arity: usize, report_errors: bool) -> Box<dyn Fn(&mut Checker) -> Option<Arc<Symbol>> + Send> { ::tsox_core::fntrace::enter("get_global_type_alias_resolver"); 
        let name = name.to_string();
        Box::new(move |c: &mut Checker| c.get_global_type_alias_symbol(&name, arity, report_errors))
    }

    pub fn get_global_type_alias_symbol(&mut self, name: &str, arity: usize, report_errors: bool) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_type_alias_symbol"); 
        let symbol = self.get_global_symbol(
            name,
            SymbolFlags::TypeAlias,
            if report_errors { Some(&CANNOT_FIND_GLOBAL_TYPE_0) } else { None },
        );
        let symbol = symbol?;
        self.get_declared_type_of_symbol(&symbol);
        if self.type_alias_links.get_or_default(&symbol).type_parameters.len() != arity {
            if report_errors {
                let decl = symbol.declarations.iter().find(|d| is_type_alias_declaration(d));
                if let Some(decl) = decl {
                    self.error_message(
                        decl,GLOBAL_TYPE_0_MUST_HAVE_1_TYPE_PARAMETER_S,
                        &[symbol_name(&symbol).to_string(), arity.to_string()],
                    );
                }
            }
            return None;
        }
        Some(symbol)
    }

    pub fn get_global_type_resolver(&self, name: &str, arity: usize, report_errors: bool) -> Box<dyn Fn(&mut Checker) -> Arc<Type> + Send> { ::tsox_core::fntrace::enter("get_global_type_resolver"); 
        let name = name.to_string();
        Box::new(move |c: &mut Checker| c.get_global_type(&name, arity, report_errors))
    }

    pub fn get_global_type_symbol_resolver(&self, name: &str, report_errors: bool) -> Box<dyn Fn(&mut Checker) -> Option<Arc<Symbol>> + Send> { ::tsox_core::fntrace::enter("get_global_type_symbol_resolver"); 
        let name = name.to_string();
        Box::new(move |c: &mut Checker| {
            c.get_global_symbol(name.as_str(), SymbolFlags::TYPE, if report_errors { Some(&CANNOT_FIND_GLOBAL_TYPE_0) } else { None })
        })
    }

    pub fn get_global_types_resolver(&self, names: &[String], arity: usize, report_errors: bool) -> Box<dyn Fn(&mut Checker) -> Vec<Arc<Type>> + Send> { ::tsox_core::fntrace::enter("get_global_types_resolver"); 
        let names: Vec<String> = names.to_vec();
        Box::new(move |c: &mut Checker| {
            names.iter().map(|name| c.get_global_type(name, arity, report_errors)).collect()
        })
    }

    pub fn get_global_value_symbol_resolver(&self, name: &str, report_errors: bool) -> Box<dyn Fn(&mut Checker) -> Option<Arc<Symbol>> + Send> { ::tsox_core::fntrace::enter("get_global_value_symbol_resolver"); 
        let name = name.to_string();
        Box::new(move |c: &mut Checker| {
            c.get_global_symbol(name.as_str(), SymbolFlags::VALUE, if report_errors { Some(&CANNOT_FIND_GLOBAL_VALUE_0) } else { None })
        })
    }

}

pub fn get_entity_name_from_type_node(node: &Node) -> Option<&Node> { ::tsox_core::fntrace::enter("get_entity_name_from_type_node"); 
    match node.kind {
        SyntaxKind::TypeReference => Some(&node.as_type_reference_node().type_name),
        SyntaxKind::ExpressionWithTypeArguments => {
            if let Some(e) = node.expression()
                && is_entity_name_expression_local(e.as_ref())
            {
                return Some(e.as_ref());
            }
            None
        }
        SyntaxKind::Identifier | SyntaxKind::QualifiedName => Some(node),
        _ => None,
    }
}

pub fn get_first_declaration(symbol: &Arc<Symbol>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("get_first_declaration"); 
    symbol.declarations.first()
}

pub fn get_first_non_ambient_class_or_function_declaration(symbol: &Arc<Symbol>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("get_first_non_ambient_class_or_function_declaration"); 
    symbol
        .declarations
        .iter()
        .find(|declaration| {
            (is_class_declaration(declaration)
                || is_function_declaration(declaration) && declaration.body().is_some())
                && !declaration.flags.contains(NodeFlags::Ambient)
        })
        .map(|d| d)
}

pub fn get_global_type_declaration(symbol: &Arc<Symbol>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("get_global_type_declaration"); 
    symbol.declarations.iter().find_map(|declaration| match declaration.kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::InterfaceDeclaration | SyntaxKind::EnumDeclaration | SyntaxKind::TypeAliasDeclaration => {
            Some(declaration)
        }
        _ => None,
    })
}

pub fn global_symbol_type_parameter_count(symbol: &Arc<Symbol>) -> usize { ::tsox_core::fntrace::enter("global_symbol_type_parameter_count"); 
    let mut names: Vec<&str> = Vec::new();
    for declaration in &symbol.declarations {
        match declaration.kind {
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression | SyntaxKind::InterfaceDeclaration => {}
            _ => continue,
        }
        for parameter in tsox_frontend::ast::mig::m3c::type_parameters(declaration) {
            if let Some(name) = parameter.name().map(|n| n.text())
                && !names.contains(&name)
            {
                names.push(name);
            }
        }
    }
    names.len()
}
