#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, Node, NodeFlags, NodeList, SyntaxKind};

use super::m5y_pc::*;
use super::m5x_pc::{add_undefined_if_definitely_required, is_in_const_context};
use tsox_checker::checker::utilities_has_only_expression_initialization::is_const_type_reference;
use tsox_frontend::ast::mig::m3e::{get_function_flags, FunctionFlags};
use tsox_frontend::ast::mig::m3e_4::for_each_return_statement;
use tsox_frontend::ast::mig::m3f_2::has_modifier;
use tsox_frontend::ast::mig::m3f_4::is_const_assertion;
use tsox_frontend::ast::mig::m3g_2::is_primitive_literal_value;
use tsox_frontend::ast::mig::m3g_3::{is_var_const, is_variable_parameter_or_property};
use tsox_frontend::ast::mig::w3::{
    get_all_accessor_declarations_for_declaration, AllAccessorDeclarations,
};

mod pseudo_node_ext {
    use super::*;
    use tsox_frontend::ast::node_data_generated::NodeData;
    use tsox_frontend::ast::node_data_generated::*;
    use tsox_frontend::ast::Symbol;

    pub struct FunctionLikeData<'a> {
        type_parameters: Option<&'a Arc<NodeList>>,
        parameters: Option<&'a Arc<NodeList>>,
        type_node: Option<&'a Arc<Node>>,
        full_signature: Option<&'a Arc<Node>>,
        body: Option<&'a Arc<Node>>,
    }

    impl<'a> FunctionLikeData<'a> {
        pub fn type_parameters(&self) -> Option<&'a Arc<NodeList>> {
            self.type_parameters
        }
        pub fn parameters(&self) -> Option<&'a Arc<NodeList>> {
            self.parameters
        }
        pub fn type_node(&self) -> Option<&'a Arc<Node>> {
            self.type_node
        }
        pub fn full_signature(&self) -> Option<&'a Arc<Node>> {
            self.full_signature
        }
    }

    pub trait PseudoNodeExt {
        fn function_like_data(&self) -> Option<FunctionLikeData<'_>>;
        fn parameters(&self) -> Option<&Arc<NodeList>>;
        fn body(&self) -> Option<&Arc<Node>>;
        fn initializer(&self) -> Option<&Arc<Node>>;
        fn label(&self) -> Option<&Arc<Node>>;
        fn symbol(&self) -> Option<Arc<Symbol>>;
        fn as_object_literal_expression(&self) -> Option<&ObjectLiteralExpressionData>;
        fn as_array_literal_expression(&self) -> Option<&ArrayLiteralExpressionData>;
        fn as_parameter_declaration(&self) -> Option<&ParameterDeclarationData>;
        fn as_get_accessor_declaration(&self) -> Option<&GetAccessorDeclarationData>;
        fn as_set_accessor_declaration(&self) -> Option<&SetAccessorDeclarationData>;
        fn as_method_declaration(&self) -> Option<&MethodDeclarationData>;
        fn as_property_declaration(&self) -> Option<&PropertyDeclarationData>;
        fn as_property_assignment(&self) -> Option<&PropertyAssignmentData>;
        fn as_variable_declaration(&self) -> Option<&VariableDeclarationData>;
        fn as_return_statement(&self) -> Option<&ReturnStatementData>;
        fn as_parenthesized_expression(&self) -> Option<&ParenthesizedExpressionData>;
        fn as_parenthesized_type_node(&self) -> Option<&ParenthesizedTypeNodeData>;
        fn as_intersection_type_node(&self) -> Option<&IntersectionTypeNodeData>;
        fn as_union_type_node(&self) -> Option<&UnionTypeNodeData>;
        fn as_type_assertion_expression(&self) -> Option<&TypeAssertionData>;
        fn as_as_expression(&self) -> Option<&AsExpressionData>;
        fn as_prefix_unary_expression(&self) -> Option<&PrefixUnaryExpressionData>;
        fn as_identifier(&self) -> Option<&IdentifierData>;
    }

    impl PseudoNodeExt for Arc<Node> {
        fn function_like_data(&self) -> Option<FunctionLikeData<'_>> {
            let f = match &self.data {
                NodeData::FunctionDeclaration(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: d.full_signature.as_ref(),
                    body: d.body.as_ref(),
                },
                NodeData::MethodDeclaration(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: d.full_signature.as_ref(),
                    body: d.body.as_ref(),
                },
                NodeData::GetAccessorDeclaration(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: d.full_signature.as_ref(),
                    body: d.body.as_ref(),
                },
                NodeData::SetAccessorDeclaration(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: d.full_signature.as_ref(),
                    body: d.body.as_ref(),
                },
                NodeData::ConstructorDeclaration(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: d.full_signature.as_ref(),
                    body: d.body.as_ref(),
                },
                NodeData::ArrowFunction(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: d.full_signature.as_ref(),
                    body: Some(&d.body),
                },
                NodeData::FunctionExpression(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: d.full_signature.as_ref(),
                    body: Some(&d.body),
                },
                NodeData::MethodSignatureDeclaration(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: None,
                    body: None,
                },
                NodeData::CallSignatureDeclaration(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: None,
                    body: None,
                },
                NodeData::ConstructSignatureDeclaration(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: None,
                    body: None,
                },
                NodeData::FunctionTypeNode(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: None,
                    body: None,
                },
                NodeData::ConstructorTypeNode(d) => FunctionLikeData {
                    type_parameters: d.type_parameters.as_ref(),
                    parameters: Some(&d.parameters),
                    type_node: d.type_node.as_ref(),
                    full_signature: None,
                    body: None,
                },
                _ => return None,
            };
            Some(f)
        }

        fn parameters(&self) -> Option<&Arc<NodeList>> {
            self.function_like_data()?.parameters
        }

        fn body(&self) -> Option<&Arc<Node>> {
            match &self.data {
                NodeData::FunctionDeclaration(d) => d.body.as_ref(),
                NodeData::MethodDeclaration(d) => d.body.as_ref(),
                NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
                NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
                NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
                NodeData::ArrowFunction(d) => Some(&d.body),
                NodeData::FunctionExpression(d) => Some(&d.body),
                _ => None,
            }
        }

        fn initializer(&self) -> Option<&Arc<Node>> {
            tsox_frontend::ast::mig::m3b::initializer(self)
        }

        fn label(&self) -> Option<&Arc<Node>> {
            match &self.data {
                NodeData::LabeledStatement(d) => Some(&d.label),
                NodeData::BreakStatement(d) => d.label.as_ref(),
                NodeData::ContinueStatement(d) => d.label.as_ref(),
                _ => None,
            }
        }

        fn symbol(&self) -> Option<Arc<Symbol>> {
            tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(self)
        }

        fn as_object_literal_expression(&self) -> Option<&ObjectLiteralExpressionData> {
            match &self.data {
                NodeData::ObjectLiteralExpression(d) => Some(d),
                _ => None,
            }
        }

        fn as_array_literal_expression(&self) -> Option<&ArrayLiteralExpressionData> {
            match &self.data {
                NodeData::ArrayLiteralExpression(d) => Some(d),
                _ => None,
            }
        }

        fn as_parameter_declaration(&self) -> Option<&ParameterDeclarationData> {
            match &self.data {
                NodeData::ParameterDeclaration(d) => Some(d),
                _ => None,
            }
        }

        fn as_get_accessor_declaration(&self) -> Option<&GetAccessorDeclarationData> {
            match &self.data {
                NodeData::GetAccessorDeclaration(d) => Some(d),
                _ => None,
            }
        }

        fn as_set_accessor_declaration(&self) -> Option<&SetAccessorDeclarationData> {
            match &self.data {
                NodeData::SetAccessorDeclaration(d) => Some(d),
                _ => None,
            }
        }

        fn as_method_declaration(&self) -> Option<&MethodDeclarationData> {
            match &self.data {
                NodeData::MethodDeclaration(d) => Some(d),
                _ => None,
            }
        }

        fn as_property_declaration(&self) -> Option<&PropertyDeclarationData> {
            match &self.data {
                NodeData::PropertyDeclaration(d) => Some(d),
                _ => None,
            }
        }

        fn as_property_assignment(&self) -> Option<&PropertyAssignmentData> {
            match &self.data {
                NodeData::PropertyAssignment(d) => Some(d),
                _ => None,
            }
        }

        fn as_variable_declaration(&self) -> Option<&VariableDeclarationData> {
            match &self.data {
                NodeData::VariableDeclaration(d) => Some(d),
                _ => None,
            }
        }

        fn as_return_statement(&self) -> Option<&ReturnStatementData> {
            match &self.data {
                NodeData::ReturnStatement(d) => Some(d),
                _ => None,
            }
        }

        fn as_parenthesized_expression(&self) -> Option<&ParenthesizedExpressionData> {
            match &self.data {
                NodeData::ParenthesizedExpression(d) => Some(d),
                _ => None,
            }
        }

        fn as_parenthesized_type_node(&self) -> Option<&ParenthesizedTypeNodeData> {
            match &self.data {
                NodeData::ParenthesizedTypeNode(d) => Some(d),
                _ => None,
            }
        }

        fn as_intersection_type_node(&self) -> Option<&IntersectionTypeNodeData> {
            match &self.data {
                NodeData::IntersectionTypeNode(d) => Some(d),
                _ => None,
            }
        }

        fn as_union_type_node(&self) -> Option<&UnionTypeNodeData> {
            match &self.data {
                NodeData::UnionTypeNode(d) => Some(d),
                _ => None,
            }
        }

        fn as_type_assertion_expression(&self) -> Option<&TypeAssertionData> {
            match &self.data {
                NodeData::TypeAssertion(d) => Some(d),
                _ => None,
            }
        }

        fn as_as_expression(&self) -> Option<&AsExpressionData> {
            match &self.data {
                NodeData::AsExpression(d) => Some(d),
                _ => None,
            }
        }

        fn as_prefix_unary_expression(&self) -> Option<&PrefixUnaryExpressionData> {
            match &self.data {
                NodeData::PrefixUnaryExpression(d) => Some(d),
                _ => None,
            }
        }

        fn as_identifier(&self) -> Option<&IdentifierData> {
            match &self.data {
                NodeData::Identifier(d) => Some(d),
                _ => None,
            }
        }
    }
}

