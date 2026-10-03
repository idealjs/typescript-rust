#![allow(unused_imports)]
#[path = "r19k11_defs.rs"]
pub mod r19k11_defs;
pub use r19k11_defs::*;
#[path = "r20k6_defs.rs"]
pub mod r20k6_defs;
pub use r20k6_defs::*;
#[path = "r18k8_flags.rs"]
pub mod r18k8_flags;
pub use r18k8_flags::*;
use tsox_core::jsnum::PseudoBigInt;
use tsox_core::tspath::get_declaration_file_extension;
use tsox_frontend::ast::mig::m3f::get_source_file_of_module;
use tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration;
use crate::checker::utilities_is_optional_symbol::is_numeric_literal_name;
use crate::checker::mig::m1d_4::get_big_int_literal_value;
use crate::checker::mig::wc1b::is_tuple_type;
use crate::checker::mig::wc2_2::get_mapped_type_modifiers;
use crate::checker::mig::wc3::{is_conflicting_private_property, MappedTypeModifiers};
use crate::checker::mig::wc3_3::is_generic_tuple_type;
use crate::checker::mig::wc3::r18k4_node_ext::NodeAccessExt;

use crate::checker::checker_checker::*;
use std::sync::Arc;
use tsox_frontend::ast::{self, is_identifier, Node, Symbol, SyntaxKind};

pub fn is_mutable_tuple_type(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_mutable_tuple_type"); 
    is_tuple_type(t) && !t.target_tuple_type().unwrap().readonly
}

pub fn is_neither_unit_type_nor_never(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_neither_unit_type_nor_never"); 
    !t.flags.intersects(TYPE_FLAGS_UNIT) && !t.flags.contains(TypeFlags::Never)
}

pub fn is_non_deferred_type_reference(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_non_deferred_type_reference"); 
    t.object_flags.intersects(ObjectFlags::Reference) && t.as_type_reference().is_some()
}

pub fn is_not_null_type(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_not_null_type"); 
    !t.flags.contains(TypeFlags::Null)
}

pub fn is_not_undefined_type(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_not_undefined_type"); 
    !t.flags.contains(TypeFlags::Undefined)
}

pub fn is_not_overload(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_not_overload"); 
    (!ast::is_function_declaration(node) && !ast::is_method_declaration(node))
        || node.body().is_some()
}

pub fn is_primitive_type_name(s: &str) -> bool { ::tsox_core::fntrace::enter("is_primitive_type_name"); 
    s == "any" || s == "string" || s == "number" || s == "boolean" || s == "never" || s == "unknown"
}

pub fn is_prototype_property(symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_prototype_property"); 
    symbol.flags.intersects(SymbolFlags::Method)
        || symbol.check_flags.intersects(CHECK_FLAGS_SYNTHETIC_METHOD)
}

pub fn is_property_immediately_referenced_within_declaration(
    declaration: &Arc<Node>,
    usage: &Arc<Node>,
    stop_at_any_property_declaration: bool,
) -> bool { ::tsox_core::fntrace::enter("is_property_immediately_referenced_within_declaration"); 
    if usage.end() > declaration.end() {
        return false;
    }
    let mut node = Some(Arc::clone(usage));
    while let Some(n) = node {
        if Arc::ptr_eq(&n, declaration) {
            break;
        }
        match n.kind {
            SyntaxKind::ArrowFunction => return false,
            SyntaxKind::PropertyDeclaration => {
                let node_parent = n.parent();
                let decl_parent = declaration.parent();
                if !stop_at_any_property_declaration {
                    return false;
                }
                let parent_matches = matches!((&node_parent, &decl_parent),
                    (Some(np), Some(dp)) if Arc::ptr_eq(np, dp));
                let parent_of_parent_matches = matches!(
                    (&node_parent, decl_parent.as_ref().and_then(|dp| dp.parent())),
                    (Some(np), Some(g)) if Arc::ptr_eq(np, &g)
                );
                return (ast::is_property_declaration(declaration) && parent_matches)
                    || (is_parameter_property_declaration(
                        declaration,
                        decl_parent.as_ref().unwrap(),
                    ) && parent_of_parent_matches);
            }
            SyntaxKind::Block => {
                if let Some(np) = n.parent() {
                    if matches!(
                        np.kind,
                        SyntaxKind::MethodDeclaration
                            | SyntaxKind::GetAccessor
                            | SyntaxKind::SetAccessor
                    ) {
                        return false;
                    }
                }
            }
            _ => {}
        }
        node = n.parent();
    }
    true
}

