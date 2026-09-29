#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{
    ArrayLiteralExpressionData, BindingElementData, BindingPatternData, CallExpressionData,
    IdentifierData, ImportClauseData, ImportDeclarationData, ImportSpecifierData,
    JsxAttributeData, JsxAttributesData, JsxElementData, JsxExpressionData, JsxFragmentData,
    JsxNamespacedNameData, NamedImportsData, NodeData, NumericLiteralData,
    ObjectLiteralExpressionData, PropertyAccessExpressionData, PropertyAssignmentData,
    QualifiedNameData, SourceFileData, SpreadAssignmentData, SpreadElementData,
    StringLiteralData, VariableDeclarationData, VariableDeclarationListData,
    VariableStatementData, VoidExpressionData,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::{ModifierList, Node, NodeList, SyntaxKind, TokenFlags};
use tsox_core::core::text::TextRange;

use crate::printer::{AutoGenerateOptions, NodeFactory};

const TOKEN_FLAGS_NONE: TokenFlags = 0;

pub trait R36K3NodeFactoryExt {
    fn new_unique_name_node(&self, text: &str, options: AutoGenerateOptions) -> Arc<Node>;
    fn new_string_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node>;
    fn new_numeric_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node>;
    fn new_true_expression(&self) -> Arc<Node>;
    fn new_this_expression(&self) -> Arc<Node>;
    fn new_variable_statement(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        declaration_list: Arc<Node>,
    ) -> Arc<Node>;
    fn new_import_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        import_clause: Arc<Node>,
        module_specifier: Arc<Node>,
        attributes: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_import_clause(
        &self,
        phase_modifier: SyntaxKind,
        name: Option<Arc<Node>>,
        named_bindings: Arc<Node>,
    ) -> Arc<Node>;
    fn new_named_imports(&self, elements: &NodeList) -> Arc<Node>;
    fn new_import_specifier(
        &self,
        is_type_only: bool,
        property_name: Arc<Node>,
        name: Arc<Node>,
    ) -> Arc<Node>;
    fn new_binding_element(
        &self,
        dot_dot_dot_token: Option<Arc<Node>>,
        property_name: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_binding_pattern(&self, kind: SyntaxKind, elements: &NodeList) -> Arc<Node>;
    fn new_array_literal_expression(&self, elements: &NodeList, multi_line: bool) -> Arc<Node>;
    fn new_property_assignment(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Arc<Node>,
    ) -> Arc<Node>;
    fn new_spread_assignment(&self, expression: Arc<Node>) -> Arc<Node>;
    fn new_spread_element(&self, expression: Arc<Node>) -> Arc<Node>;
    fn update_source_file(
        &self,
        file: &SourceFile,
        statements: &NodeList,
        end_of_file_token: Arc<Node>,
    ) -> Arc<Node>;
}

fn identifier_node(text: &str) -> Arc<Node> {
    Arc::new(Node::new(
        SyntaxKind::Identifier,
        NodeData::Identifier(IdentifierData {
            text: text.to_string(),
        }),
    ))
}

fn token_node(kind: SyntaxKind) -> Arc<Node> {
    Arc::new(Node::new(kind, NodeData::Token))
}

impl R36K3NodeFactoryExt for NodeFactory<'_> {
    fn new_unique_name_node(&self, text: &str, options: AutoGenerateOptions) -> Arc<Node> {
        let generated = self.new_unique_name_ex(text, options);
        identifier_node(generated.text())
    }