pub use pseudo_node_ext::PseudoNodeExt;

pub struct PseudoChecker {
    pub strict_null_checks: bool,
    pub exact_optional_property_types: bool,
}

impl PseudoChecker {
    pub fn can_get_type_from_object_literal(&self, node: &Arc<Node>) -> Option<Vec<Arc<Node>>> {
        let properties = node
            .as_object_literal_expression()
            .map(|d| d.properties.clone());
        let Some(properties) = properties else {
            return None;
        };
        if properties.nodes.is_empty() {
            return None;
        }
        let mut error_nodes: Vec<Arc<Node>> = Vec::new();
        for e in &properties.nodes {
            if e.flags.intersects(NodeFlags::ThisNodeHasError) {
                error_nodes.push(Arc::clone(e));
                continue;
            }
            if e.kind == SyntaxKind::ShorthandPropertyAssignment || e.kind == SyntaxKind::SpreadAssignment {
                error_nodes.push(Arc::clone(e));
                continue;
            }
            let name = e.name();
            if let Some(name) = name {
                if name.flags.intersects(NodeFlags::ThisNodeHasError) {
                    error_nodes.push(Arc::clone(name));
                    continue;
                }
                if name.kind == SyntaxKind::PrivateIdentifier {
                    error_nodes.push(Arc::clone(e));
                    continue;
                }
                if name.kind == SyntaxKind::ComputedPropertyName {
                    if let Some(expression) = name.expression() {
                        if !is_primitive_literal_value(expression, false) {
                            error_nodes.push(Arc::clone(name));
                        }
                    }
                }
            }
        }
        Some(error_nodes)
    }

