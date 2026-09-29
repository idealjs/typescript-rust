#![allow(unused_imports)]
use crate::checker::mig::m2a::r19k11_defs::*;
use crate::checker::mig::m2a::r18k8_flags::*;
use tsox_frontend::ast::mig::m3h::walk_up_binding_elements_and_patterns;
use crate::checker::mig::m2c_3::signature_has_rest_parameter;
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol;
use crate::checker::utilities_has_only_expression_initialization::is_known_symbol;
use crate::checker::utilities_is_optional_symbol::is_numeric_literal_name;
use crate::checker::mig::m1d_4::get_boolean_literal_value;
use crate::checker::mig::wc3::r18k4_node_ext::NodeAccessExt;

use crate::checker::checker_checker::*;
use crate::checker::types::Signature;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

#[path = "r28k5_defs.rs"]
pub(crate) mod r28k5_defs;

impl Checker {
    pub fn is_parameter_of_context_sensitive_signature(&mut self, symbol: &Arc<Symbol>) -> bool {
        let Some(mut decl) = symbol.value_declaration.clone() else {
            return false;
        };
        if ast::is_binding_element(&decl) {
            if let Some(d) = walk_up_binding_elements_and_patterns(&decl) {
                decl = d;
            }
        }
        if ast::is_parameter_declaration(&decl) {
            return decl
                .parent()
                .is_some_and(|p| self.is_context_sensitive_function_or_object_literal_method(&p));
        }
        false
    }

    pub fn is_potentially_uncalled_decorator(
        &mut self,
        decorator: &Arc<Node>,
        signatures: &[Arc<Signature>],
    ) -> bool {
        !signatures.is_empty()
            && signatures.iter().all(|sig| {
                sig.min_argument_count == 0
                    && !signature_has_rest_parameter(sig)
                    && sig.parameters.len() < self.get_decorator_argument_count(decorator, sig)
            })
    }

    pub fn is_promise_resolve_arity_error(&mut self, node: &Arc<Node>) -> bool {
        let expr = node.expression();
        if !ast::is_call_expression(node) || !expr.as_ref().is_some_and(|e| ast::is_identifier(e)) {
            return false;
        }
        let expr = expr.unwrap();
        let symbol = self.resolve_name(&expr.text(), &expr, SymbolFlags::VALUE, false);
        let Some(symbol) = symbol else {
            return false;
        };
        let Some(decl) = symbol.value_declaration.clone() else {
            return false;
        };
        let parent = decl.parent();
        let grandparent = parent.as_ref().and_then(|p| p.parent());
        if !ast::is_parameter_declaration(&decl)
            || !parent.as_ref().is_some_and(|p| ast::is_function_expression_or_arrow_function(p))
            || !grandparent.as_ref().is_some_and(|gp| ast::is_new_expression(gp))
            || !grandparent
                .as_ref()
                .and_then(|gp| gp.expression())
                .is_some_and(|e| ast::is_identifier(&e))
        {
            return false;
        }
        let Some(global_promise_symbol) = crate::checker::mig::m1c::r25k9_defs::Checker::get_global_promise_constructor_symbol_or_nil(self) else {
            return false;
        };
        let gp = grandparent.unwrap();
        let constructor_expr = gp.expression().unwrap();
        self.get_resolved_symbol(&constructor_expr)
            .is_some_and(|s| Arc::ptr_eq(&s, &global_promise_symbol))
    }


    pub fn is_property_identical_to(
        &mut self,
        source_prop: &Arc<Symbol>,
        target_prop: &Arc<Symbol>,
    ) -> bool {
        self.compare_properties(
            source_prop,
            target_prop,
            &|c: &mut Checker, source: &Arc<Type>, target: &Arc<Type>| {
                c.compare_types_identical(source, target)
            },
        ) != Ternary::False
    }

    pub fn is_property_in_class_derived_from(
        &mut self,
        prop: &Arc<Symbol>,
        base_class: &Arc<Type>,
    ) -> bool {
        self.for_each_property(prop, &mut |c: &mut Checker, sp: &Arc<Symbol>| {
            if let Some(source_class) = c.get_declaring_class(sp) {
                return c.has_base_type(&source_class, base_class);
            }
            false
        })
    }

    pub fn is_readonly_array_symbol(&mut self, symbol: Option<&Arc<Symbol>>) -> bool {
        let Some(symbol) = symbol else {
            return false;
        };
        let Some(target) = self.global_readonly_array_type.get().and_then(|t| t.symbol.clone()) else {
            return false;
        };
        self.get_symbol_if_same_reference(symbol, &target)
            .is_some()
    }

    pub fn is_readonly_array_type(&mut self, t: &Arc<Type>) -> bool {
        self.global_readonly_array_type.get().is_some_and(|global| {
            t.object_flags.intersects(ObjectFlags::Reference)
                && t.target().is_some_and(|target| Arc::ptr_eq(target, global))
        })
    }

