#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::relater::RelationKind;
use crate::checker::relater_relation::Relation;
use std::sync::Arc;
use tsox_core::diagnostics as msg;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

thread_local! {
    static NON_EXISTENT_PROPERTIES: std::cell::RefCell<std::collections::HashSet<(usize, usize, bool)>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

pub fn non_existent_properties_contains(key: &(usize, usize, bool)) -> bool {
    NON_EXISTENT_PROPERTIES.with(|s| s.borrow().contains(key))
}

pub fn non_existent_properties_insert(key: (usize, usize, bool)) {
    NON_EXISTENT_PROPERTIES.with(|s| {
        s.borrow_mut().insert(key);
    });
}

pub fn is_expression_node_r24k4(node: &Arc<Node>) -> bool {
    use SyntaxKind::*;
    matches!(
        node.kind,
        Identifier
            | StringLiteral
            | NumericLiteral
            | NoSubstitutionTemplateLiteral
            | TemplateExpression
            | ThisKeyword
            | SuperKeyword
            | TrueKeyword
            | FalseKeyword
            | NullKeyword
            | ArrayLiteralExpression
            | ObjectLiteralExpression
            | PropertyAccessExpression
            | ElementAccessExpression
            | CallExpression
            | NewExpression
            | BinaryExpression
            | PrefixUnaryExpression
            | PostfixUnaryExpression
            | ConditionalExpression
            | ArrowFunction
            | FunctionExpression
            | ClassExpression
            | ParenthesizedExpression
            | NonNullExpression
            | AsExpression
            | SatisfiesExpression
            | TypeAssertionExpression
            | AwaitExpression
            | DeleteExpression
            | TypeOfExpression
            | VoidExpression
            | YieldExpression
            | TaggedTemplateExpression
            | JsxElement
            | JsxSelfClosingElement
            | JsxExpression
            | MetaProperty
    )
}

impl Checker {
    pub fn subtype_relation(&self) -> Relation {
        Relation::new(RelationKind::Subtype)
    }

    pub fn enum_number_index_info(&mut self) -> Arc<IndexInfo> {
        self.new_index_info(&self.string_type(), &self.number_type(), false, None, &[])
    }

    pub fn check_type_related_to_ex(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        error_node: Option<&Arc<Node>>,
        head_message: Option<&msg::Message>,
        diagnostic_output: Option<&mut Vec<ast::Diagnostic>>,
    ) -> bool {
        self.check_type_related_to_and_optionally_elaborate(
            source, target, relation, error_node, None, head_message, diagnostic_output,
        )
    }

    pub fn resolve_qualified_name(
        &mut self,
        name: &Arc<Node>,
        left: &Arc<Node>,
        right: &Arc<Node>,
        meaning: SymbolFlags,
        ignore_errors: bool,
        location: Option<&Arc<Node>>,
    ) -> Option<Arc<Symbol>> {
        let namespace = self.resolve_entity_name(left, SymbolFlags::NAMESPACE, ignore_errors, false, location);
        let namespace = namespace?;
        if ast::node_is_missing(Some(right)) {
            return None;
        }
        if self
            .unknown_symbol
            .as_ref()
            .is_some_and(|u| Arc::ptr_eq(&namespace, u))
        {
            return Some(namespace);
        }
        let text = right.text();
        let exports = self.get_exports_of_symbol(&namespace);
        let mut symbol = exports
            .get(&text)
            .filter(|s| s.flags.intersects(meaning))
            .map(|s| self.get_merged_symbol(s));
        if symbol.is_none() && namespace.flags.intersects(SymbolFlags::Alias) {
            let resolved = self.resolve_alias(&namespace);
            let exports = self.get_exports_of_symbol(&resolved);
            symbol = exports
                .get(&text)
                .filter(|s| s.flags.intersects(meaning))
                .map(|s| self.get_merged_symbol(s));
        }
        if symbol.is_none() && !ignore_errors {
            let namespace_name = self.get_fully_qualified_name(&namespace, None);
            let declaration_name = tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(right));
            if let Some(suggestion) = self.get_suggested_symbol_for_nonexistent_module(right, &namespace) {
                let suggestion_name = self.symbol_to_string(&suggestion);
                self.error_message(
                    right,msg::X_0_HAS_NO_EXPORTED_MEMBER_NAMED_1_DID_YOU_MEAN_2,
                    &[
                        namespace_name,
                        declaration_name,
                        suggestion_name,
                    ],
                );
                return None;
            }
            self.error_message(
                right,msg::NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                &[namespace_name, declaration_name],
            );
        }
        let _ = name;
        symbol
    }

    pub fn mark_property_alias_referenced(
        &mut self,
        location: &Arc<Node>,
        prop_symbol: Option<&Arc<Symbol>>,
        parent_type: Option<&Arc<Type>>,
    ) {
        if crate::checker::mig::m2a::is_part_of_import_equals_module_reference(location) {
            return;
        }
        let left = if ast::is_property_access_expression(location) {
            location.expression()
        } else {
            match &location.data {
                tsox_frontend::ast::NodeData::QualifiedName(d) => Some(&d.left),
                _ => None,
            }
        };
        let Some(left) = left else { return };
        if left.kind == SyntaxKind::ThisKeyword || !ast::is_identifier(left) {
            return;
        }
        let Some(parent_symbol) = self.get_resolved_symbol(left) else {
            return;
        };
        if self
            .unknown_symbol
            .as_ref()
            .is_some_and(|u| Arc::ptr_eq(&parent_symbol, u))
        {
            return;
        }
        if self.compiler_options.get_isolated_modules()
            || (self.compiler_options.should_preserve_const_enums()
                && crate::checker::mig::wc3_2::is_export_or_export_expression(location))
        {
            self.mark_alias_referenced(&parent_symbol, location);
            return;
        }
        let left_type_owned;
        let left_type = match parent_type {
            Some(t) => t,
            None => {
                left_type_owned = self.check_expression_cached(left);
                &left_type_owned
            }
        };
        if crate::checker::utilities_is_optional_symbol::is_type_any(left_type)
            || Arc::ptr_eq(left_type, &self.silent_never_type())
        {
            self.mark_alias_referenced(&parent_symbol, location);
            return;
        }
        let mut prop = prop_symbol.cloned();
        if prop.is_none() && parent_type.is_none() {
            let apparent_type = self.get_apparent_type(left_type);
            prop = self.get_property_of_type(&apparent_type, left.text());
        }
        let skip_mark = prop.is_some_and(|p| {
            crate::checker::mig::m2d::is_const_enum_or_const_enum_only_module(&p)
                || (p.flags.intersects(SymbolFlags::ENUM)
                    && location
                        .parent()
                        .as_ref()
                        .is_some_and(|par| par.kind == SyntaxKind::EnumMember))
        });
        if !skip_mark {
            self.mark_alias_referenced(&parent_symbol, location);
        }
    }
}