    pub fn can_get_type_from_array_literal(&self, node: &Arc<Node>) -> Option<Vec<Arc<Node>>> {
        if !is_in_const_context(node) {
            return Some(vec![Arc::clone(node)]);
        }
        let elements = node
            .as_array_literal_expression()
            .map(|d| d.elements.clone());
        let Some(elements) = elements else {
            return None;
        };
        for e in &elements.nodes {
            if e.kind == SyntaxKind::SpreadElement {
                return Some(vec![Arc::clone(e)]);
            }
        }
        None
    }

    pub fn clone_type_parameters(&self, nodes: Option<&Arc<NodeList>>) -> Option<Vec<Arc<Node>>> {
        let nodes = nodes?;
        if nodes.nodes.is_empty() {
            return None;
        }
        Some(nodes.nodes.clone())
    }

    pub fn clone_parameters(&self, nodes: Option<&Arc<NodeList>>) -> Option<Vec<PseudoParameter>> {
        let nodes = nodes?;
        if nodes.nodes.is_empty() {
            return None;
        }
        let last_required = last_required_param_index(&nodes.nodes);
        let mut result = Vec::with_capacity(nodes.nodes.len());
        for (i, e) in nodes.nodes.iter().enumerate() {
            let p = e.as_parameter_declaration();
            let mut optional = p
                .as_ref()
                .and_then(|d| d.question_token.as_ref())
                .is_some();
            let has_initializer = p
                .as_ref()
                .and_then(|d| d.initializer.as_ref())
                .is_some();
            if !optional && has_initializer {
                optional = i + 1 >= last_required;
            }
            let name = e.name().cloned().unwrap_or_else(|| Arc::clone(e));
            let ty = self.type_from_parameter_worker(e, i, last_required);
            result.push(new_pseudo_parameter(
                p.as_ref()
                    .and_then(|d| d.dot_dot_dot_token.as_ref())
                    .is_some(),
                name,
                optional,
                ty,
            ));
        }
        Some(result)
    }

    pub fn create_return_from_signature(&self, fn_node: &Arc<Node>) -> PseudoType {
        if ast::is_function_like(fn_node) {
            if let Some(d) = fn_node.function_like_data() {
                if let Some(r) = &d.type_node() {
                    return new_pseudo_type_direct(Arc::clone(r));
                }
            }
        }
        if is_value_signature_declaration(fn_node) {
            return self.type_from_single_return_expression(fn_node);
        }
        new_pseudo_type_no_result(Arc::clone(fn_node))
    }