    pub fn is_readonly_assignment_declaration(&mut self, node: &Arc<Node>) -> bool {
        if !ast::is_call_expression(node) {
            return false;
        }
                let Some(arguments) = node.arguments() else {
            return false;
        };
        let property_descriptor_type = self.check_expression_cached(&arguments.nodes[2]);
        if self
            .get_type_of_property_of_type(&property_descriptor_type, "value")
            .is_some()
        {
            if let Some(writable_prop) =
                self.get_property_of_type(&property_descriptor_type, "writable")
            {
                let writable_type = if writable_prop
                    .value_declaration
                    .as_ref()
                    .is_some_and(|d| ast::is_property_assignment(d))
                {
                    let initializer = writable_prop
                        .value_declaration
                        .as_ref()
                        .unwrap()
                        .initializer()
                        .unwrap();
                    self.check_expression_cached(&initializer)
                } else {
                    self.get_type_of_symbol(&writable_prop)
                };
                return writable_type.flags.contains(TypeFlags::BooleanLiteral)
                    && !get_boolean_literal_value(&writable_type);
            }
            return true;
        }
        self.get_type_of_property_of_type(&property_descriptor_type, "set")
            .is_none()
    }


    pub fn is_readonly_type_operator(&mut self, node: &Arc<Node>) -> bool {
        ast::is_type_operator_node(node)
            && node.as_type_operator_node().operator == SyntaxKind::ReadonlyKeyword
    }

    pub fn is_reducible_intersection(&mut self, t: &Arc<Type>) -> bool {
        if let Some(d) = t.as_intersection_type() {
            if d.unique_literal_filled_instantiation.get().is_none() {
                let mapper = self.unique_literal_mapper();
                let instantiated = self.instantiate_type(t, Some(&mapper));
                d.unique_literal_filled_instantiation
                    .set(instantiated)
                    .ok();
            }
            if let Some(instantiated) = d.unique_literal_filled_instantiation.get() {
                let instantiated = Arc::clone(instantiated);
                let reduced = self.get_reduced_type(&instantiated);
                return !Arc::ptr_eq(&reduced, &instantiated);
            }
        }
        false
    }

    pub fn is_reference_to_some_type(&mut self, t: &Arc<Type>, targets: &[Arc<Type>]) -> bool {
        t.object_flags.intersects(ObjectFlags::Reference)
            && t.target()
                .is_some_and(|target| targets.iter().any(|x| Arc::ptr_eq(x, &target)))
    }

    pub fn is_reference_to_type(&mut self, t: &Arc<Type>, target: &Arc<Type>) -> bool {
        t.object_flags.intersects(ObjectFlags::Reference)
            && t.target().is_some_and(|x| Arc::ptr_eq(&x, target))
    }

    pub fn is_referenced(&mut self, symbol: &Arc<Symbol>) -> bool {
        !self
            .symbol_reference_links
            .get(symbol)
            .map(|links| links.reference_kinds.is_empty())
            .unwrap_or(true)
    }

    pub fn is_resolved_by_type_alias(&mut self, node: &Arc<Node>) -> bool {
        if let Some(parent) = node.parent() {
            match parent.kind {
                SyntaxKind::ParenthesizedType
                | SyntaxKind::NamedTupleMember
                | SyntaxKind::TypeReference
                | SyntaxKind::UnionType
                | SyntaxKind::IntersectionType
                | SyntaxKind::IndexedAccessType
                | SyntaxKind::ConditionalType
                | SyntaxKind::TypeOperator
                | SyntaxKind::ArrayType
                | SyntaxKind::TupleType => return self.is_resolved_by_type_alias(&parent),
                SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration => {
                    return true
                }
                _ => {}
            }
        }
        false
    }

    pub fn is_return_iterator_result(&mut self, t: &Arc<Type>) -> bool {
        crate::checker::mig::m1c::r25k9_defs::Checker::is_iterator_result(self, t, IterationTypeKind::RETURN)
    }

    pub fn is_yield_iterator_result(&mut self, t: &Arc<Type>) -> bool {
        crate::checker::mig::m1c::r25k9_defs::Checker::is_iterator_result(self, t, IterationTypeKind::YIELD)
    }

    pub fn is_same_scoped_binding_element(
        &mut self,
        node: &Arc<Node>,
        declaration: &Arc<Node>,
    ) -> bool {
        if ast::is_binding_element(declaration) {
            return ast::find_ancestor(node, ast::is_binding_element).is_some_and(|be| {
                let a = ast::get_root_declaration(&be);
                let b = ast::get_root_declaration(declaration);
                Arc::ptr_eq(&a, &b)
            });
        }
        false
    }