pub fn is_part_of_import_equals_module_reference(location: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_part_of_import_equals_module_reference"); 
    let Some(import_equals) = ast::find_ancestor_kind(location, SyntaxKind::ImportEqualsDeclaration)
    else {
        return false;
    };
    let mut node = Some(Arc::clone(location));
    while let Some(n) = node {
        if Arc::ptr_eq(&n, &import_equals) {
            break;
        }
        let module_reference = &import_equals.as_import_equals_declaration().module_reference;
        if Arc::ptr_eq(module_reference, &n) {
            return true;
        }
        node = n.parent();
    }
    false
}

pub fn is_partial_mapped_type(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_partial_mapped_type"); 
    t.object_flags.intersects(ObjectFlags::Mapped)
        && get_mapped_type_modifiers(t)
            .intersects(MappedTypeModifiers::IncludeOptional)
}

pub fn is_rest_parameter(param: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_rest_parameter"); 
    param.as_parameter_declaration().dot_dot_dot_token.is_some()
}

pub fn is_single_element_generic_tuple_type(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_single_element_generic_tuple_type"); 
    is_generic_tuple_type(t) && t.target_tuple_type().unwrap().element_infos.len() == 1
}

pub fn is_spread_argument(arg: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_spread_argument"); 
    ast::is_spread_element(arg)
        || (ast::is_synthetic_expression(arg) && arg.as_synthetic_expression().is_spread)
}

pub fn is_type_reference_with_generic_arguments(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_type_reference_with_generic_arguments"); 
    is_non_deferred_type_reference(t)
        && t.as_type_reference()
            .map(|r| {
                r.type_arguments
                    .iter()
                    .any(|t| t.flags.contains(TypeFlags::TypeParameter) || is_type_reference_with_generic_arguments(t))
            })
            .unwrap_or(false)
}

pub fn is_unconstrained_type_parameter(tp: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_unconstrained_type_parameter"); 
    let target = tp.target().unwrap_or(tp);
    if target.symbol.is_none() {
        return false;
    }
    target.symbol.as_ref().unwrap().declarations.iter().all(|d| {
        !(ast::is_type_parameter_declaration(d)
            && (d.as_type_parameter_declaration().constraint.is_some()
                || ast::is_mapped_type_node(&d.parent().unwrap())
                || ast::is_infer_type_node(&d.parent().unwrap())))
    })
}

pub fn is_zero_bigint(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_zero_bigint"); 
    get_big_int_literal_value(t).is_zero()
}

pub fn is_thisless(symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_thisless"); 
    if symbol.declarations.len() == 1 {
        let declaration = &symbol.declarations[0];
        match declaration.kind {
            SyntaxKind::Parameter
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature => is_thisless_variable_like_declaration(declaration),
            SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => is_thisless_function_like_declaration(declaration),
            _ => false,
        }
    } else {
        false
    }
}

pub fn is_thisless_variable_like_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_thisless_variable_like_declaration"); 
    if let Some(type_node) = node.type_() {
        return is_thisless_type(&type_node);
    }
    node.initializer().is_none()
}

pub fn is_thisless_type(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_thisless_type"); 
    match node.kind {
        SyntaxKind::AnyKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::VoidKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::NeverKeyword
        | SyntaxKind::LiteralType => true,
        SyntaxKind::ArrayType => is_thisless_type(&node.as_array_type_node().element_type),
        SyntaxKind::TypeReference => node
            .type_arguments()
            .map(|args| args.iter().all(is_thisless_type))
            .unwrap_or(true),
        _ => false,
    }
}

pub fn is_thisless_function_like_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_thisless_function_like_declaration"); 
    let return_type = node.type_();
    (ast::is_constructor_declaration(node)
        || return_type.as_ref().is_some_and(|t| is_thisless_type(t)))
        && node
            .parameters()
            .map(|params| params.iter().all(is_thisless_variable_like_declaration))
            .unwrap_or(true)
        && node
            .type_parameters()
            .map(|tps| tps.iter().all(is_thisless_type_parameter))
            .unwrap_or(true)
}