    pub fn get_accessor_member(
        &self,
        accessor: &Arc<Node>,
        name: &Arc<Node>,
    ) -> Option<PseudoObjectElement> {
        let declarations = accessor
            .symbol()
            .map(|s| s.declarations.clone())
            .unwrap_or_default();
        let all_accessors =
            get_all_accessor_declarations_for_declaration(accessor, &declarations);

        let get_annotated = all_accessors
            .get_accessor
            .as_ref()
            .and_then(|g| g.type_node())
            .is_some();
        let set_annotated = all_accessors
            .set_accessor
            .as_ref()
            .and_then(|s| s.parameters())
            .and_then(|p| p.nodes.first())
            .and_then(|p| p.as_parameter_declaration())
            .and_then(|d| d.type_node.clone())
            .is_some();
        if get_annotated && set_annotated {
            if accessor.kind == SyntaxKind::GetAccessor {
                return Some(new_pseudo_get_accessor(
                    Arc::clone(accessor),
                    Arc::clone(name),
                    false,
                    self.type_from_accessor(accessor),
                ));
            }
            if accessor.kind == SyntaxKind::SetAccessor {
                if let Some(parameters) = self
                    .clone_parameters(accessor.as_set_accessor_declaration().map(|d| &d.parameters))
                {
                    if let Some(p) = parameters.into_iter().next() {
                        return Some(new_pseudo_set_accessor(
                            Arc::clone(accessor),
                            Arc::clone(name),
                            false,
                            p,
                        ));
                    }
                }
            }
        }

        if let Some(first) = all_accessors.first_accessor.as_ref() {
            if Arc::ptr_eq(accessor, first) {
                let accessor_type = self.type_from_accessor(accessor);
                let readonly =
                    accessor.kind == SyntaxKind::GetAccessor && all_accessors.second_accessor.is_none();
                return Some(new_pseudo_property_assignment(
                    readonly,
                    Arc::clone(name),
                    false,
                    accessor_type,
                ));
            }
        }
        None
    }

    pub fn get_type_annotation_from_all_accessor_declarations(
        &self,
        node: &Arc<Node>,
        accessors: &AllAccessorDeclarations,
    ) -> Option<Arc<Node>> {
        let mut accessor_type = self.get_type_annotation_from_accessor(node);
        if accessor_type.is_none() {
            if let Some(first) = accessors.first_accessor.as_ref() {
                if !Arc::ptr_eq(node, first) {
                    accessor_type = self.get_type_annotation_from_accessor(first);
                }
            }
        }
        if accessor_type.is_none() {
            if let Some(second) = accessors.second_accessor.as_ref() {
                if !Arc::ptr_eq(node, second) {
                    accessor_type = self.get_type_annotation_from_accessor(second);
                }
            }
        }
        accessor_type
    }

    pub fn get_type_annotation_from_accessor(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::GetAccessor {
            return node
                .as_get_accessor_declaration()
                .and_then(|d| d.type_node.clone());
        }
        if node.kind == SyntaxKind::SetAccessor {
            let parameters = node
                .as_set_accessor_declaration()
                .map(|d| d.parameters.clone());
            let first = parameters
                .as_ref()
                .and_then(|p| p.nodes.first().cloned());
            let Some(p) = first else {
                return None;
            };
            if p.kind != SyntaxKind::Parameter {
                return None;
            }
            return p.as_parameter_declaration().and_then(|d| d.type_node.clone());
        }
        None
    }

    pub fn type_from_accessor(&self, accessor: &Arc<Node>) -> PseudoType {
        let declarations = accessor
            .symbol()
            .map(|s| s.declarations.clone())
            .unwrap_or_default();
        let accessor_declarations =
            get_all_accessor_declarations_for_declaration(accessor, &declarations);
        let accessor_type =
            self.get_type_annotation_from_all_accessor_declarations(accessor, &accessor_declarations);
        if let Some(accessor_type) = &accessor_type {
            if !ast::is_type_predicate_node(accessor_type) {
                return new_pseudo_type_direct(Arc::clone(accessor_type));
            }
        }
        if let Some(get_accessor) = accessor_declarations.get_accessor.as_ref() {
            let res = self.create_return_from_signature(get_accessor);
            if res.kind == PseudoTypeKind::Inferred {
                let inferred = res.as_pseudo_type_inferred().unwrap();
                if inferred.error_nodes.is_empty() {
                    let mut error_nodes = vec![Arc::clone(get_accessor)];
                    if let Some(set_accessor) = accessor_declarations.set_accessor.as_ref() {
                        error_nodes.push(Arc::clone(set_accessor));
                    }
                    return new_pseudo_type_inferred_with_errors(
                        Arc::clone(&inferred.expression),
                        inferred.is_signature_return,
                        error_nodes,
                    );
                }
            }
            return res;
        }
        new_pseudo_type_no_result(Arc::clone(accessor))
    }

    pub fn type_from_array_literal(&self, node: &Arc<Node>) -> PseudoType {
        if let Some(error_nodes) = self.can_get_type_from_array_literal(node) {
            if !error_nodes.is_empty() {
                return new_pseudo_type_inferred_with_errors(Arc::clone(node), false, error_nodes);
            }
        }
        if is_in_const_context(node) && is_contextually_typed(node) {
            return new_pseudo_type_inferred(Arc::clone(node), false);
        }
        let elements = node
            .as_array_literal_expression()
            .map(|d| d.elements.clone());
        let mut results = Vec::new();
        if let Some(elements) = elements {
            for e in &elements.nodes {
                results.push(self.type_from_expression(e));
            }
        }
        new_pseudo_type_tuple(results)
    }

    pub fn type_from_expando_property(&self, node: &Arc<Node>) -> PseudoType {
        if let Some(declared_type) = node.type_node() {
            return new_pseudo_type_direct(Arc::clone(declared_type));
        }
        new_pseudo_type_no_result(Arc::clone(node))
    }