    pub fn is_side_effect_free(&mut self, node: &Arc<Node>) -> bool {
        let node = ast::skip_parentheses(node);
        match node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::TemplateExpression
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ClassExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::TypeOfExpression
            | SyntaxKind::NonNullExpression
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxElement => true,
            SyntaxKind::ConditionalExpression => {
                let ce = node.as_conditional_expression();
                self.is_side_effect_free(&ce.when_true) && self.is_side_effect_free(&ce.when_false)
            }
            SyntaxKind::BinaryExpression => {
                let be = node.as_binary_expression();
                if ast::is_assignment_operator(be.operator_token.kind) {
                    false
                } else {
                    self.is_side_effect_free(&be.left) && self.is_side_effect_free(&be.right)
                }
            }
            SyntaxKind::PrefixUnaryExpression => matches!(
                node.as_prefix_unary_expression().operator,
                SyntaxKind::ExclamationToken
                    | SyntaxKind::PlusToken
                    | SyntaxKind::MinusToken
                    | SyntaxKind::TildeToken
            ),
            _ => false,
        }
    }


    pub fn is_some_symbol_assigned(&mut self, root_declaration: &Arc<Node>) -> bool {
        let Some(name) = root_declaration.name() else {
            return false;
        };
        self.is_some_symbol_assigned_worker(&name)
    }

    pub fn is_some_symbol_assigned_worker(&mut self, node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::Identifier {
            let parent = node.parent().unwrap();
            return self
                .get_symbol_of_declaration(&parent)
                .is_some_and(|symbol| self.is_symbol_assigned(&symbol));
        }
        node.elements()
            .map(|elements| {
                elements
                    .nodes
                    .iter()
                    .any(|e| e.name().is_some_and(|n| self.is_some_symbol_assigned_worker(&n)))
            })
            .unwrap_or(false)
    }

    pub fn is_string_index_signature_only_type_worker(&mut self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Object)
            && !self.is_generic_mapped_type(t)
            && self.get_properties_of_type(t).is_empty()
            && self.get_index_infos_of_type(t).len() == 1
        {
            let string_type = self.string_type.get().cloned();
            return string_type.is_some_and(|st| self.get_index_info_of_type(t, &st).is_some());
        }
        t.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            && t.types().is_some_and(|types| {
                types
                    .iter()
                    .all(|x| self.is_string_index_signature_only_type_worker(x))
            })
    }

    fn symbol_used_in_binary_chain_visit(
        &mut self,
        child: &Arc<Node>,
        tested_symbol: &Arc<Symbol>,
    ) -> bool {
        if ast::is_identifier(child) {
            if let Some(symbol) = self.get_symbol_at_location(child) {
                if Arc::ptr_eq(&symbol, tested_symbol) {
                    return true;
                }
            }
        }
        let mut result = false;
        child.for_each_child(|c: &Arc<Node>| {
            if !result {
                result = self.symbol_used_in_binary_chain_visit(c, tested_symbol);
            }
            result
        });
        result
    }

    pub fn is_symbol_used_in_binary_expression_chain(
        &mut self,
        node: &Arc<Node>,
        tested_symbol: &Arc<Symbol>,
    ) -> bool {
        let mut current = Some(Arc::clone(node));
        while let Some(n) = current {
            if !ast::is_binary_expression(&n)
                || n.as_binary_expression().operator_token.kind
                    != SyntaxKind::AmpersandAmpersandToken
            {
                break;
            }
            let right = n.as_binary_expression().right.clone();
            if self.symbol_used_in_binary_chain_visit(&right, tested_symbol) {
                return true;
            }
            current = n.parent();
        }
        false
    }

    pub fn is_symbol_with_computed_name(&mut self, symbol: &Arc<Symbol>) -> bool {
        if !symbol.declarations.is_empty() {
            return symbol.declarations[0]
                .name()
                .is_some_and(|n| ast::is_computed_property_name(&n));
        }
        false
    }

    pub fn is_symbol_with_numeric_name(&mut self, symbol: &Arc<Symbol>) -> bool {
        if is_numeric_literal_name(&symbol.name) {
            return true;
        }
        if !symbol.declarations.is_empty() {
            return symbol.declarations[0]
                .name()
                .is_some_and(|n| self.is_numeric_name(&n));
        }
        false
    }

    pub fn is_symbol_with_symbol_name(&mut self, symbol: &Arc<Symbol>) -> bool {
        if is_known_symbol(symbol) {
            return true;
        }
        if !symbol.declarations.is_empty() {
            return symbol.declarations[0].name().is_some_and(|n| {
                ast::is_computed_property_name(&n) && {
                    let t = self.check_computed_property_name_type(&n);
                    self.is_type_assignable_to_kind(&t, TYPE_FLAGS_ES_SYMBOL)
                }
            });
        }
        false
    }

    pub fn is_template_literal_context(&mut self, node: &Arc<Node>) -> bool {
        node.parent().is_some_and(|parent| {
            (ast::is_parenthesized_expression(&parent) && self.is_template_literal_context(&parent))
                || (ast::is_element_access_expression(&parent)
                    && Arc::ptr_eq(&parent.as_element_access_expression().argument_expression, node))
        })
    }

    pub fn is_template_literal_contextual_type(&mut self, t: &Arc<Type>) -> bool {
        t.flags
            .intersects(TYPE_FLAGS_STRING_LITERAL | TYPE_FLAGS_TEMPLATE_LITERAL)
            || (t.flags.intersects(TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE) && {
                let base = self
                    .get_base_constraint_of_type(t)
                    .unwrap_or_else(|| self.unknown_type());
                self.maybe_type_of_kind(&base, TYPE_FLAGS_STRING_LIKE)
            })
    }
}
