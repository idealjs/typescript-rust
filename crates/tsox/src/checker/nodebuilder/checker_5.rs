#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn signature_to_parameter_nodes(&mut self, sig: &Signature) -> Vec<Arc<Node>> {
        sig.parameters
            .iter()
            .map(|param| {
                let name = self.identifier(&param.name);
                let param_type = self.get_type_of_symbol(param);
                let type_node = self.type_to_type_node(&param_type);
                let optional = param.flags.contains(SymbolFlags::Optional);
                self.parameter_node(name, optional, type_node)
            })
            .collect()
    }

    pub(crate) fn call_signature_to_node(&mut self, sig: &Signature) -> Arc<Node> {
        let params = self.signature_to_parameter_nodes(sig);
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let ret_node = self.type_to_type_node(&ret_type);
        self.function_type_node(params, ret_node)
    }

    pub fn symbol_to_type_node(
        &mut self,
        symbol: &Arc<Symbol>,
        mask: SymbolFlags,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {
        let _ = mask;

        let name = self.identifier(&symbol.name);

        let type_args = type_arguments.or_else(|| {
            let t = self.get_type_of_symbol(symbol);
            if let Some(obj) = t.as_object() {
                if !obj.type_arguments.is_empty() {
                    let arg_nodes: Vec<Arc<Node>> = obj
                        .type_arguments
                        .iter()
                        .map(|ty| self.type_to_type_node(ty))
                        .collect();
                    return Some(Arc::new(NodeList::new(arg_nodes)));
                }
            }
            None
        });
        self.type_reference_node(name, type_args)
    }

    pub(crate) fn keyword_node(&self, kind: SyntaxKind) -> Arc<Node> {
        Arc::new(Node::new(kind, NodeData::Token))
    }

    pub(crate) fn identifier(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    pub(crate) fn string_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags: 0,
            }),
        ))
    }

    pub(crate) fn numeric_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(NumericLiteralData {
                text: text.to_string(),
                token_flags: 0,
            }),
        ))
    }

    pub(crate) fn bigint_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::BigIntLiteral,
            NodeData::BigIntLiteral(BigIntLiteralData {
                text: format!("{}n", text),
                token_flags: 0,
            }),
        ))
    }

    pub(crate) fn literal_type_node(&self, literal: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(LiteralTypeNodeData { literal }),
        ))
    }

    pub(crate) fn type_reference_node(
        &self,
        type_name: Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeReference,
            NodeData::TypeReferenceNode(TypeReferenceNodeData {
                type_name,
                type_arguments,
            }),
        ))
    }

    pub(crate) fn array_type_node(&self, element_type: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ArrayType,
            NodeData::ArrayTypeNode(ArrayTypeNodeData { element_type }),
        ))
    }

    pub(crate) fn tuple_type_node(&self, elements: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TupleType,
            NodeData::TupleTypeNode(TupleTypeNodeData {
                elements: Arc::new(NodeList::new(elements)),
            }),
        ))
    }

    pub(crate) fn union_type_node(&self, types: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::UnionType,
            NodeData::UnionTypeNode(UnionTypeNodeData {
                types: Arc::new(NodeList::new(types)),
            }),
        ))
    }

    pub(crate) fn intersection_type_node(&self, types: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::IntersectionType,
            NodeData::IntersectionTypeNode(IntersectionTypeNodeData {
                types: Arc::new(NodeList::new(types)),
            }),
        ))
    }

    pub(crate) fn parenthesized_type_node(&self, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ParenthesizedType,
            NodeData::ParenthesizedTypeNode(ParenthesizedTypeNodeData { type_node }),
        ))
    }

    pub(crate) fn function_type_node(&self, params: Vec<Arc<Node>>, ret: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(FunctionTypeNodeData {
                type_parameters: None,
                parameters: Arc::new(NodeList::new(params)),
                type_node: Some(ret),
            }),
        ))
    }

    pub(crate) fn type_literal_node(&self, members: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeLiteral,
            NodeData::TypeLiteralNode(TypeLiteralNodeData {
                members: Arc::new(NodeList::new(members)),
            }),
        ))
    }

    pub(crate) fn property_signature_node(
        &self,
        name: Arc<Node>,
        optional: bool,
        type_node: Arc<Node>,
    ) -> Arc<Node> {
        let postfix_token = if optional {
            Some(self.keyword_node(SyntaxKind::QuestionToken))
        } else {
            None
        };

        let initializer = Arc::new(Node::new(
            SyntaxKind::MissingDeclaration,
            NodeData::MissingDeclaration(MissingDeclarationData { modifiers: None }),
        ));
        Arc::new(Node::new(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                modifiers: None,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
        ))
    }

    pub(crate) fn parameter_node(&self, name: Arc<Node>, optional: bool, type_node: Arc<Node>) -> Arc<Node> {
        let question_token = if optional {
            Some(self.keyword_node(SyntaxKind::QuestionToken))
        } else {
            None
        };
        Arc::new(Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name,
                question_token,
                type_node: Some(type_node),
                initializer: None,
            }),
        ))
    }

    pub(crate) fn rest_type_node(&self, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::RestType,
            NodeData::RestTypeNode(RestTypeNodeData { type_node }),
        ))
    }

    pub(crate) fn type_operator_node(&self, operator: SyntaxKind, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeOperator,
            NodeData::TypeOperatorNode(TypeOperatorNodeData {
                operator,
                type_node,
            }),
        ))
    }

    pub fn symbol_to_string(&mut self, symbol: &Arc<Symbol>) -> String {
        self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::AllowAnyNodeKind,
            SymbolFlags::all(),
        )
    }

    pub fn symbol_to_string_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        flags: SymbolFormatFlags,
        _meaning: SymbolFlags,
    ) -> String {
        let name = symbol.name.clone();

        if flags.contains(SymbolFormatFlags::WriteTypeParametersOrArguments) {
            if let Some(tps) = self.collect_type_parameter_names(symbol) {
                if !tps.is_empty() {
                    return format!("{}<{}>", name, tps.join(", "));
                }
            }
        }
        name
    }

    pub(crate) fn collect_type_parameter_names(&self, symbol: &Arc<Symbol>) -> Option<Vec<String>> {
        for decl in &symbol.declarations {
            let tps = match &decl.data {
                NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::TypeAliasDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
                _ => continue,
            };
            if let Some(tps) = tps {
                return Some(
                    tps.iter()
                        .map(|tp| match &tp.data {
                            NodeData::TypeParameterDeclaration(tpd) => tpd.name.text().to_string(),
                            _ => String::new(),
                        })
                        .collect(),
                );
            }
        }
        None
    }

}