    pub fn type_from_expression(&self, node: &Arc<Node>) -> PseudoType {
        match node.kind {
            SyntaxKind::OmittedExpression => pseudo_type_undefined(),
            SyntaxKind::ParenthesizedExpression => {
                let inner = node
                    .as_parenthesized_expression()
                    .map(|d| d.expression.clone())
                    .unwrap_or_else(|| Arc::clone(node));
                self.type_from_expression(&inner)
            }
            SyntaxKind::Identifier => {
                if let Some(d) = node.as_identifier() {
                    if d.text == "undefined" {
                        return pseudo_type_undefined();
                    }
                }
                new_pseudo_type_inferred(Arc::clone(node), false)
            }
            SyntaxKind::NullKeyword => pseudo_type_null(),
            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression => {
                self.type_from_function_like_expression(node)
            }
            SyntaxKind::TypeAssertionExpression => {
                let d = node.as_type_assertion_expression();
                let expression = d.map(|d| d.expression.clone()).unwrap_or_else(|| Arc::clone(node));
                let type_node = d.map(|d| d.type_node.clone()).unwrap_or_else(|| Arc::clone(node));
                self.type_from_type_assertion(&expression, &type_node)
            }
            SyntaxKind::AsExpression => {
                let d = node.as_as_expression();
                let expression = d.map(|d| d.expression.clone()).unwrap_or_else(|| Arc::clone(node));
                let type_node = d.map(|d| d.type_node.clone()).unwrap_or_else(|| Arc::clone(node));
                self.type_from_type_assertion(&expression, &type_node)
            }
            SyntaxKind::PrefixUnaryExpression => {
                if is_primitive_literal_value(node, true) {
                    return self.type_from_primitive_literal_prefix(node);
                }
                new_pseudo_type_inferred(Arc::clone(node), false)
            }
            SyntaxKind::ArrayLiteralExpression => self.type_from_array_literal(node),
            SyntaxKind::ObjectLiteralExpression => self.type_from_object_literal(node),
            SyntaxKind::ClassExpression => {
                new_pseudo_type_inferred_with_errors(Arc::clone(node), false, vec![Arc::clone(node)])
            }
            SyntaxKind::TemplateExpression => {
                if is_in_const_context(node) {
                    return new_pseudo_type_inferred(Arc::clone(node), false);
                }
                new_pseudo_type_maybe_const_location(
                    Arc::clone(node),
                    Some(new_pseudo_type_inferred(Arc::clone(node), false)),
                    Some(pseudo_type_string()),
                )
            }
            SyntaxKind::NumericLiteral => new_pseudo_type_maybe_const_location(
                Arc::clone(node),
                Some(new_pseudo_type_numeric_literal(Arc::clone(node))),
                Some(pseudo_type_number()),
            ),
            SyntaxKind::NoSubstitutionTemplateLiteral | SyntaxKind::StringLiteral => {
                new_pseudo_type_maybe_const_location(
                    Arc::clone(node),
                    Some(new_pseudo_type_string_literal(Arc::clone(node))),
                    Some(pseudo_type_string()),
                )
            }
            SyntaxKind::BigIntLiteral => new_pseudo_type_maybe_const_location(
                Arc::clone(node),
                Some(new_pseudo_type_big_int_literal(Arc::clone(node))),
                Some(pseudo_type_big_int()),
            ),
            SyntaxKind::TrueKeyword => new_pseudo_type_maybe_const_location(
                Arc::clone(node),
                Some(pseudo_type_true()),
                Some(pseudo_type_boolean()),
            ),
            SyntaxKind::FalseKeyword => new_pseudo_type_maybe_const_location(
                Arc::clone(node),
                Some(pseudo_type_false()),
                Some(pseudo_type_boolean()),
            ),
            _ => new_pseudo_type_inferred(Arc::clone(node), false),
        }
    }

    pub fn type_from_function_like_expression(&self, node: &Arc<Node>) -> PseudoType {
        if let Some(d) = node.function_like_data() {
            if let Some(full_signature) = d.full_signature() {
                return new_pseudo_type_direct(Arc::clone(full_signature));
            }
        }
        let return_type = self.create_return_from_signature(node);
        let type_parameters = self.clone_type_parameters(
            node.function_like_data().and_then(|d| d.type_parameters()),
        );
        let parameters = self.clone_parameters(
            node.function_like_data().and_then(|d| d.parameters()),
        );
        new_pseudo_type_single_call_signature(
            Arc::clone(node),
            parameters.unwrap_or_default(),
            type_parameters.unwrap_or_default(),
            Some(return_type),
        )
    }

