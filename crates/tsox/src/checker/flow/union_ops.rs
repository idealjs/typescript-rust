use std::sync::Arc;

use crate::ast::{Node, NodeData, NodeFlags, Symbol, SymbolFlags, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::*;

use super::PropertyPresence;
use super::is_assignment_operator;

use super::FlowRef;

impl Checker {
    pub(crate) fn constituent_is_definitely_falsy(&self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
            return true;
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {

            return matches!(t.literal_value(), Some(crate::checker::types::LiteralValue::Boolean(false)));
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            return t.intrinsic_name().is_some_and(|n| n == "\"\"" || n.is_empty());
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            return t.intrinsic_name().is_some_and(|n| n == "0");
        }
        false
    }

    pub(crate) fn flow_constituents_public(&self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        self.constituent_types(t)
    }

    pub(crate) fn flow_constituent_definitely_falsy(&self, t: &Arc<Type>) -> bool {
        self.constituent_is_definitely_falsy(t)
    }

    fn extract_definitely_falsy_constituents(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let falsy: Vec<Arc<Type>> = self
            .constituent_types(t)
            .into_iter()
            .filter(|c| self.constituent_is_definitely_falsy(c))
            .collect();
        self.rebuild_union_or_never(t, falsy)
    }

    fn remove_definitely_falsy_constituents(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let kept: Vec<Arc<Type>> = self
            .constituent_types(t)
            .into_iter()
            .filter(|c| !self.constituent_is_definitely_falsy(c))
            .collect();
        if kept.is_empty() {
            return Arc::clone(t);
        }
        self.rebuild_union_or_never(t, kept)
    }

    pub(crate) fn flow_union_of(&self, types: &[Arc<Type>]) -> Arc<Type> {
        let mut all: Vec<Arc<Type>> = Vec::new();
        for t in types {
            for c in self.constituent_types(t) {
                if !all.iter().any(|s| Arc::ptr_eq(s, &c)) {
                    all.push(c);
                }
            }
        }
        if all.is_empty() {
            return self.never_type();
        }
        if all.len() == 1 {
            return all.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: all,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn remove_type_from_union(&self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !self.types_overlap(t, value_type))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }

        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub fn remove_flags_from_union(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !t.flags.intersects(flags))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn filter_type_by_flags(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| t.flags.intersects(flags))
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn filter_type_by_object(&self, type_: &Arc<Type>, is_loose: bool) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let mut matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {

                t.flags.contains(TypeFlags::Object)
                    || t.flags.contains(TypeFlags::Null)
                    || (is_loose && t.flags.contains(TypeFlags::Undefined))
            })
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching.drain(..).collect(),
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn filter_type_by_callable(&self, type_: &Arc<Type>, keep_callable: bool) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let is_callable = !self
                    .get_signatures_of_type(t, SignatureKind::Call)
                    .is_empty();
                if keep_callable {
                    is_callable
                } else {
                    !is_callable
                }
            })
            .collect();
        if filtered.is_empty() {
            return self.never_type();
        }
        if filtered.len() == 1 {
            return filtered.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: filtered,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn remove_object_from_union(&self, type_: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Object) && !t.flags.contains(TypeFlags::Null))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn remove_falsy_from_union(&self, type_: &Arc<Type>, falsy_flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                if t.flags.intersects(falsy_flags) {

                    if t.flags.contains(TypeFlags::BooleanLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            return matches!(lit.value, LiteralValue::Boolean(true));
                        }
                    }

                    if t.flags.contains(TypeFlags::StringLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            if let LiteralValue::String(s) = &lit.value {
                                return !s.is_empty();
                            }
                        }
                        return false;
                    }

                    if t.flags.contains(TypeFlags::NumberLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            if let LiteralValue::Number(n) = &lit.value {
                                return n.0 != 0.0;
                            }
                        }
                        return false;
                    }
                    return false;
                }
                true
            })
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn filter_to_falsy(&self, type_: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let falsy_flags =
            TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Void | TypeFlags::BooleanLiteral;
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                if t.flags.intersects(falsy_flags) {

                    if t.flags.contains(TypeFlags::BooleanLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            return matches!(lit.value, LiteralValue::Boolean(false));
                        }
                    }
                    return true;
                }

                if t.flags.contains(TypeFlags::StringLiteral) {
                    if let TypeData::Literal(lit) = &t.data {
                        if let LiteralValue::String(s) = &lit.value {
                            return s.is_empty();
                        }
                    }
                }

                if t.flags.contains(TypeFlags::NumberLiteral) {
                    if let TypeData::Literal(lit) = &t.data {
                        if let LiteralValue::Number(n) = &lit.value {
                            return n.0 == 0.0;
                        }
                    }
                }
                false
            })
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn intersect_or_narrow(&mut self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {

        if self.is_type_assignable_to(value_type, type_) {
            return Arc::clone(value_type);
        }

        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let matching: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| self.is_type_assignable_to(value_type, t))
                .collect();
            if matching.len() == 1 {
                return matching.into_iter().next().expect("exactly one");
            }
            if matching.is_empty() {
                return Arc::clone(value_type);
            }
            return self.get_union_type(matching);
        }
        Arc::clone(value_type)
    }

    pub(crate) fn types_overlap(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {

        if a.flags.contains(TypeFlags::Union)
            || b.flags.contains(TypeFlags::Union)
            || a.flags.contains(TypeFlags::Intersection)
            || b.flags.contains(TypeFlags::Intersection)
        {
            let a_types = self.constituent_types(a);
            let b_types = self.constituent_types(b);
            for at in &a_types {
                for bt in &b_types {
                    if self.literals_overlap(at, bt) {
                        return true;
                    }
                }
            }
            return false;
        }
        self.literals_overlap(a, b)
    }

    fn literals_overlap(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {

        let a_is_literal = a.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral,
        );
        let b_is_literal = b.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral,
        );
        if a_is_literal && b_is_literal {

            return match (&a.data, &b.data) {
                (TypeData::Literal(a_lit), TypeData::Literal(b_lit)) => a_lit.value == b_lit.value,
                _ => false,
            };
        }
        if a_is_literal {

            return a.flags.intersects(b.flags);
        }
        if b_is_literal {
            return a.flags.intersects(b.flags);
        }

        a.flags.intersects(b.flags)
    }

    pub(crate) fn is_symbol_identifier(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {

        if matches!(
            node.kind,
            SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
        ) {
            return self
                .program
                .symbol_map()
                .symbol_of(node)
                .is_some_and(|s| Arc::ptr_eq(s, symbol));
        }
        if node.kind != SyntaxKind::Identifier {
            return false;
        }

        let symbol_map = self.program.symbol_map();
        if let Some(sym) = symbol_map.symbol_of(node) {
            let eq = Arc::ptr_eq(sym, symbol);
            return eq;
        }

        let node_name = match &node.data {
            NodeData::Identifier(data) => &data.text,
            _ => return false,
        };
        let eq = node_name == &symbol.name;
        eq
    }

    pub(crate) fn expr_matches_target(&self, node: &Arc<Node>, target: &FlowRef) -> bool {
        match target {
            FlowRef::Symbol(symbol) => self.is_symbol_identifier(node, symbol),
            FlowRef::Node(reference) => self.is_matching_reference(reference, node),
        }
    }

    pub(crate) fn is_matching_reference(&self, source: &Arc<Node>, target: &Arc<Node>) -> bool {
        match &target.data {

            NodeData::ParenthesizedExpression(p) => {
                return self.is_matching_reference(source, &p.expression);
            }
            NodeData::NonNullExpression(n) => {
                return self.is_matching_reference(source, &n.expression);
            }
            _ => {}
        }
        match target.kind {
            SyntaxKind::BinaryExpression => {
                if let NodeData::BinaryExpression(bin) = &target.data {
                    if is_assignment_operator(bin.operator_token.kind)
                        && self.is_matching_reference(source, &bin.left)
                    {
                        return true;
                    }
                    if bin.operator_token.kind == SyntaxKind::CommaToken
                        && self.is_matching_reference(source, &bin.right)
                    {
                        return true;
                    }
                }
                return false;
            }
            _ => {}
        }
        match source.kind {
            SyntaxKind::BinaryExpression => {

                if let NodeData::BinaryExpression(bin) = &source.data {
                    if bin.operator_token.kind == SyntaxKind::CommaToken {
                        return self.is_matching_reference(&bin.right, target);
                    }
                    if is_assignment_operator(bin.operator_token.kind) {
                        return self.is_matching_reference(&bin.left, target);
                    }
                }
                return false;
            }
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier => {
                if target.kind == SyntaxKind::Identifier {
                    return match (
                        self.resolve_identifier(source),
                        self.resolve_identifier(target),
                    ) {
                        (Some(s), Some(t)) => Arc::ptr_eq(&s, &t),
                        _ => false,
                    };
                }

                if matches!(
                    target.kind,
                    SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
                ) {
                    let Some(source_sym) = self.resolve_identifier(source) else {
                        return false;
                    };
                    let Some(target_sym) =
                        self.program.symbol_map().symbol_of(target).cloned()
                    else {
                        return false;
                    };

                    let source_unwrapped = source_sym
                        .export_symbol
                        .clone()
                        .unwrap_or_else(|| Arc::clone(&source_sym));
                    let target_unwrapped = target_sym
                        .export_symbol
                        .clone()
                        .unwrap_or(target_sym);
                    return Arc::ptr_eq(&source_unwrapped, &target_unwrapped);
                }
                false
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => target.kind == source.kind,
            SyntaxKind::NonNullExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::SatisfiesExpression => {
                if let Some(inner) = source.expression() {
                    self.is_matching_reference(&inner, target)
                } else {
                    false
                }
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(source_prop_name) = self.get_accessed_property_name(source) {
                    if matches!(
                        target.kind,
                        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                    ) {
                        if let Some(target_prop_name) = self.get_accessed_property_name(target) {
                            if target_prop_name == source_prop_name {
                                let source_receiver = source.expression();
                                let target_receiver = target.expression();
                                if let (Some(s), Some(t)) = (source_receiver, target_receiver) {
                                    return self.is_matching_reference(&s, &t);
                                }
                            }
                        }
                    }
                }

                if source.kind == SyntaxKind::ElementAccessExpression
                    && target.kind == SyntaxKind::ElementAccessExpression
                {
                    let (NodeData::ElementAccessExpression(source_ea),
                         NodeData::ElementAccessExpression(target_ea)) =
                        (&source.data, &target.data)
                    else {
                        return false;
                    };
                    if source_ea.argument_expression.kind == SyntaxKind::Identifier
                        && target_ea.argument_expression.kind == SyntaxKind::Identifier
                    {
                        let matching_args = match (
                            self.resolve_identifier(&source_ea.argument_expression),
                            self.resolve_identifier(&target_ea.argument_expression),
                        ) {
                            (Some(s), Some(t)) if Arc::ptr_eq(&s, &t) => {
                                self.symbol_is_const_variable(&s)
                                    || (self.is_parameter_or_mutable_local(&s)
                                        && !self.symbol_is_assigned(&s))
                            }
                            _ => false,
                        };
                        if matching_args {
                            let (Some(s), Some(t)) = (
                                source.expression(),
                                target.expression(),
                            ) else {
                                return false;
                            };
                            return self.is_matching_reference(&s, &t);
                        }
                    }
                }
                false
            }
            SyntaxKind::QualifiedName => {
                if let NodeData::QualifiedName(qualified) = &source.data {
                    if matches!(
                        target.kind,
                        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                    ) {
                        if let Some(target_prop_name) = self.get_accessed_property_name(target) {
                            if qualified.right.text() == target_prop_name {
                                if let Some(t) = target.expression() {
                                    return self.is_matching_reference(&qualified.left, &t);
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub(crate) fn contains_matching_reference(&self, source: &Arc<Node>, target: &Arc<Node>) -> bool {
        let mut source = Arc::clone(source);
        while matches!(
            source.kind,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
        ) {
            let Some(inner) = source.expression() else {
                break;
            };
            if self.is_matching_reference(inner, target) {
                return true;
            }
            source = Arc::clone(inner);
        }
        false
    }

    fn get_accessed_property_name(&self, access: &Arc<Node>) -> Option<String> {
        match &access.data {
            NodeData::PropertyAccessExpression(pa) => Some(pa.name.text().to_string()),
            NodeData::ElementAccessExpression(ea) => {
                match &ea.argument_expression.data {
                    NodeData::StringLiteral(s) => Some(s.text.clone()),
                    NodeData::NumericLiteral(n) => Some(n.text.clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn is_parameter_or_mutable_local(&self, symbol: &Arc<Symbol>) -> bool {
        symbol
            .flags
            .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
    }

    fn symbol_is_assigned(&self, symbol: &Arc<Symbol>) -> bool {
        let Some(decl) = symbol.value_declaration.as_ref() else {
            return true;
        };
        let Some(container) = Self::enclosing_function_or_source_file(decl) else {
            return true;
        };
        let mut assigned = false;
        Self::scan_assignment_targets(&container, &symbol.name, &mut assigned);
        assigned
    }

    pub(crate) fn enclosing_function_or_source_file(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(node);
        loop {
            if matches!(
                current.kind,
                SyntaxKind::SourceFile
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::Constructor
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                return Some(current);
            }
            current = Arc::clone(current.parent.as_ref()?);
        }
    }

    fn scan_assignment_targets(node: &Arc<Node>, name: &str, assigned: &mut bool) {
        if *assigned {
            return;
        }
        match &node.data {
            NodeData::BinaryExpression(bin) => {
                if is_assignment_operator(bin.operator_token.kind)
                    && bin.left.kind == SyntaxKind::Identifier
                    && bin.left.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && unary.operand.kind == SyntaxKind::Identifier
                    && unary.operand.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            NodeData::PostfixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && unary.operand.kind == SyntaxKind::Identifier
                    && unary.operand.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            _ => {}
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            Self::scan_assignment_targets(child, name, assigned);
            *assigned
        });
    }

    pub(crate) fn const_alias_initializer(&self, expr: &Arc<Node>) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }

        let sym = self.resolve_identifier(expr)?;
        if !self.symbol_is_const_variable(&sym) {
            return None;
        }
        let decl = sym.value_declaration.as_ref()?;
        if decl.kind != SyntaxKind::VariableDeclaration {
            return None;
        }
        let NodeData::VariableDeclaration(var_data) = &decl.data else {
            return None;
        };

        if var_data.type_node.is_some() {
            return None;
        }
        let init = var_data.initializer.as_ref()?;
        Some(Self::skip_parentheses(init))
    }

    pub(crate) fn symbol_is_const_variable(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(parent) = &decl.parent {
                if parent.kind == SyntaxKind::VariableDeclarationList
                    && parent.flags.contains(NodeFlags::Const)
                {
                    return true;
                }
            }
        }
        false
    }

    fn skip_parentheses(node: &Arc<Node>) -> Arc<Node> {
        let mut current = Arc::clone(node);
        loop {
            if let NodeData::ParenthesizedExpression(p) = &current.data {
                current = Arc::clone(&p.expression);
                continue;
            }
            return current;
        }
    }

    pub(crate) fn evolve_array_at_mutation(
        &mut self,
        node: &Arc<Node>,
        pre_type: &Arc<Type>,
        target: &FlowRef,
    ) -> Option<Arc<Type>> {

        let receiver = self.get_array_mutation_receiver(node)?;
        if !self.expr_matches_target(&receiver, target) {
            return None;
        }

        let evolving = if pre_type.object_flags.contains(ObjectFlags::EvolvingArray) {
            Arc::clone(pre_type)
        } else if self.is_auto_array_type(pre_type) {
            self.get_evolving_array_type(self.never_type())
        } else {

            return Some(Arc::clone(pre_type));
        };

        let args = self.get_call_arguments(node);
        let mut arg_types: Vec<Arc<Type>> = Vec::with_capacity(args.len());
        for arg in &args {
            let t = self.get_type_of_node(arg);
            arg_types.push(self.get_widened_type_of_literal(&t));
        }

        let mut evolved = evolving;
        match &node.data {
            NodeData::BinaryExpression(bin)
                if is_assignment_operator(bin.operator_token.kind) =>
            {
                if let NodeData::ElementAccessExpression(ea) = &bin.left.data {
                    let index_type = self.get_type_of_node(&ea.argument_expression);
                    if index_type
                        .flags
                        .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
                    {
                        let t = self.get_type_of_node(&bin.right);
                        let widened = self.get_widened_type_of_literal(&t);
                        evolved = self.add_evolving_array_element_type(&evolved, widened);
                    }
                }
            }
            _ => {
                for arg_type in arg_types {
                    evolved = self.add_evolving_array_element_type(&evolved, arg_type);
                }
            }
        }
        Some(evolved)
    }

    fn get_array_mutation_receiver(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            NodeData::CallExpression(call) => {

                if let NodeData::PropertyAccessExpression(prop) = &call.expression.data {
                    return Some(Arc::clone(&prop.expression));
                }
                None
            }
            NodeData::BinaryExpression(bin) => {

                if let NodeData::ElementAccessExpression(ea) = &bin.left.data {
                    return Some(Arc::clone(&ea.expression));
                }
                None
            }
            _ => None,
        }
    }

    fn get_call_arguments(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        match &node.data {
            NodeData::CallExpression(call) => call.arguments.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }

    fn binding_element_in_var_pattern(element: &Arc<Node>) -> bool {
        let pattern = element.parent.as_ref();
        let Some(decl) = pattern.and_then(|p| p.parent.as_ref()) else {
            return false;
        };
        if decl.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let Some(list) = decl.parent.as_ref() else {
            return false;
        };
        if list.kind != SyntaxKind::VariableDeclarationList {
            return false;
        }

        !(list
            .flags
            .intersects(crate::ast::node_flags::NodeFlags::Let)
            || list
                .flags
                .intersects(crate::ast::node_flags::NodeFlags::Const))
    }

    pub(crate) fn assignment_flow_type(
        &mut self,
        expr: &Arc<Node>,
        target: &FlowRef,
        declared: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let evolving = declared.object_flags.contains(ObjectFlags::EvolvingArray)
            || self.is_auto_array_type(declared);
        match &expr.data {

            NodeData::BinaryExpression(bin) => {
                if !is_assignment_operator(bin.operator_token.kind) {
                    return None;
                }
                if !self.expr_matches_target(&bin.left, target) {
                    return None;
                }

                if bin.operator_token.kind == SyntaxKind::EqualsToken {

                    let assigned = if matches!(
                        &bin.right.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        self.auto_array_type()
                    } else {
                        self.get_type_of_node(&bin.right)
                    };
                    return Some(self.reduced_assignment_type(declared, &assigned, evolving));
                }

                let assigned = self.get_type_of_node(&bin.right);
                let possibly_nullish = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null));
                let possibly_falsy = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| self.constituent_is_definitely_falsy(c));
                let possibly_truthy = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| !self.constituent_is_definitely_falsy(c));
                match bin.operator_token.kind {
                    SyntaxKind::QuestionQuestionEqualsToken if possibly_nullish => {

                        let non_null = self.get_non_nullable_type_of(declared);
                        Some(self.flow_union_of(&[non_null, assigned]))
                    }
                    SyntaxKind::BarBarEqualsToken if possibly_falsy => {

                        let truthy = self.remove_definitely_falsy_constituents(declared);
                        Some(self.flow_union_of(&[truthy, assigned]))
                    }
                    SyntaxKind::AmpersandAmpersandEqualsToken if possibly_truthy => {

                        let falsy = self.extract_definitely_falsy_constituents(declared);
                        Some(self.flow_union_of(&[falsy, assigned]))
                    }
                    SyntaxKind::QuestionQuestionEqualsToken
                    | SyntaxKind::BarBarEqualsToken
                    | SyntaxKind::AmpersandAmpersandEqualsToken => {
                        Some(Arc::clone(declared))
                    }
                    _ => None,
                }
            }

            NodeData::PostfixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
                {
                    Some(self.number_type())
                } else {
                    None
                }
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
                {
                    Some(self.number_type())
                } else {
                    None
                }
            }

            NodeData::VariableDeclaration(_) | NodeData::BindingElement(_) => {
                let FlowRef::Symbol(symbol) = target else {
                    return None;
                };
                let element_symbol = self.program.symbol_map().symbol_of(expr).cloned();
                let matched = match &element_symbol {
                    Some(s) => {
                        Arc::ptr_eq(s, symbol)
                            || symbol
                                .export_symbol
                                .as_ref()
                                .is_some_and(|e| Arc::ptr_eq(s, e))
                    }

                    None => match &expr.data {
                        NodeData::BindingElement(be) => be
                            .name
                            .as_ref()
                            .and_then(|name| self.resolve_identifier(name))
                            .is_some_and(|s| Arc::ptr_eq(&s, symbol)),
                        _ => false,
                    },
                } || (

                    element_symbol.as_ref().is_some_and(|s| {
                        s.name == symbol.name
                            && symbol
                                .flags
                                .contains(crate::ast::SymbolFlags::FunctionScopedVariable)
                            && Self::binding_element_in_var_pattern(expr)
                    })
                );
                if !matched {
                    return None;
                }
                let assigned = self.initial_type_of_declaration(expr)?;
                Some(self.reduced_assignment_type(declared, &assigned, evolving))
            }

            NodeData::Identifier(_) if self.expr_matches_target(expr, target) => {
                Some(Arc::clone(declared))
            }
            _ => None,
        }
    }

    fn reduced_assignment_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
        evolving: bool,
    ) -> Arc<Type> {
        if evolving {
            return Arc::clone(assigned);
        }

        if declared.flags.contains(TypeFlags::Null)
            && (self.is_auto_array_type(assigned) || assigned.object_flags.contains(ObjectFlags::EvolvingArray))
        {
            return Arc::clone(assigned);
        }
        if !declared.is_union() {
            return Arc::clone(declared);
        }
        self.get_assignment_reduced_type(declared, assigned)
    }

    fn get_assignment_reduced_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
    ) -> Arc<Type> {
        if Arc::ptr_eq(declared, assigned) {
            return Arc::clone(declared);
        }
        if assigned.flags.contains(TypeFlags::Never) {
            return Arc::clone(assigned);
        }
        let constituents = self.constituent_types(declared);
        let kept: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| self.type_maybe_assignable_to(assigned, t))
            .collect();
        let reduced = self.rebuild_union_or_never(declared, kept);
        if self.is_type_assignable_to(assigned, &reduced) {
            reduced
        } else {
            Arc::clone(declared)
        }
    }

    fn type_maybe_assignable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if !source.is_union() {
            return self.is_type_assignable_to(source, target);
        }
        let constituents = self.constituent_types(source);
        if constituents.iter().any(|t| Arc::ptr_eq(t, target)) {
            return true;
        }
        constituents
            .iter()
            .any(|t| self.is_type_assignable_to(t, target))
    }

    pub(crate) fn initial_type_of_declaration(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> {
        match &expr.data {
            NodeData::VariableDeclaration(vd) => {
                if let Some(init) = &vd.initializer {

                    if matches!(
                        &init.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        return Some(self.auto_array_type());
                    }
                    if matches!(
                        init.kind,
                        crate::ast::SyntaxKind::NullKeyword | crate::ast::SyntaxKind::UndefinedKeyword
                    ) {
                        return Some(self.auto_type());
                    }
                    return Some(self.get_type_of_node(init));
                }
                let for_stmt = Self::for_in_or_of_statement_of(expr)?;
                let NodeData::ForInOrOfStatement(data) = &for_stmt.data else {
                    return None;
                };
                match for_stmt.kind {
                    SyntaxKind::ForInStatement => Some(self.string_type()),
                    SyntaxKind::ForOfStatement => {
                        let rhs = self.get_type_of_node(&data.expression);
                        Some(self.iterated_element_type(&rhs))
                    }
                    _ => None,
                }
            }
            NodeData::BindingElement(be) => {
                let pattern = Arc::clone(expr.parent.as_ref()?);
                let pattern_parent = Arc::clone(pattern.parent.as_ref()?);
                let parent_type = self.initial_type_of_declaration(&pattern_parent);
                let mut t = match (&parent_type, pattern.kind) {
                    (Some(parent_type), SyntaxKind::ObjectBindingPattern) => {
                        match Self::binding_element_property_name(expr) {
                            Some(name) => self.get_property_type_of_type(parent_type, &name),
                            None => None,
                        }
                    }
                    (
                        Some(parent_type),
                        SyntaxKind::ArrayBindingPattern,
                    ) if be.dot_dot_dot_token.is_none() => {
                        match Self::binding_element_index(&pattern, expr) {
                            Some(index) => {
                                self.destructured_array_element_type(parent_type, index)
                            }
                            None => None,
                        }
                    }
                    _ => None,
                };
                if let Some(default_expr) = &be.initializer {
                    let default_type = self.get_type_of_node(default_expr);

                    t = match t {
                        Some(t) => {
                            let non_undefined = self.remove_flags_from_union(&t, TypeFlags::Undefined);
                            Some(self.get_union_type(vec![non_undefined, default_type]))
                        }
                        None => Some(default_type),
                    };
                }
                t
            }
            _ => None,
        }
    }

    fn binding_element_property_name(element: &Arc<Node>) -> Option<String> {
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        if let Some(pn) = &be.property_name {
            return Some(pn.text().to_string());
        }
        be.name.as_ref().map(|n| n.text().to_string())
    }

    fn binding_element_index(pattern: &Arc<Node>, element: &Arc<Node>) -> Option<usize> {
        let NodeData::BindingPattern(data) = &pattern.data else {
            return None;
        };
        data.elements
            .nodes
            .iter()
            .position(|e| Arc::ptr_eq(e, element))
    }

    fn destructured_array_element_type(
        &mut self,
        parent_type: &Arc<Type>,
        index: usize,
    ) -> Option<Arc<Type>> {
        if self.is_tuple_type(parent_type) {
            return self.get_tuple_element_type(parent_type, index);
        }
        if self.is_array_type(parent_type) {
            return Some(self.get_array_element_type(parent_type));
        }
        Some(self.get_any_type())
    }

    fn iterated_element_type(&mut self, rhs: &Arc<Type>) -> Arc<Type> {

        if rhs.is_union() {
            let parts: Vec<Arc<Type>> = self
                .constituent_types(rhs)
                .into_iter()
                .map(|c| self.iterated_element_type(&c))
                .filter(|t| !t.flags.contains(TypeFlags::Never))
                .collect();
            if parts.is_empty() {
                return self.get_any_type();
            }
            if parts.len() == 1 {
                return parts.into_iter().next().expect("exactly one");
            }
            return self.get_union_type(parts);
        }
        if self.is_array_type(rhs) {
            return self.get_array_element_type(rhs);
        }
        if rhs.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral) {
            return self.string_type();
        }
        self.get_any_type()
    }

    fn for_in_or_of_statement_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let list = decl.parent.as_ref()?;
        if list.kind != SyntaxKind::VariableDeclarationList {
            return None;
        }
        let stmt = list.parent.as_ref()?;
        if matches!(
            stmt.kind,
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
        ) {
            Some(Arc::clone(stmt))
        } else {
            None
        }
    }

    pub(crate) fn for_in_expression_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let stmt = Self::for_in_or_of_statement_of(decl)?;
        if stmt.kind != SyntaxKind::ForInStatement {
            return None;
        }
        match &stmt.data {
            NodeData::ForInOrOfStatement(d) => Some(Arc::clone(&d.expression)),
            _ => None,
        }
    }

    pub(crate) fn get_property_of_type(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Symbol>> {
        if let Some(sym) = self.get_property_of_type_cached(t, name) {
            return Some(sym);
        }
        if let Some(interface_sym) = self
            .unresolved_interface_symbol_of(t)
            && let Some(member) = self
                .resolve_interface_type_ex(&interface_sym, None)
                .as_structured()
                .and_then(|s| s.members.get(name))
        {
            return Some(Arc::clone(member));
        }
        None
    }

    fn unresolved_interface_symbol_of(&self, t: &Arc<Type>) -> Option<Arc<Symbol>> {
        if !t.flags.contains(crate::checker::types::TypeFlags::Object) {
            return None;
        }
        let sym = t.symbol.as_ref()?;
        let has_interface_decl = sym
            .declarations
            .iter()
            .any(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)));
        if !has_interface_decl {
            return None;
        }
        if self.type_alias_links.get(sym).map(|l| l.declared_type.is_some()) == Some(true) {
            return None;
        }
        if let Some(structured) = t.as_structured()
            && !structured.members.entries.is_empty()
        {
            return None;
        }
        Some(Arc::clone(sym))
    }

    pub(crate) fn get_property_of_type_cached(&self, t: &Arc<Type>, name: &str) -> Option<Arc<Symbol>> {

        if let TypeData::Mapped(m) = &t.data
            && m.type_parameter.is_some()
        {
            let sym = Symbol::new(SymbolFlags::Property, name.to_string());
            return Some(Arc::new(sym));
        }

        if let Some(structured) = t.as_structured() {
            if let Some(sym) = structured.members.get(name) {
                return Some(Arc::clone(sym));
            }
        }

        let is_array_like = self.is_array_type(t)
            || matches!(&t.data, TypeData::EvolvingArray(_));
        if is_array_like
            && let Some(array_sym) = self.globals.get("Array")
        {

            if let Some(declared) = self
                .type_alias_links
                .get(array_sym)
                .and_then(|l| l.declared_type.clone())
                && let Some(structured) = declared.as_structured()
                && let Some(member) = structured.members.get(name)
            {
                return Some(Arc::clone(member));
            }

            if let Some(member) = array_sym.members.get(name) {
                return Some(Arc::clone(member));
            }
        }

        if t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::Anonymous)
            && let Some(structured) = t.as_structured()
            && structured.call_signature_count > 0
            && !self.is_array_type(t)
            && !matches!(&t.data, TypeData::EvolvingArray(_))
        {
            if let Some(function_sym) = self.globals.get("Function") {
                if let Some(member) = function_sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }

        if let Some(interface_name) = self.primitive_interface_name(t) {
            if let Some(sym) = self.globals.get(interface_name) {
                if let Some(member) = sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }
        None
    }

    fn primitive_interface_name(&self, t: &Arc<Type>) -> Option<&'static str> {
        if t.flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            Some("String")
        } else if t
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            Some("Number")
        } else if t
            .flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            Some("Boolean")
        } else if t
            .flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            Some("BigInt")
        } else {
            None
        }
    }

    pub(crate) fn get_property_type_of_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> {
        let sym = self.get_property_of_type(t, name)?;
        Some(self.get_type_of_symbol(&sym))
    }

    pub(crate) fn type_has_property(&self, t: &Arc<Type>, name: &str) -> PropertyPresence {
        if let Some(structured) = t.as_structured() {
            if let Some(sym) = structured.members.get(name) {
                if sym.flags.contains(SymbolFlags::Optional) {
                    return PropertyPresence::Maybe;
                }
                return PropertyPresence::Definitely;
            }
            if !structured.index_infos.is_empty() {
                return PropertyPresence::Maybe;
            }
            return PropertyPresence::DefinitelyNot;
        }

        if t.flags.contains(TypeFlags::Object) {
            return PropertyPresence::Maybe;
        }

        PropertyPresence::DefinitelyNot
    }

    pub(crate) fn get_instance_type_of_constructor(&mut self, ctor_type: &Arc<Type>) -> Option<Arc<Type>> {

        if let Some(prop_sym) = self.get_property_of_type(ctor_type, "prototype") {
            let prop_type = self.get_type_of_symbol(&prop_sym);
            if !prop_type.flags.contains(TypeFlags::Any) {
                return Some(prop_type);
            }
        }

        let construct_sigs = self.get_signatures_of_type(ctor_type, SignatureKind::Construct);
        if !construct_sigs.is_empty() {
            let mut return_types: Vec<Arc<Type>> = Vec::new();
            for sig in &construct_sigs {
                if let Some(rt) = self.get_return_type_of_signature(sig) {
                    if !return_types.iter().any(|t| Arc::ptr_eq(t, &rt)) {
                        return_types.push(rt);
                    }
                }
            }
            if !return_types.is_empty() {
                return Some(self.get_union_type(return_types));
            }
        }
        None
    }

    pub(crate) fn get_accessed_property_name_from_node(node: &Arc<Node>) -> Option<String> {
        match &node.data {
            NodeData::StringLiteral(s) => Some(s.text.clone()),
            NodeData::NumericLiteral(n) => Some(n.text.clone()),
            NodeData::Identifier(id) => Some(id.text.clone()),
            NodeData::PropertyAccessExpression(pa) => Some(pa.name.text().to_string()),
            NodeData::ElementAccessExpression(ea) => {
                Self::get_accessed_property_name_from_node(&ea.argument_expression)
            }

            NodeData::BindingElement(be) => be
                .property_name
                .as_ref()
                .map(|pn| pn.text().to_string())
                .or_else(|| be.name.as_ref().map(|n| n.text().to_string())),
            _ => None,
        }
    }

    pub(crate) fn discriminant_alias_access(
        &self,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(expr)?;
        if !self.symbol_is_const_variable(&sym) {
            return None;
        }
        let decl = Arc::clone(sym.value_declaration.as_ref()?);

        if let Some(init) = Self::candidate_variable_declaration_initializer(&decl) {
            if matches!(
                init.kind,
                SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
            ) {
                if let Some(recv) = init.expression() {
                    if self.is_symbol_identifier(recv, symbol) {
                        return Some(init);
                    }
                }
            }
        }

        if decl.kind == SyntaxKind::BindingElement {
            let NodeData::BindingElement(be) = &decl.data else {
                return None;
            };
            if be.dot_dot_dot_token.is_none() && be.initializer.is_none() {
                let pattern = decl.parent.as_ref()?;
                let var_decl = Arc::clone(pattern.parent.as_ref()?);
                if let Some(init) = Self::candidate_variable_declaration_initializer(&var_decl) {
                    let init_matches = match init.kind {
                        SyntaxKind::Identifier => self.is_symbol_identifier(&init, symbol),
                        SyntaxKind::PropertyAccessExpression
                        | SyntaxKind::ElementAccessExpression => init
                            .expression()
                            .is_some_and(|recv| self.is_symbol_identifier(recv, symbol)),
                        _ => false,
                    };
                    if init_matches {
                        return Some(decl);
                    }
                }
            }
        }
        None
    }

    fn candidate_variable_declaration_initializer(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let NodeData::VariableDeclaration(data) = &decl.data else {
            return None;
        };
        if data.type_node.is_some() {
            return None;
        }
        data.initializer.as_ref().map(Self::skip_parentheses)
    }

    pub(crate) fn is_property_access_on_reference(&self, node: &Arc<Node>, reference: &Arc<Node>) -> bool {
        let mut r = reference;
        loop {
            match &r.data {
                NodeData::ParenthesizedExpression(p) => r = &p.expression,
                NodeData::NonNullExpression(n) => r = &n.expression,
                _ => break,
            }
        }
        match &node.data {
            NodeData::PropertyAccessExpression(pa) => {
                self.is_matching_reference(r, &pa.expression)
            }
            NodeData::ElementAccessExpression(ea) => {
                self.is_matching_reference(r, &ea.expression)
            }
            _ => false,
        }
    }

    pub(crate) fn is_property_access_on_symbol(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        match &node.data {
            NodeData::PropertyAccessExpression(pa) => {

                pa.question_dot_token.is_none() && self.is_symbol_identifier(&pa.expression, symbol)
            }
            NodeData::ElementAccessExpression(ea) => {
                ea.question_dot_token.is_none() && self.is_symbol_identifier(&ea.expression, symbol)
            }
            _ => false,
        }
    }

    pub(crate) fn narrow_to_subtype(&mut self, type_: &Arc<Type>, candidate: &Arc<Type>) -> Arc<Type> {

        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(candidate);
        }
        if type_.is_union() {

            let constituents = self.constituent_types(type_);
            let mapped: Vec<Arc<Type>> = constituents
                .into_iter()
                .map(|t| {
                    if self.is_type_assignable_to(&t, candidate) {
                        t
                    } else if self.is_type_assignable_to(candidate, &t) {
                        Arc::clone(candidate)
                    } else {
                        self.never_type()
                    }
                })
                .collect();
            return self.rebuild_union_or_never(type_, mapped);
        }

        if self.is_type_assignable_to(candidate, type_) {
            Arc::clone(candidate)
        } else {
            Arc::clone(type_)
        }
    }

    pub(crate) fn remove_subtype_from_union(&mut self, type_: &Arc<Type>, candidate: &Arc<Type>) -> Arc<Type> {
        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !self.is_type_assignable_to(t, candidate))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
        if self.is_type_assignable_to(type_, candidate) {
            self.never_type()
        } else {
            Arc::clone(type_)
        }
    }

    pub(crate) fn rebuild_union_or_never(
        &mut self,
        original: &Arc<Type>,
        constituents: Vec<Arc<Type>>,
    ) -> Arc<Type> {
        if constituents.is_empty() {
            return self.never_type();
        }
        if constituents.len() == 1 {
            return constituents.into_iter().next().expect("exactly one");
        }

        if let TypeData::Union(u) = &original.data {
            if u.union_or_intersection.types.len() == constituents.len()
                && u.union_or_intersection
                    .types
                    .iter()
                    .zip(constituents.iter())
                    .all(|(a, b)| Arc::ptr_eq(a, b))
            {
                return Arc::clone(original);
            }
        }
        self.get_union_type(constituents)
    }
}
