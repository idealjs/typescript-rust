#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    pub(crate) fn get_declaration_name(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::VariableDeclaration(data) => self.node_text(&data.name),
            NodeData::VariableStatement(_) => String::new(),
            NodeData::FunctionDeclaration(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::FunctionExpression(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_FUNCTION.to_string()),
            NodeData::ArrowFunction(_) => INTERNAL_SYMBOL_NAME_FUNCTION.to_string(),
            NodeData::ClassDeclaration(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::ClassExpression(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_CLASS.to_string()),
            NodeData::InterfaceDeclaration(data) => self.node_text(&data.name),
            NodeData::TypeAliasDeclaration(data) => self.node_text(&data.name),
            NodeData::EnumDeclaration(data) => self.node_text(&data.name),
            NodeData::ModuleDeclaration(data) => self.node_text(&data.name),
            NodeData::ParameterDeclaration(data) => {
                if data.name.kind == SyntaxKind::ThisKeyword {
                    "this".to_string()
                } else {
                    self.node_text(&data.name)
                }
            }
            NodeData::BindingElement(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),

            NodeData::ImportSpecifier(data) => self.node_text(&data.name),
            NodeData::ImportClause(data) => data.name.as_ref().map_or_else(
                || {
                    data.named_bindings
                        .as_ref()
                        .map_or_else(|| String::new(), |n| self.node_text(n))
                },
                |n| self.node_text(n),
            ),
            NodeData::PropertyDeclaration(data) => self.node_text(&data.name),
            NodeData::JsxAttribute(data) => data
                .name
                .jsx_namespaced_name_text()
                .unwrap_or_else(|| self.node_text(&data.name)),
            NodeData::MethodDeclaration(data) => self.node_text(&data.name),
            NodeData::MethodSignatureDeclaration(data) => self.node_text(&data.name),
            NodeData::PropertySignatureDeclaration(data) => self.node_text(&data.name),
            NodeData::PropertyAssignment(data) => self.node_text(&data.name),
            NodeData::ShorthandPropertyAssignment(data) => self.node_text(&data.name),
            NodeData::EnumMember(data) => self.node_text(&data.name),
            NodeData::GetAccessorDeclaration(data) => self.node_text(&data.name),
            NodeData::SetAccessorDeclaration(data) => self.node_text(&data.name),
            NodeData::TypeParameterDeclaration(data) => self.node_text(&data.name),

            NodeData::ImportEqualsDeclaration(data) => self.node_text(&data.name),
            NodeData::NamespaceImport(data) => self.node_text(&data.name),

            NodeData::ExportSpecifier(data) => self.node_text(&data.name),
            NodeData::Identifier(data) => data.text.clone(),

            NodeData::ExportAssignment(data) => {
                if data.is_export_equals {
                    INTERNAL_SYMBOL_NAME_EXPORT_EQUALS.to_string()
                } else {
                    INTERNAL_SYMBOL_NAME_DEFAULT.to_string()
                }
            }

            NodeData::ExportDeclaration(_) => INTERNAL_SYMBOL_NAME_EXPORT_STAR.to_string(),

            NodeData::NamespaceExport(data) => self.node_text(&data.name),
            NodeData::NamespaceExportDeclaration(data) => self.node_text(&data.name),
            NodeData::BinaryExpression(bin) => {
                match crate::binder::get_assignment_declaration_kind(node) {
                    crate::binder::bind_js_assignment_declarations::JsDeclarationKind::Property
                    | crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ThisProperty
                    | crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ExportsProperty => {
                        crate::binder::bind_js_assignment_declarations::get_element_or_property_access_name(&bin.left)
                            .map(|n| self.node_text(&n))
                            .unwrap_or_else(|| self.node_text(&bin.left))
                    }
                    _ => INTERNAL_SYMBOL_NAME_EXPORT_EQUALS.to_string(),
                }
            }
            NodeData::CallExpression(call) => {
                match crate::binder::get_assignment_declaration_kind(node) {
                    crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ObjectDefinePropertyValue
                    | crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ObjectDefinePropertyExports => {
                        call.arguments
                            .nodes
                            .get(1)
                            .map(|n| self.node_text(n))
                            .unwrap_or_default()
                    }
                    _ => String::new(),
                }
            }
            _ => String::new(),
        }
    }

    pub(crate) fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(data) => data.text.clone(),

            NodeData::PrivateIdentifier(data) => data.text.clone(),
            NodeData::StringLiteral(data) => data.text.clone(),
            NodeData::NumericLiteral(data) => data.text.clone(),
            NodeData::NoSubstitutionTemplateLiteral(data) => data.text.clone(),
            NodeData::BigIntLiteral(data) => data.text.clone(),
            _ => String::new(),
        }
    }
}