    pub fn type_from_object_literal(&self, node: &Arc<Node>) -> PseudoType {
        if let Some(error_nodes) = self.can_get_type_from_object_literal(node) {
            if !error_nodes.is_empty() {
                return new_pseudo_type_inferred_with_errors(Arc::clone(node), false, error_nodes);
            }
        }
        let properties = node
            .as_object_literal_expression()
            .map(|d| d.properties.clone());
        let Some(properties) = properties else {
            return new_pseudo_type_object_literal(Vec::new());
        };
        if properties.nodes.is_empty() {
            return new_pseudo_type_object_literal(Vec::new());
        }
        let mut results: Vec<PseudoObjectElement> = Vec::with_capacity(properties.nodes.len());
        for e in &properties.nodes {
            match e.kind {
                SyntaxKind::MethodDeclaration => {
                    let optional = e
                        .as_method_declaration()
                        .and_then(|d| d.postfix_token.as_ref())
                        .map(|t| t.kind == SyntaxKind::QuestionToken)
                        .unwrap_or(false);
                    let full_signature = e
                        .function_like_data()
                        .and_then(|d| d.full_signature().cloned());
                    if let Some(full_signature) = full_signature {
                        results.push(new_pseudo_property_assignment(
                            false,
                            e.name().cloned().unwrap_or_else(|| Arc::clone(e)),
                            optional,
                            new_pseudo_type_direct(full_signature),
                        ));
                    } else {
                        let type_parameters = self.clone_type_parameters(
                            e.as_method_declaration().and_then(|d| d.type_parameters.as_ref()),
                        );
                        let parameters = self.clone_parameters(
                            e.function_like_data().and_then(|d| d.parameters()),
                        );
                        results.push(new_pseudo_object_method(
                            Arc::clone(e),
                            e.name().cloned().unwrap_or_else(|| Arc::clone(e)),
                            optional,
                            type_parameters.unwrap_or_default(),
                            parameters.unwrap_or_default(),
                            Some(self.create_return_from_signature(e)),
                        ));
                    }
                }
                SyntaxKind::PropertyAssignment => {
                    let optional = e
                        .as_property_assignment()
                        .and_then(|d| d.postfix_token.as_ref())
                        .map(|t| t.kind == SyntaxKind::QuestionToken)
                        .unwrap_or(false);
                    let initializer = e
                        .as_property_assignment()
                        .map(|d| d.initializer.clone())
                        .unwrap_or_else(|| Arc::clone(e));
                    results.push(new_pseudo_property_assignment(
                        false,
                        e.name().cloned().unwrap_or_else(|| Arc::clone(e)),
                        optional,
                        self.type_from_expression(&initializer),
                    ));
                }
                SyntaxKind::SetAccessor | SyntaxKind::GetAccessor => {
                    if let Some(name) = e.name() {
                        if let Some(member) = self.get_accessor_member(e, name) {
                            results.push(member);
                        }
                    }
                }
                _ => {}
            }
        }
        new_pseudo_type_object_literal(results)
    }

    pub fn type_from_parameter(&self, node: &Arc<Node>) -> PseudoType {
        let Some(parent) = node.parent() else {
            return new_pseudo_type_no_result(Arc::clone(node));
        };
        if parent.kind == SyntaxKind::SetAccessor {
            return self.type_from_accessor(&parent);
        }
        let data = node.as_parameter_declaration();
        let initializer = data.and_then(|d| d.initializer.clone());
        if initializer.is_none() {
            if let Some(type_node) = data.and_then(|d| d.type_node.clone()) {
                return new_pseudo_type_direct(type_node);
            }
            return new_pseudo_type_no_result(Arc::clone(node));
        }
        let parameters = parent.parameters();
        let self_idx = parameters
            .as_ref()
            .and_then(|p| p.nodes.iter().position(|n| Arc::ptr_eq(n, node)))
            .unwrap_or(0);
        let last_required = last_required_param_index(
            parameters.as_ref().map(|p| p.nodes.as_slice()).unwrap_or(&[]),
        );
        self.type_from_parameter_worker(node, self_idx, last_required)
    }

    pub fn type_from_parameter_worker(
        &self,
        node: &Arc<Node>,
        self_idx: usize,
        last_required: usize,
    ) -> PseudoType {
        let Some(parent) = node.parent() else {
            return new_pseudo_type_no_result(Arc::clone(node));
        };
        if parent.kind == SyntaxKind::SetAccessor {
            return self.type_from_accessor(&parent);
        }
        let has_required_after = self_idx < last_required.saturating_sub(1);
        let data = node.as_parameter_declaration();
        let declared_type = data.and_then(|d| d.type_node.clone());
        let initializer = data.and_then(|d| d.initializer.clone());
        if let Some(declared_type) = declared_type {
            let result = new_pseudo_type_direct(declared_type);
            if self.strict_null_checks && initializer.is_some() && has_required_after {
                return add_undefined_if_definitely_required(result);
            }
            return result;
        }
        let name_is_identifier = data
            .map(|d| d.name.kind == SyntaxKind::Identifier)
            .unwrap_or(false);
        if let Some(initializer) = initializer {
            if name_is_identifier && !is_contextually_typed(node) {
                let mut expr = self.type_from_expression(&initializer);
                if expr.kind == PseudoTypeKind::Inferred {
                    let inferred = expr.as_pseudo_type_inferred().unwrap();
                    if inferred.error_nodes.is_empty() {
                        expr = new_pseudo_type_inferred_with_errors(
                            Arc::clone(&inferred.expression),
                            false,
                            vec![Arc::clone(node)],
                        );
                    }
                }
                if !self.strict_null_checks {
                    return expr;
                }
                if !has_required_after {
                    return expr;
                }
                return add_undefined_if_definitely_required(expr);
            }
        }
        new_pseudo_type_no_result(Arc::clone(node))
    }