    fn new_string_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }

    fn new_numeric_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(NumericLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }


    fn new_true_expression(&self) -> Arc<Node> {
        token_node(SyntaxKind::TrueKeyword)
    }


    fn new_this_expression(&self) -> Arc<Node> {
        token_node(SyntaxKind::ThisKeyword)
    }





    fn new_variable_statement(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        declaration_list: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::VariableStatement,
            NodeData::VariableStatement(VariableStatementData {
                modifiers,
                declaration_list,
            }),
        ))
    }

    fn new_import_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        import_clause: Arc<Node>,
        module_specifier: Arc<Node>,
        attributes: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ImportDeclaration,
            NodeData::ImportDeclaration(ImportDeclarationData {
                modifiers,
                import_clause: Some(import_clause),
                module_specifier,
                attributes,
            }),
        ))
    }

    fn new_import_clause(
        &self,
        phase_modifier: SyntaxKind,
        name: Option<Arc<Node>>,
        named_bindings: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ImportClause,
            NodeData::ImportClause(ImportClauseData {
                phase_modifier: Some(phase_modifier),
                name,
                named_bindings: Some(named_bindings),
            }),
        ))
    }

    fn new_named_imports(&self, elements: &NodeList) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NamedImports,
            NodeData::NamedImports(NamedImportsData {
                elements: Arc::new(NodeList {
                    loc: elements.loc,
                    nodes: elements.nodes.clone(),
                }),
            }),
        ))
    }

    fn new_import_specifier(
        &self,
        is_type_only: bool,
        property_name: Arc<Node>,
        name: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ImportSpecifier,
            NodeData::ImportSpecifier(ImportSpecifierData {
                is_type_only,
                property_name: Some(property_name),
                name,
            }),
        ))
    }

    fn new_binding_element(
        &self,
        dot_dot_dot_token: Option<Arc<Node>>,
        property_name: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(BindingElementData {
                dot_dot_dot_token,
                property_name,
                name,
                initializer,
            }),
        ))
    }

    fn new_binding_pattern(&self, kind: SyntaxKind, elements: &NodeList) -> Arc<Node> {
        Arc::new(Node::new(
            kind,
            NodeData::BindingPattern(BindingPatternData {
                elements: Arc::new(NodeList {
                    loc: elements.loc,
                    nodes: elements.nodes.clone(),
                }),
            }),
        ))
    }


    fn new_array_literal_expression(&self, elements: &NodeList, multi_line: bool) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ArrayLiteralExpression,
            NodeData::ArrayLiteralExpression(ArrayLiteralExpressionData {
                elements: Arc::new(NodeList {
                    loc: elements.loc,
                    nodes: elements.nodes.clone(),
                }),
                multi_line,
            }),
        ))
    }

    fn new_property_assignment(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Arc<Node>,
    ) -> Arc<Node> {
        let type_node = type_node.unwrap_or_else(|| {
            Arc::new(Node::with_loc(
                SyntaxKind::Unknown,
                NodeData::Token,
                TextRange::undefined(),
            ))
        });
        Arc::new(Node::new(
            SyntaxKind::PropertyAssignment,
            NodeData::PropertyAssignment(PropertyAssignmentData {
                modifiers,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
        ))
    }

    fn new_spread_assignment(&self, expression: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::SpreadAssignment,
            NodeData::SpreadAssignment(SpreadAssignmentData { expression }),
        ))
    }

    fn new_spread_element(&self, expression: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::SpreadElement,
            NodeData::SpreadElement(SpreadElementData { expression }),
        ))
    }



    fn update_source_file(
        &self,
        file: &SourceFile,
        statements: &NodeList,
        end_of_file_token: Arc<Node>,
    ) -> Arc<Node> {
        match &file.node.data {
            NodeData::SourceFile(d) => {
                let mut node = Node::new(
                    SyntaxKind::SourceFile,
                    NodeData::SourceFile(SourceFileData {
                        statements: Arc::new(NodeList {
                        loc: statements.loc,
                        nodes: statements.nodes.clone(),
                    }),
                        end_of_file_token,
                        global_exports: d.global_exports.clone(),
                    }),
                );
                node.flags = file.node.flags;
                node.loc = file.node.loc;
                Arc::new(node)
            }
            _ => file.node.clone(),
        }
    }
}

pub trait R36K3NodeAccessExt {
    fn as_variable_declaration(&self) -> &VariableDeclarationData;
    fn as_import_specifier(&self) -> &ImportSpecifierData;
    fn as_jsx_element(&self) -> &JsxElementData;
    fn as_jsx_fragment(&self) -> &JsxFragmentData;
    fn as_jsx_expression(&self) -> &JsxExpressionData;
    fn as_jsx_namespaced_name(&self) -> &JsxNamespacedNameData;
    fn as_object_literal_expression(&self) -> &ObjectLiteralExpressionData;
    fn as_qualified_name(&self) -> &QualifiedNameData;
    fn as_string_literal(&self) -> &StringLiteralData;
    fn as_jsx_attribute(&self) -> &JsxAttributeData;
    fn as_source_file_data(&self) -> &SourceFileData;
    fn tag_name(&self) -> Arc<Node>;
    fn attributes_node(&self) -> &Arc<Node>;
    fn properties(&self) -> Vec<Arc<Node>>;
    fn opening_element(&self) -> Arc<Node>;
    fn opening_fragment(&self) -> Arc<Node>;
    fn children(&self) -> &NodeList;
    fn dot_dot_dot_token(&self) -> Option<Arc<Node>>;
    fn statements(&self) -> Vec<Arc<Node>>;
}

