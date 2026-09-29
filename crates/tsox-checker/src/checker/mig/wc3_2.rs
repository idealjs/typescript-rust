#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::mig::m2c_3::some_type;
use crate::checker::mig::wc2::r23k3_defs::some_type_self;
use crate::checker::utilities_is_optional_symbol::get_selected_modifier_flags;
use crate::checker::utilities_token_is_identifier_or_keyword::is_tuple_type;
use crate::checker::mig::wc3::{ThisAssignmentDeclarationKind, JSDeclarationKind, symbol_ptr_key, CHECK_FLAGS_NON_UNIFORM_AND_LITERAL, NodeAccessExt};
use tsox_core::core::compiler_options_kinds::ResolutionMode;
use tsox_core::core::compiler_options::ModuleKind;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn is_constraint_position(&mut self, t: &Arc<Type>, node: &Arc<Node>) -> bool {
        let parent = node.parent().unwrap();
        let is_call_or_new = ast::is_call_expression(&parent) || ast::is_new_expression(&parent);
        if ast::is_property_access_expression(&parent) || ast::is_qualified_name(&parent) {
            return true;
        }
        if is_call_or_new && Arc::ptr_eq(&parent.expression().unwrap(), node) {
            return true;
        }
        if ast::is_element_access_expression(&parent)
            && Arc::ptr_eq(&parent.expression().unwrap(), node)
        {
            let argument_expression = parent
                .as_element_access_expression()
                .argument_expression
                .clone();
            let argument_type = self.get_type_of_expression(&argument_expression);
            return !(some_type_self(self, t, |c, x| c.is_generic_type_without_nullable_constraint(x))
                && self.is_generic_index_type(&argument_type));
        }
        false
    }

    pub fn is_constructor_accessible(&mut self, node: &Arc<Node>, signature: Option<&Arc<Signature>>) -> bool {
        let Some(signature) = signature else {
            return true;
        };
        let Some(declaration) = signature.declaration.as_ref() else {
            return true;
        };
        let modifiers =
            get_selected_modifier_flags(declaration, ModifierFlags::NonPublicAccessibilityModifier);
        if modifiers.is_empty() || !ast::is_constructor_declaration(declaration) {
            return true;
        }
        let class_symbol = self
            .get_symbol_of_node(declaration.parent().unwrap().as_ref())
            .unwrap();
        let declaring_class_declaration = tsox_frontend::ast::mig::x4ast::get_class_like_declaration_of_symbol(&class_symbol);
        let declaring_class = self.get_declared_type_of_symbol(&class_symbol);
        if !self.is_node_within_class(node, &declaring_class_declaration.unwrap()) {
            let containing_class = ast::get_containing_class(node);
            if let Some(containing_class) = containing_class {
                if modifiers.intersects(ModifierFlags::Protected) {
                    let containing_class_symbol =
                        self.get_symbol_of_node(containing_class.as_ref()).unwrap();
                    let containing_type = self.get_declared_type_of_symbol(&containing_class_symbol);
                    if self.type_has_protected_accessible_base(&class_symbol, &containing_type) {
                        return true;
                    }
                }
            }
            if modifiers.intersects(ModifierFlags::Private) {
                let declaring_class_str = self.type_to_string(&declaring_class);
                self.error_message(
                    node,msg::CONSTRUCTOR_OF_CLASS_0_IS_PRIVATE_AND_ONLY_ACCESSIBLE_WITHIN_THE_CLASS_DECLARATION,
                    &[declaring_class_str],
                );
            }
            if modifiers.intersects(ModifierFlags::Protected) {
                let declaring_class_str = self.type_to_string(&declaring_class);
                self.error_message(
                    node,msg::CONSTRUCTOR_OF_CLASS_0_IS_PROTECTED_AND_ONLY_ACCESSIBLE_WITHIN_THE_CLASS_DECLARATION,
                    &[declaring_class_str],
                );
            }
            return false;
        }
        true
    }

    pub fn is_constructor_declared_this_property(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> (ThisAssignmentDeclarationKind, Option<Arc<Node>>) {
        if symbol.value_declaration.is_none()
            || !ast::is_binary_expression(symbol.value_declaration.as_ref().unwrap())
        {
            return (ThisAssignmentDeclarationKind::None, None);
        }
        if let Some(kind) = self.this_expando_kinds.get(&symbol_ptr_key(symbol)) {
            let kind = *kind;
            let location = self
                .this_expando_locations
                .get(&symbol_ptr_key(symbol))
                .cloned();
            let Some(location) = location else {
                panic!("location should be cached whenever this expando symbol is cached");
            };
            return (kind, location);
        }
        let mut all_this = true;
        let mut type_annotation: Option<Arc<Node>> = None;
        for declaration in &symbol.declarations {
            if !ast::is_binary_expression(declaration) {
                all_this = false;
                break;
            }
            let bin = declaration.as_binary_expression();
            let left_element_access = bin.left.kind == SyntaxKind::ElementAccessExpression;
            let left_argument_is_literal = left_element_access
                && ast::is_string_or_numeric_literal_like(
                    &bin.left.as_element_access_expression().argument_expression,
                );
            if crate::binder::bind_js_assignment_declarations::get_assignment_declaration_kind(declaration)
                == crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ThisProperty
                && (!left_element_access || left_argument_is_literal)
            {
                if bin.type_node.is_some() {
                    type_annotation = Some(bin.type_node.clone().unwrap());
                }
            } else {
                all_this = false;
                break;
            }
        }
        let mut location: Option<Arc<Node>> = None;
        let mut kind = ThisAssignmentDeclarationKind::None;
        if all_this {
            if let Some(type_annotation) = type_annotation {
                location = Some(type_annotation);
                kind = ThisAssignmentDeclarationKind::Typed;
            } else {
                location = self.get_declaring_constructor(symbol);
                kind = if location.is_none() {
                    ThisAssignmentDeclarationKind::Method
                } else {
                    ThisAssignmentDeclarationKind::Constructor
                };
            }
        }
        self.this_expando_kinds
            .insert(symbol_ptr_key(symbol), kind);
        self.this_expando_locations
            .insert(symbol_ptr_key(symbol), location.clone());
        (kind, location)
    }

    pub fn is_constructor_type(&mut self, t: &Arc<Type>) -> bool {
        if !self.get_signatures_of_type(t, SignatureKind::Construct).is_empty() {
            return true;
        }
        if t.flags.intersects(crate::checker::types_type_id::TYPE_FLAGS_TYPE_VARIABLE) {
            let constraint = self.get_base_constraint_of_type(t);
            return constraint.is_some() && self.is_mixin_constructor_type(&constraint.unwrap());
        }
        false
    }

    pub fn is_context_sensitive_function_like_declaration(&mut self, node: &Arc<Node>) -> bool {
        tsox_frontend::ast::mig::m3f_2::has_context_sensitive_parameters(node)
            || self.has_context_sensitive_return_expression(node)
            || self.has_context_sensitive_yield_expression(node)
    }

    pub fn is_context_sensitive_function_or_object_literal_method(&mut self, fn_node: &Arc<Node>) -> bool {
        (ast::is_function_expression_or_arrow_function(fn_node) || ast::is_object_literal_method(fn_node))
            && self.is_context_sensitive_function_like_declaration(fn_node)
    }

    pub fn is_declaration_contained_by(
        &self,
        symbol: &Arc<Symbol>,
        container: &Arc<Symbol>,
    ) -> bool {
        if let Some(declaration) = symbol.value_declaration.as_ref() {
            for d in &container.declarations {
                if declaration.loc.contained_by(&d.loc) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_deferred_type(&mut self, t: &Arc<Type>, check_tuples: bool) -> bool {
        self.is_generic_type(t)
            || check_tuples
                && is_tuple_type(t)
                && self
                    .get_element_types(t)
                    .iter()
                    .any(|x| self.is_generic_type(x))
    }

    pub fn is_deferred_type_reference_node(
        &mut self,
        node: &Arc<Node>,
        has_default_type_arguments: bool,
    ) -> bool {
        if self.alias_symbol_for_type_node(node).is_some() {
            return true;
        }
        if self.is_resolved_by_type_alias(node) {
            match node.kind {
                SyntaxKind::ArrayType => {
                    let element_type = node.as_array_type_node().element_type.clone();
                    return self.may_resolve_type_alias(&element_type);
                }
                SyntaxKind::TupleType => {
                    return node
                        .elements()
                        .is_some_and(|l| l.iter().any(|e| self.may_resolve_type_alias(e)));
                }
                SyntaxKind::TypeReference => {
                    return has_default_type_arguments
                        || node
                            .type_arguments()
                            .is_some_and(|l| l.iter().any(|e| self.may_resolve_type_alias(e)));
                }
                _ => panic!("Unhandled case in isDeferredTypeReferenceNode"),
            }
        }
        false
    }

    pub fn is_deprecated_symbol(&mut self, symbol: &Arc<Symbol>) -> bool {
        let parent_symbol = self.get_parent_of_symbol(symbol);
        if parent_symbol.is_some() && symbol.declarations.len() > 1 {
            if parent_symbol.unwrap().flags.intersects(SymbolFlags::Interface) {
                return symbol
                    .declarations
                    .iter()
                    .any(|d| self.is_deprecated_declaration(d));
            } else {
                return symbol
                    .declarations
                    .iter()
                    .all(|d| self.is_deprecated_declaration(d));
            }
        }
        (symbol.value_declaration.is_some()
            && self.is_deprecated_declaration(symbol.value_declaration.as_ref().unwrap()))
            || (!symbol.declarations.is_empty()
                && symbol
                    .declarations
                    .iter()
                    .all(|d| self.is_deprecated_declaration(d)))
    }

    pub fn is_discriminant_with_never_type(&mut self, prop: &Arc<Symbol>) -> bool {
        !prop.flags.intersects(SymbolFlags::Optional)
            && prop.check_flags.intersects(CHECK_FLAGS_NON_UNIFORM_AND_LITERAL)
            && !prop.check_flags.intersects(CheckFlags::HasNeverType)
            && self.get_type_of_symbol(prop).flags.contains(TypeFlags::Never)
    }

    pub fn is_empty_resolved_type(&mut self, t: &Arc<Type>) -> bool {
        !Arc::ptr_eq(t, &self.any_function_type())
            && t.as_structured().map_or(true, |s| {
                s.properties.is_empty() && s.signatures.is_empty() && s.index_infos.is_empty()
            })
    }

    pub fn is_exact_optional_property_mismatch(
        &mut self,
        source: Option<&Arc<Type>>,
        target: Option<&Arc<Type>>,
    ) -> bool {
        match (source, target) {
            (Some(source), Some(target)) => {
                self.maybe_type_of_kind(source, TypeFlags::Undefined)
                    && crate::checker::mig::wc3::r24k3_defs::contains_missing_type(self, target)
            }
            _ => false,
        }
    }

    pub fn is_excluded_mapped_property_name(
        &mut self,
        t: &Arc<Type>,
        property_name_type: &Arc<Type>,
    ) -> bool {
        if t.flags.contains(TypeFlags::Conditional) {
            let true_type = self.get_true_type_from_conditional_type(t);
            let false_type = self.get_false_type_from_conditional_type(t);
            let check_type = t.as_conditional_type().unwrap().check_type.clone();
            let extends_type = t.as_conditional_type().unwrap().extends_type.clone();
            return self
                .get_reduced_type(true_type.as_ref().unwrap())
                .flags
                .contains(TypeFlags::Never)
                && Arc::ptr_eq(
                    &self.get_actual_type_variable(false_type.as_ref().unwrap()),
                    &self.get_actual_type_variable(check_type.as_ref().unwrap()),
                )
                && self.is_type_assignable_to(property_name_type, extends_type.as_ref().unwrap());
        }
        if t.flags.contains(TypeFlags::Intersection) {
            return t
                .types()
                .unwrap()
                .iter()
                .any(|x| self.is_excluded_mapped_property_name(x, property_name_type));
        }
        false
    }
}

pub fn is_const_enum_object_type(t: &Arc<Type>) -> bool {
    t.object_flags.intersects(ObjectFlags::Anonymous)
        && t.symbol().is_some()
        && is_const_enum_symbol(t.symbol().as_ref().unwrap())
}

pub fn is_const_enum_symbol(symbol: &Symbol) -> bool {
    symbol.flags.intersects(SymbolFlags::ConstEnum)
}

pub fn is_contained_by_namespace(node: &Arc<Node>) -> bool {
    let mut container = node.parent().unwrap();
    if !ast::is_source_file(&container) {
        container = container.parent().unwrap();
    }
    ast::is_module_declaration(&container) && !ast::is_ambient_module(&container)
}

pub fn is_es2015_or_later_iterable(n: &str) -> bool {
    matches!(
        n,
        "Float32Array"
            | "Float64Array"
            | "Int16Array"
            | "Int32Array"
            | "Int8Array"
            | "NodeList"
            | "Uint16Array"
            | "Uint32Array"
            | "Uint8Array"
            | "Uint8ClampedArray"
    )
}

pub fn is_esm_format_import_importing_commonjs_format_file(
    usage_mode: ResolutionMode,
    target_mode: ResolutionMode,
) -> bool {
    usage_mode == ModuleKind::ESNext && target_mode == ModuleKind::CommonJS
}

pub fn is_export_or_export_expression(location: &Arc<Node>) -> bool {
    ast::find_ancestor(location, |n: &Node| {
        let Some(parent) = n.parent() else {
            return false;
        };
        if tsox_frontend::ast::mig::x6a::is_any_export_assignment(&parent) {
            return parent.expression().is_some_and(|e| e.id() == n.id())
                && ast::is_entity_name_expression(n);
        }
        if ast::is_export_specifier(&parent) {
            return parent.as_export_specifier().name.id() == n.id()
                || parent
                    .as_export_specifier()
                    .property_name
                    .as_ref()
                    .is_some_and(|p| p.id() == n.id());
        }
        false
    })
    .is_some()
}