    pub fn type_from_primitive_literal_prefix(&self, node: &Arc<Node>) -> PseudoType {
        let data = node.as_prefix_unary_expression();
        let operator = data.map(|d| d.operator);
        let operand = data
            .map(|d| d.operand.clone())
            .unwrap_or_else(|| Arc::clone(node));
        let mut expr: Arc<Node> = Arc::clone(node);
        if operator == Some(SyntaxKind::PlusToken) {
            expr = Arc::clone(&operand);
        }
        if operand.kind == SyntaxKind::BigIntLiteral {
            return new_pseudo_type_maybe_const_location(
                Arc::clone(node),
                Some(new_pseudo_type_big_int_literal(expr)),
                Some(pseudo_type_big_int()),
            );
        }
        if operand.kind == SyntaxKind::NumericLiteral {
            return new_pseudo_type_maybe_const_location(
                Arc::clone(node),
                Some(new_pseudo_type_numeric_literal(expr)),
                Some(pseudo_type_number()),
            );
        }
        new_pseudo_type_no_result(Arc::clone(node))
    }

    pub fn type_from_property(&self, node: &Arc<Node>) -> PseudoType {
        if let Some(t) = node.type_node() {
            return new_pseudo_type_direct(Arc::clone(t));
        }
        if ast::is_property_declaration(node) {
            let init = node.initializer().cloned();
            if let Some(init) = init {
                if !is_contextually_typed(node) {
                    let readonly_template = has_modifier(node, ast::ModifierFlags::Readonly)
                        && ast::is_template_expression(init.as_ref());
                    if readonly_template {
                        return new_pseudo_type_no_result(Arc::clone(node));
                    }
                    let expr = self.type_from_expression(&init);
                    if expr.kind != PseudoTypeKind::Inferred {
                        if expr.kind != PseudoTypeKind::Direct {
                            let postfix_is_question = node
                                .as_property_declaration()
                                .and_then(|d| d.postfix_token.as_ref())
                                .map(|t| t.kind == SyntaxKind::QuestionToken)
                                .unwrap_or(false);
                            if postfix_is_question {
                                return add_undefined_if_definitely_required(expr);
                            }
                        }
                        return expr;
                    }
                    let has_errors = expr
                        .as_pseudo_type_inferred()
                        .map(|i| !i.error_nodes.is_empty())
                        .unwrap_or(false);
                    if has_errors {
                        if expr.kind != PseudoTypeKind::Direct {
                            let postfix_is_question = node
                                .as_property_declaration()
                                .and_then(|d| d.postfix_token.as_ref())
                                .map(|t| t.kind == SyntaxKind::QuestionToken)
                                .unwrap_or(false);
                            if postfix_is_question {
                                return add_undefined_if_definitely_required(expr);
                            }
                        }
                        return expr;
                    }
                }
            }
        }
        new_pseudo_type_no_result(Arc::clone(node))
    }

    pub fn type_from_property_assignment(&self, node: &Arc<Node>) -> PseudoType {
        if let Some(annotation) = node.type_node() {
            return new_pseudo_type_direct(Arc::clone(annotation));
        }
        if node.kind == SyntaxKind::PropertyAssignment {
            let init = node.initializer().cloned();
            if let Some(init) = init {
                let expr = self.type_from_expression(&init);
                let usable = expr.kind != PseudoTypeKind::Inferred
                    || expr
                        .as_pseudo_type_inferred()
                        .map(|i| !i.error_nodes.is_empty())
                        .unwrap_or(false);
                if usable {
                    return expr;
                }
            }
        }
        new_pseudo_type_no_result(Arc::clone(node))
    }

    pub fn type_from_single_return_expression(&self, fn_node: &Arc<Node>) -> PseudoType {
        let mut candidate_expr: Option<Arc<Node>> = None;
        let body = fn_node.body();
        if let Some(body) = &body {
            if !ast::node_is_missing(Some(body)) {
                let flags = get_function_flags(Some(fn_node));
                if flags.0 & FunctionFlags::ASYNC_GENERATOR.0 != 0 {
                    return new_pseudo_type_inferred(Arc::clone(fn_node), true);
                }
                if body.kind == SyntaxKind::Block {
                    for_each_return_statement(body, &mut |stmt: &Arc<Node>| {
                        if stmt.parent().map(|p| !Arc::ptr_eq(&p, body)).unwrap_or(true) {
                            candidate_expr = None;
                            return true;
                        }
                        if candidate_expr.is_none() {
                            candidate_expr = stmt
                                .as_return_statement()
                                .and_then(|d| d.expression.clone());
                            if candidate_expr.is_none() {
                                return true;
                            }
                            return false;
                        }
                        candidate_expr = None;
                        true
                    });
                } else {
                    candidate_expr = Some(Arc::clone(body));
                }
            }
        }
        if let Some(candidate) = candidate_expr {
            if is_contextually_typed(&candidate) {
                let t = if candidate.kind == SyntaxKind::TypeAssertionExpression {
                    candidate.as_type_assertion_expression().map(|d| d.type_node.clone())
                } else if candidate.kind == SyntaxKind::AsExpression {
                    candidate.as_as_expression().map(|d| d.type_node.clone())
                } else {
                    None
                };
                if let Some(t) = t {
                    if !is_const_type_reference(t.as_ref()) {
                        return new_pseudo_type_direct(t);
                    }
                }
            } else {
                return self.type_from_expression(&candidate);
            }
        }
        new_pseudo_type_inferred(Arc::clone(fn_node), true)
    }