macro_rules! as_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl R36K3NodeAccessExt for Node {
    as_data!(as_variable_declaration, VariableDeclaration, VariableDeclarationData);
    as_data!(as_import_specifier, ImportSpecifier, ImportSpecifierData);
    as_data!(as_jsx_element, JsxElement, JsxElementData);
    as_data!(as_jsx_fragment, JsxFragment, JsxFragmentData);
    as_data!(as_jsx_expression, JsxExpression, JsxExpressionData);
    as_data!(as_jsx_namespaced_name, JsxNamespacedName, JsxNamespacedNameData);
    as_data!(
        as_object_literal_expression,
        ObjectLiteralExpression,
        ObjectLiteralExpressionData
    );
    as_data!(as_qualified_name, QualifiedName, QualifiedNameData);
    as_data!(as_string_literal, StringLiteral, StringLiteralData);
    as_data!(as_jsx_attribute, JsxAttribute, JsxAttributeData);
    as_data!(as_source_file_data, SourceFile, SourceFileData);

    fn tag_name(&self) -> Arc<Node> {
        match &self.data {
            NodeData::JsxOpeningElement(d) => d.tag_name.clone(),
            NodeData::JsxSelfClosingElement(d) => d.tag_name.clone(),
            NodeData::JsxClosingElement(d) => d.tag_name.clone(),
            _ => panic!("AsJsxOpeningLikeElement on wrong node kind"),
        }
    }

    fn attributes_node(&self) -> &Arc<Node> {
        match &self.data {
            NodeData::JsxOpeningElement(d) => &d.attributes,
            NodeData::JsxSelfClosingElement(d) => &d.attributes,
            _ => panic!("AsJsxOpeningLikeElement on wrong node kind"),
        }
    }

    fn properties(&self) -> Vec<Arc<Node>> {
        match &self.data {
            NodeData::ObjectLiteralExpression(d) => d.properties.nodes.clone(),
            NodeData::JsxAttributes(d) => d.properties.nodes.clone(),
            _ => panic!("AsObjectLiteralExpressionOrJsxAttributes on wrong node kind"),
        }
    }

    fn opening_element(&self) -> Arc<Node> {
        self.as_jsx_element().opening_element.clone()
    }

    fn opening_fragment(&self) -> Arc<Node> {
        self.as_jsx_fragment().opening_fragment.clone()
    }

    fn children(&self) -> &NodeList {
        match &self.data {
            NodeData::JsxElement(d) => &d.children,
            NodeData::JsxFragment(d) => &d.children,
            _ => panic!("AsJsxContainer on wrong node kind"),
        }
    }

    fn dot_dot_dot_token(&self) -> Option<Arc<Node>> {
        self.as_jsx_expression().dot_dot_dot_token.clone()
    }

    fn statements(&self) -> Vec<Arc<Node>> {
        self.as_source_file_data().statements.nodes.clone()
    }
}

pub trait R36K3SourceFileExt {
    fn as_node(&self) -> Arc<Node>;
    fn file_name(&self) -> &str;
    fn text(&self) -> &str;
    fn end_of_file_token(&self) -> Arc<Node>;
}

impl R36K3SourceFileExt for SourceFile {
    fn as_node(&self) -> Arc<Node> {
        self.node.clone()
    }

    fn file_name(&self) -> &str {
        &self.file_name
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn end_of_file_token(&self) -> Arc<Node> {
        match &self.node.data {
            NodeData::SourceFile(d) => d.end_of_file_token.clone(),
            _ => panic!("node is not a source file"),
        }
    }
}

use crate::mig::m4j::JsxTransformer;

pub struct JsxNodeVisitor<'a> {
    pub tx: &'a mut JsxTransformer,
}

impl<'a> JsxNodeVisitor<'a> {
    pub fn visit_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        let node = node?;
        let visited = self.tx.visit(Some(node));
        match visited {
            Some(v) if v.kind == SyntaxKind::SyntaxList => {
                let children = match &v.data {
                    NodeData::SyntaxList(d) => d.children.clone(),
                    _ => unreachable!(),
                };
                if children.len() != 1 {
                    panic!("Expected only a single node to be written to output");
                }
                let lifted = children.into_iter().next().unwrap();
                if lifted.kind == SyntaxKind::SyntaxList {
                    panic!("The result of visiting and lifting a Node may not be SyntaxList");
                }
                Some(lifted)
            }
            visited => visited,
        }
    }

    pub fn visit_slice(&mut self, nodes: &[Arc<Node>]) -> (Vec<Arc<Node>>, bool) {
        let mut changed = false;
        let mut result = Vec::with_capacity(nodes.len());
        for node in nodes {
            match self.tx.visit(Some(node)) {
                Some(visited) => {
                    changed |= !Arc::ptr_eq(&visited, node);
                    result.push(visited);
                }
                None => changed = true,
            }
        }
        (result, changed)
    }

    pub fn visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::SourceFile {
            let data = node.as_source_file_data();
            let (statements, statements_changed) = self.visit_slice(&data.statements.nodes);
            let eof = data.end_of_file_token.clone();
            let eof_visited = self.tx.visit(Some(&eof));
            let eof_changed = eof_visited.as_ref().map_or(true, |v| !Arc::ptr_eq(v, &eof));
            let eof = eof_visited.unwrap_or(eof);
            if !statements_changed && !eof_changed {
                return Some(node.clone());
            }
            let mut rebuilt = Node::new(
                SyntaxKind::SourceFile,
                NodeData::SourceFile(SourceFileData {
                    statements: Arc::new(NodeList::new(statements)),
                    end_of_file_token: eof,
                    global_exports: data.global_exports.clone(),
                }),
            );
            rebuilt.flags = node.flags;
            rebuilt.loc = node.loc;
            return Some(Arc::new(rebuilt));
        }

        let mut changed = false;
        let mut _visited_children: Vec<Option<Arc<Node>>> = Vec::new();
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            let visited = self.tx.visit(Some(child));
            changed |= visited.as_ref().map_or(true, |v| !Arc::ptr_eq(v, child));
            _visited_children.push(visited);
            true
        });
        if !changed {
            return Some(node.clone());
        }
        panic!(
            "JsxTransformer visit_each_child rebuild for kind {:?} pending Go factory update-function port (progress_notes_r36k3.md)",
            node.kind
        )
    }
}