pub fn is_thisless_type_parameter(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_thisless_type_parameter"); 
    let constraint = &node.as_type_parameter_declaration().constraint;
    constraint.is_none() || constraint.as_ref().is_some_and(|c| is_thisless_type(c))
}

pub fn is_unary_tuple_type_node(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_unary_tuple_type_node"); 
    ast::is_tuple_type_node(node)
        && node
            .elements()
            .map(|elements| elements.nodes.len() == 1)
            .unwrap_or(false)
}

impl Checker {
    pub fn is_not_replacable_by_method(&mut self, decl: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_not_replacable_by_method"); 
        !self
            .get_symbol_of_declaration(decl)
            .map(|s| s.flags.intersects(SymbolFlags::ReplaceableByMethod))
            .unwrap_or(false)
    }

    pub fn is_never_reduced_property(&mut self, prop: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_never_reduced_property"); 
        self.is_discriminant_with_never_type(prop) || is_conflicting_private_property(prop)
    }

    pub fn is_node_used_during_class_initialization(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_node_used_during_class_initialization"); 
        let mut current = Some(Arc::clone(node));
        while let Some(element) = current {
            if (ast::is_constructor_declaration(&element) && element.body().is_some())
                || ast::is_property_declaration(&element)
            {
                return true;
            }
            if ast::is_class_like(&element) || ast::is_function_like_declaration(&element) {
                return false;
            }
            current = element.parent();
        }
        false
    }

    pub fn is_non_generic_object_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_non_generic_object_type"); 
        t.flags.contains(TypeFlags::Object) && !self.is_generic_mapped_type(t)
    }

    pub fn is_null_or_undefined(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_null_or_undefined"); 
        let expr = ast::skip_parentheses(node);
        match expr.kind {
            SyntaxKind::NullKeyword => true,
            SyntaxKind::Identifier => self
                .get_resolved_symbol(&expr)
                .is_some_and(|s| self.undefined_symbol.as_ref().is_some_and(|u| Arc::ptr_eq(&s, u))),
            _ => false,
        }
    }

    pub fn is_numeric_computed_name(&mut self, name: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_numeric_computed_name"); 
        let t = self.check_computed_property_name_type(name);
        self.is_type_assignable_to_kind(&t, TYPE_FLAGS_NUMBER_LIKE)
    }

    pub fn is_numeric_name(&mut self, name: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_numeric_name"); 
        match name.kind {
            SyntaxKind::ComputedPropertyName => self.is_numeric_computed_name(name),
            SyntaxKind::Identifier
            | SyntaxKind::NumericLiteral
            | SyntaxKind::StringLiteral => is_numeric_literal_name(&name.text()),
            _ => false,
        }
    }

    pub fn is_only_importable_as_default(
        &mut self,
        usage: &Arc<Node>,
        resolved_module: Option<&Arc<Symbol>>,
        import_attributes_type: Option<&Arc<Type>>,
    ) -> bool { ::tsox_core::fntrace::enter("is_only_importable_as_default"); 
        if MODULE_KIND_NODE16 <= self.module_kind && self.module_kind <= MODULE_KIND_NODE_NEXT {
            let usage_mode = self.get_emit_syntax_for_module_specifier_expression(usage);
            if usage_mode == MODULE_KIND_ES_NEXT {
                let resolved_module = match resolved_module {
                    Some(m) => Some(Arc::clone(m)),
                    None => self.resolve_external_module_name_worker(
                        usage,
                        Some(usage),
                        None,
                        true,
                        false,
                        import_attributes_type,
                    ),
                };
                let target_file = resolved_module
                    .as_ref()
                    .and_then(|m| get_source_file_of_module(m));
                return target_file
                    .and_then(|file| self.get_source_file_of_node(&file))
                    .is_some_and(|sf| {
                        ast::is_json_source_file(&sf)
                            || get_declaration_file_extension(&sf.file_name) == ".d.json.ts"
                    });
            }
        }
        false
    }
}
