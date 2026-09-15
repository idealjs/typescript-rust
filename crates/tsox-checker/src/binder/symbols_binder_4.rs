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
            NodeData::ModuleDeclaration(data) => {
                let name_text = self.node_text(&data.name);
                // Go getDeclarationName：带 import attributes 的 pattern ambient 模块
                // 用唯一名（~pattern@id）避免同名 pattern 在 bind 期合并
                let file_text = self
                    .current_source_file
                    .as_ref()
                    .map(|f| f.text.as_str())
                    .unwrap_or("");
                if data.name.kind == SyntaxKind::StringLiteral
                    && name_text.matches('*').count() == 1
                    && module_declaration_has_with_clause(
                        file_text,
                        node,
                        &data.name,
                        data.body.as_ref(),
                    )
                {
                    format!("~{name_text}@pattern@{}", node.id())
                } else {
                    name_text
                }
            }
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
            NodeData::JsxAttribute(data) => data
                .name
                .jsx_namespaced_name_text()
                .unwrap_or_else(|| self.node_text(&data.name)),
            NodeData::MethodDeclaration(data) => self.member_name_text(&data.name),
            NodeData::MethodSignatureDeclaration(data) => self.member_name_text(&data.name),
            NodeData::PropertySignatureDeclaration(data) => self.member_name_text(&data.name),
            NodeData::PropertyAssignment(data) => self.member_name_text(&data.name),
            NodeData::ShorthandPropertyAssignment(data) => self.node_text(&data.name),
            NodeData::EnumMember(data) => self.node_text(&data.name),
            NodeData::GetAccessorDeclaration(data) => self.member_name_text(&data.name),
            NodeData::SetAccessorDeclaration(data) => self.member_name_text(&data.name),
            NodeData::PropertyDeclaration(data) => self.member_name_text(&data.name),
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

    /// 成员声明名：计算属性名为 `Symbol.<知名符号>` 形态时用内部名
    /// `__@<name>`（Go getDeclarationName 的 well-known symbol 分支等价，
    /// 去掉 Go 的 @symbolId 后缀，两端一致即可命中）
    pub(crate) fn member_name_text(&self, name: &Arc<Node>) -> String {
        if name.kind == SyntaxKind::ComputedPropertyName
            && let NodeData::ComputedPropertyName(cd) = &name.data
            && let Some(internal) = well_known_symbol_member_name(&cd.expression)
        {
            return internal;
        }
        self.node_text(name)
    }
}

pub(crate) fn well_known_symbol_member_name(expr: &Arc<Node>) -> Option<String> {
    if let NodeData::PropertyAccessExpression(pa) = &expr.data
        && let NodeData::Identifier(base) = &pa.expression.data
        && base.text == "Symbol"
        && pa.name.kind == SyntaxKind::Identifier
    {
        let prop = match &pa.name.data {
            NodeData::Identifier(id) => id.text.as_str(),
            _ => return None,
        };
        if matches!(
            prop,
            "iterator"
                | "asyncIterator"
                | "custom"
                | "dispose"
                | "asyncDispose"
                | "hasInstance"
                | "isConcatSpreadable"
                | "match"
                | "matchAll"
                | "replace"
                | "search"
                | "species"
                | "split"
                | "toPrimitive"
                | "toStringTag"
                | "unscopables"
        ) {
            return Some(format!("__@{prop}"));
        }
    }
    None
}

pub(crate) fn module_declaration_has_with_clause(
    source_text: &str,
    node: &Arc<Node>,
    name: &Arc<Node>,
    body: Option<&Arc<Node>>,
) -> bool {
    let start = name.loc.end().min(source_text.len());
    let end = body
        .map(|b| b.loc.pos())
        .unwrap_or(node.loc.end())
        .max(start)
        .min(source_text.len());
    source_text[start..end].trim_start().starts_with("with")
}
