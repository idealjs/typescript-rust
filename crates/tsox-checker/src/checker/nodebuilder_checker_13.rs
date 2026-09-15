#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

impl Checker {
    pub(crate) fn format_quick_info_for_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) -> String {
        let flags = symbol.flags;

        if flags.intersects(SymbolFlags::Function) {
            return self.format_function_quick_info(symbol, false);
        }
        if flags.intersects(SymbolFlags::Method) {
            return self.format_function_quick_info(symbol, true);
        }
        if flags.intersects(SymbolFlags::Class) {
            return self.format_class_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::Interface) {
            return self.format_interface_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::ENUM) {
            return self.format_enum_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::TypeAlias) {
            return self.format_type_alias_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::TypeParameter) {
            return self.format_type_parameter_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::EnumMember) {
            return self.format_enum_member_quick_info(symbol);
        }

        if flags.intersects(SymbolFlags::VARIABLE)
            || flags.intersects(SymbolFlags::Property)
            || flags.intersects(SymbolFlags::ACCESSOR)
        {
            return self.format_variable_quick_info(symbol, node);
        }
        if flags.intersects(SymbolFlags::MODULE) {
            return format!("module {}", symbol.name);
        }
        if flags.intersects(SymbolFlags::NamespaceModule) {
            return format!("namespace {}", symbol.name);
        }
        if flags.intersects(SymbolFlags::Alias) {
            return self.format_alias_quick_info(symbol);
        }

        let t = self.get_type_of_symbol(symbol);
        format!("{}: {}", symbol.name, self.type_to_string(&t))
    }

    /// JS 函数参数符号的文档：所在函数 JSDoc 的同名 @param tag 注释
    /// （Go getDocumentationForSymbol 对 parameter 走 JSDocParameterTag）
    pub(crate) fn jsdoc_param_tag_documentation(&mut self, symbol: &Arc<Symbol>) -> Option<String> {
        let decl = symbol
            .value_declaration
            .as_ref()
            .or_else(|| symbol.declarations.first())?;
        if decl.kind != SyntaxKind::Parameter {
            return None;
        }
        let sf = self.get_source_file_of_node(decl)?;
        let fn_node = decl.parent()?;
        let jds = tsox_frontend::parser::parse_jsdoc_for_node(&sf, &fn_node);
        for jd in &jds {
            let NodeData::JSDoc(doc) = &jd.data else {
                continue;
            };
            let Some(tags) = &doc.tags else {
                continue;
            };
            for tag in tags.nodes.iter() {
                let NodeData::JSDocParameterOrPropertyTag(pd) = &tag.data else {
                    continue;
                };
                if pd.name.kind != SyntaxKind::Identifier || pd.name.text() != symbol.name {
                    continue;
                }
                let text = jsdoc_comment_text(&pd.comment);
                let text = text.trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        None
    }

    pub(crate) fn format_function_quick_info(
        &mut self,
        symbol: &Arc<Symbol>,
        is_method: bool,
    ) -> String {
        let prefix = if is_method { "" } else { "function " };
        let name = self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::WriteTypeParametersOrArguments,
            SymbolFlags::all(),
        );
        let t = self.get_type_of_symbol(symbol);

        if let Some(structured) = t.as_structured() {
            if let Some(sig) = structured.call_signatures().first() {
                let params = self.format_signature_parameters(sig);
                let ret = sig
                    .resolved_return_type
                    .get()
                    .cloned()
                    .unwrap_or_else(|| self.any_type());
                let ret_str = self.type_to_string(&ret);
                return format!("{}{}({}): {}", prefix, name, params, ret_str);
            }
        }

        format!("{}{}: {}", prefix, name, self.type_to_string(&t))
    }

    pub(crate) fn format_class_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let name = self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::WriteTypeParametersOrArguments,
            SymbolFlags::all(),
        );
        format!("class {}", name)
    }

    pub(crate) fn format_interface_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let name = self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::WriteTypeParametersOrArguments,
            SymbolFlags::all(),
        );
        format!("interface {}", name)
    }

    pub(crate) fn format_enum_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        format!("enum {}", symbol.name)
    }

    pub(crate) fn format_type_alias_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let name = self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::WriteTypeParametersOrArguments,
            SymbolFlags::all(),
        );

        if let Some(t) = self.try_get_type_alias_declared_type(symbol) {
            let t_str = self.type_to_string(&t);
            format!("type {} = {}", name, t_str)
        } else {
            format!("type {}", name)
        }
    }

    pub(crate) fn format_type_parameter_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let constraint = self.get_constraint_of_type_parameter_symbol(symbol);
        match constraint {
            Some(c) => format!("{} extends {}", symbol.name, self.type_to_string(&c)),
            None => symbol.name.clone(),
        }
    }

    pub(crate) fn format_enum_member_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let t = self.get_type_of_symbol(symbol);
        format!("{}.{}", "<enum>", self.type_to_string(&t))
    }

    pub(crate) fn format_variable_quick_info(
        &mut self,
        symbol: &Arc<Symbol>,
        _node: &Arc<Node>,
    ) -> String {
        let prefix = self.variable_decl_prefix(symbol);
        let t = self.get_type_of_symbol(symbol);
        format!("{}{}: {}", prefix, symbol.name, self.type_to_string(&t))
    }

    pub(crate) fn variable_decl_prefix(&self, symbol: &Arc<Symbol>) -> &'static str {
        for decl in &symbol.declarations {
            if let Some(parent) = decl.parent() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    if parent.flags.contains(tsox_frontend::ast::NodeFlags::Const) {
                        return "const ";
                    }
                    if parent.flags.contains(tsox_frontend::ast::NodeFlags::Let) {
                        return "let ";
                    }

                    return "var ";
                }
            }
        }

        // 绑定元素：穿透模式链找 VariableDeclarationList 的声明风格
        for decl in &symbol.declarations {
            if decl.kind == SyntaxKind::BindingElement {
                let mut cur = decl.parent();
                while let Some(n) = cur {
                    if n.kind == SyntaxKind::VariableDeclaration {
                        if let Some(p) = n.parent().as_ref() {
                            if p.kind == SyntaxKind::VariableDeclarationList {
                                if p.flags.contains(tsox_frontend::ast::NodeFlags::Const) {
                                    return "const ";
                                }
                                if p.flags.contains(tsox_frontend::ast::NodeFlags::Let) {
                                    return "let ";
                                }
                                return "var ";
                            }
                        }
                    }
                    if matches!(
                        n.kind,
                        SyntaxKind::ObjectBindingPattern
                            | SyntaxKind::ArrayBindingPattern
                            | SyntaxKind::BindingElement
                    ) {
                        cur = n.parent();
                        continue;
                    }
                    break;
                }
            }
        }
        if symbol.flags.contains(SymbolFlags::BlockScopedVariable)
            && !symbol.declarations.iter().any(|d| {
                d.parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::CatchClause)
            })
        {
            "let "
        } else {
            "var "
        }
    }

    pub(crate) fn format_alias_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        format!("import {}", symbol.name)
    }

    pub(crate) fn format_signature_parameters(&mut self, sig: &Signature) -> String {
        let parts: Vec<String> = sig
            .parameters
            .iter()
            .map(|param| {
                let name = param.name.clone();
                let param_type = self.get_type_of_symbol(param);
                let type_str = self.type_to_string(&param_type);
                if param.flags.contains(SymbolFlags::Optional) {
                    format!("{}?: {}", name, type_str)
                } else {
                    format!("{}: {}", name, type_str)
                }
            })
            .collect();
        parts.join(", ")
    }

    #[allow(dead_code)]
    pub(crate) fn symbol_is_const(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(parent) = decl.parent() {
                if parent.kind == SyntaxKind::VariableDeclarationList
                    && parent.flags.contains(tsox_frontend::ast::NodeFlags::Const)
                {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn try_get_type_alias_declared_type(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(t) = &links.declared_type {
                // 环窗口期的 error 驻留视为未解析，触发窗口外重算
                if !crate::checker::utilities::is_type_error(t) {
                    return Some(Arc::clone(t));
                }
            }
        }

        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
        if !self.push_type_resolution(key, crate::checker::TypeResolutionProperty::DeclaredType) {
            return None;
        }
        let result = self.resolve_alias_body(symbol);
        self.pop_type_resolution();

        if !crate::checker::utilities::is_type_error(&result) {
            self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&result));
        }
        Some(result)
    }

    pub(crate) fn get_constraint_of_type_parameter_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        let t = self.get_type_of_symbol(symbol);
        if t.flags.contains(TypeFlags::TypeParameter) {
            return self.get_constraint_of_type_parameter(&t);
        }
        None
    }

    pub(crate) fn node_has_type(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::NumericLiteral
                | SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::TemplateExpression
                | SyntaxKind::ArrayLiteralExpression
                | SyntaxKind::ObjectLiteralExpression
                | SyntaxKind::BinaryExpression
                | SyntaxKind::PrefixUnaryExpression
                | SyntaxKind::PostfixUnaryExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::NewExpression
                | SyntaxKind::PropertyAccessExpression
                | SyntaxKind::ElementAccessExpression
                | SyntaxKind::ParenthesizedExpression
                | SyntaxKind::ConditionalExpression
                | SyntaxKind::TypeAssertionExpression
                | SyntaxKind::AsExpression
                | SyntaxKind::NonNullExpression
        )
    }
}

pub(crate) const MAX_SERIALIZATION_LEVEL: i32 = 2;

/// JSDoc tag comment 的 NodeList → 纯文本
fn jsdoc_comment_text(comment: &Option<Arc<tsox_frontend::ast::NodeList>>) -> String {
    let Some(list) = comment else {
        return String::new();
    };
    let mut out = String::new();
    for part in list.iter() {
        if let NodeData::JSDocText(td) = &part.data {
            out.push_str(&td.text.join(""));
        }
    }
    out
}