    pub fn type_from_type_assertion(
        &self,
        expression: &Arc<Node>,
        type_node: &Arc<Node>,
    ) -> PseudoType {
        if is_const_type_reference(type_node.as_ref()) {
            return self.type_from_expression(expression);
        }
        new_pseudo_type_direct(Arc::clone(type_node))
    }

    pub fn type_from_variable(&self, declaration: &Arc<Node>) -> PseudoType {
        let data = declaration.as_variable_declaration();
        if let Some(t) = data.and_then(|d| d.type_node.clone()) {
            return new_pseudo_type_direct(t);
        }
        let init = data.and_then(|d| d.initializer.clone());
        let has_single_declaration = declaration
            .symbol()
            .map(|s| {
                s.declarations.len() == 1
                    || tsox_core::core::core::count_where(&s.declarations, |d: &Arc<Node>| {
                        d.kind == SyntaxKind::VariableDeclaration
                    }) == 1
            })
            .unwrap_or(false);
        if let Some(init) = init {
            if has_single_declaration && !is_contextually_typed(declaration) {
                if is_var_const(declaration) && ast::is_template_expression(init.as_ref()) {
                    return new_pseudo_type_no_result(Arc::clone(declaration));
                }
                let expr = self.type_from_expression(&init);
                let usable = expr.kind != PseudoTypeKind::Inferred
                    || expr
                        .as_pseudo_type_inferred()
                        .map(|i| !i.error_nodes.is_empty())
                        .unwrap_or(false);
                if usable {
                    return expr;
                }
            }
        }
        new_pseudo_type_no_result(Arc::clone(declaration))
    }
}

pub fn is_const_context_propagating_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::SpreadElement
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::TemplateSpan
            | SyntaxKind::PrefixUnaryExpression
    )
}

pub fn is_contextually_typed(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    ast::find_ancestor(&parent, |n: &Node| {
        if ast::is_call_expression(n) {
            return true;
        }
        if ast::is_satisfies_expression(n) {
            return true;
        }
        if (is_variable_parameter_or_property(n) || ast::is_assertion_expression(n))
            && n.type_node().is_some()
            && !is_const_assertion(n)
        {
            return true;
        }
        ast::is_jsx_element(n) || ast::is_jsx_expression(n)
    })
    .is_some()
}

pub fn is_optional_initialized_or_rest_parameter(node: &Arc<Node>) -> bool {
    node.as_parameter_declaration()
        .map(|d| d.dot_dot_dot_token.is_some() || d.initializer.is_some() || d.question_token.is_some())
        .unwrap_or(false)
}

pub fn is_undefined_pseudo_type(t: &PseudoType) -> bool {
    if t.kind == PseudoTypeKind::Undefined {
        return true;
    }
    if t.kind == PseudoTypeKind::MaybeConstLocation {
        if let Some(mc) = t.as_pseudo_type_maybe_const_location() {
            if let Some(const_type) = &mc.const_type {
                return is_undefined_pseudo_type(const_type);
            }
        }
    }
    false
}

pub fn is_value_signature_declaration(node: &Node) -> bool {
    ast::is_function_expression(node)
        || ast::is_arrow_function(node)
        || ast::is_method_declaration(node)
        || ast::is_accessor(node)
        || ast::is_function_declaration(node)
        || ast::is_constructor_declaration(node)
}

pub fn last_required_param_index(params: &[Arc<Node>]) -> usize {
    for i in (0..params.len()).rev() {
        if !is_optional_initialized_or_rest_parameter(&params[i]) {
            return i + 1;
        }
    }
    0
}

pub fn type_node_could_refer_to_undefined(node: &Arc<Node>) -> bool {
    let mut node = Arc::clone(node);
    while node.kind == SyntaxKind::ParenthesizedType {
        let next = node
            .as_parenthesized_type_node()
            .map(|d| d.type_node.clone())
            .unwrap_or_else(|| Arc::clone(&node));
        node = next;
    }
    match node.kind {
        SyntaxKind::TypeReference
        | SyntaxKind::IndexedAccessType
        | SyntaxKind::TypeQuery
        | SyntaxKind::OptionalType
        | SyntaxKind::RestType
        | SyntaxKind::ImportType => true,
        SyntaxKind::IntersectionType => node
            .as_intersection_type_node()
            .map(|d| {
                d.types
                    .nodes
                    .iter()
                    .any(|t| type_node_could_refer_to_undefined(t))
            })
            .unwrap_or(false),
        SyntaxKind::UnionType => node
            .as_union_type_node()
            .map(|d| {
                d.types
                    .nodes
                    .iter()
                    .any(|t| type_node_could_refer_to_undefined(t))
            })
            .unwrap_or(false),
        SyntaxKind::ConditionalType => true,
        SyntaxKind::TypeOperator => true,
        SyntaxKind::TypePredicate => true,
        SyntaxKind::UndefinedKeyword => true,
        _ => false,
    }
}
