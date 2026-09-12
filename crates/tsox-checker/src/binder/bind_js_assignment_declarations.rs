#![allow(unused_imports)]

use crate::binder::symbols::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsDeclarationKind {
    None,
    /// module.exports = expr（module.exports = exports 除外）
    ModuleExports,
    /// exports.name = expr / module.exports.name = expr
    ExportsProperty,
    /// this.name = expr
    ThisProperty,
    /// F.name = expr / F[name] = expr（JS 或 TS 文件）
    Property,
    /// Object.defineProperty(x, 'name', ...)
    ObjectDefinePropertyValue,
    /// Object.defineProperty(exports || module.exports, 'name', ...)
    ObjectDefinePropertyExports,
}

pub(crate) fn get_assignment_declaration_kind(node: &Arc<Node>) -> JsDeclarationKind {
    // JavaScriptFile 标志只落在 SourceFile 节点上；沿祖先判定
    fn is_in_js(n: &Arc<Node>) -> bool {
        let mut cur = Some(Arc::clone(n));
        while let Some(c) = cur {
            if c.flags.contains(NodeFlags::JavaScriptFile) {
                return true;
            }
            cur = c.parent();
        }
        false
    }
    match &node.data {
        NodeData::BinaryExpression(bin) => {
            if bin.operator_token.kind != SyntaxKind::EqualsToken || !is_access_expression(&bin.left)
            {
                return JsDeclarationKind::None;
            }
            if is_in_js(node) {
                let base = bin_left_expression(&bin.left);
                if is_module_exports_access_expression(&bin.left)
                    && !is_exports_identifier(&bin.right)
                {
                    return JsDeclarationKind::ModuleExports;
                }
                if (is_module_exports_access_expression(&base) || is_exports_identifier(&base))
                    && get_element_or_property_access_name(&bin.left).is_some()
                {
                    return JsDeclarationKind::ExportsProperty;
                }
                if base.kind == SyntaxKind::ThisKeyword {
                    return JsDeclarationKind::ThisProperty;
                }
            }
            if bin.left.kind == SyntaxKind::PropertyAccessExpression
                && is_entity_name_expression(&bin_left_expression(&bin.left))
                && bin_left_name(&bin.left).is_some_and(|n| n.kind == SyntaxKind::Identifier)
                || bin.left.kind == SyntaxKind::ElementAccessExpression
                    && is_entity_name_expression(&bin_left_expression(&bin.left))
            {
                return JsDeclarationKind::Property;
            }
            JsDeclarationKind::None
        }
        NodeData::CallExpression(call) => {
            if is_in_js(node)
                && is_bindable_object_define_property_call(call)
            {
                let entity = &call.arguments.nodes[0];
                if is_exports_identifier(entity) || is_module_exports_access_expression(entity) {
                    return JsDeclarationKind::ObjectDefinePropertyExports;
                }
                return JsDeclarationKind::ObjectDefinePropertyValue;
            }
            JsDeclarationKind::None
        }
        _ => JsDeclarationKind::None,
    }
}

pub(crate) fn expression_is_alias(node: &Arc<Node>) -> bool {
    matches!(node.kind, SyntaxKind::Identifier | SyntaxKind::QualifiedName)
        || node.kind == SyntaxKind::ClassExpression
}

pub(crate) fn is_exports_identifier(node: &Arc<Node>) -> bool {
    matches!(&node.data, NodeData::Identifier(i) if i.text == "exports")
}

pub(crate) fn is_module_identifier(node: &Arc<Node>) -> bool {
    matches!(&node.data, NodeData::Identifier(i) if i.text == "module")
}

pub(crate) fn is_module_exports_access_expression(node: &Arc<Node>) -> bool {
    if is_access_expression(node) && is_module_identifier(&access_expression_base(node)) {
        if let Some(name) = get_element_or_property_access_name(node) {
            return name.text() == "exports";
        }
    }
    false
}

fn is_access_expression(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
    )
}

fn access_expression_base(node: &Arc<Node>) -> Arc<Node> {
    match &node.data {
        NodeData::PropertyAccessExpression(d) => Arc::clone(&d.expression),
        NodeData::ElementAccessExpression(d) => Arc::clone(&d.expression),
        _ => Arc::clone(node),
    }
}

fn bin_left_expression(left: &Arc<Node>) -> Arc<Node> {
    access_expression_base(left)
}

fn bin_left_name(left: &Arc<Node>) -> Option<Arc<Node>> {
    match &left.data {
        NodeData::PropertyAccessExpression(d) => Some(Arc::clone(&d.name)),
        _ => None,
    }
}

fn is_entity_name_expression(node: &Arc<Node>) -> bool {
    match &node.data {
        NodeData::Identifier(i) => !matches!(i.text.as_str(), "this" | "super" | "null" | "true" | "false"),
        NodeData::PropertyAccessExpression(d) => is_entity_name_expression(&d.expression),
        _ => false,
    }
}

/// Go GetElementOrPropertyAccessName：属性访问名或字符串/数字字面量下标
pub(crate) fn get_element_or_property_access_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::PropertyAccessExpression(d) => Some(Arc::clone(&d.name)),
        NodeData::ElementAccessExpression(d) => {
            let arg = &d.argument_expression;
            if matches!(
                arg.kind,
                SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
            ) {
                Some(Arc::clone(arg))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_bindable_object_define_property_call(call: &tsox_frontend::ast::CallExpressionData) -> bool {
    if call.arguments.nodes.len() != 3 {
        return false;
    }
    let callee = &call.expression;
    let NodeData::PropertyAccessExpression(pae) = &callee.data else {
        return false;
    };
    is_module_exports_style_object(&pae.expression)
        && pae.name.kind == SyntaxKind::Identifier
        && pae.name.text() == "defineProperty"
        && matches!(
            call.arguments.nodes[1].kind,
            SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
        )
        && is_bindable_static_name_expression(&call.arguments.nodes[0])
}

fn is_module_exports_style_object(node: &Arc<Node>) -> bool {
    matches!(&node.data, NodeData::Identifier(i) if i.text == "Object")
}

fn is_bindable_static_name_expression(node: &Arc<Node>) -> bool {
    match &node.data {
        NodeData::Identifier(i) => {
            !matches!(i.text.as_str(), "this" | "super" | "null" | "true" | "false")
        }
        NodeData::PropertyAccessExpression(d) => {
            is_bindable_static_name_expression(&d.expression) && d.name.kind == SyntaxKind::Identifier
        }
        NodeData::ElementAccessExpression(d) => {
            is_bindable_static_name_expression(&d.expression)
                && matches!(
                    d.argument_expression.kind,
                    SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
                )
        }
        _ => false,
    }
}

impl Binder {
    /// Go bindModuleExportsAssignment：module.exports = expr 在文件模块的
    /// exports 表里声明 "export=" 符号
    pub(crate) fn bind_module_exports_assignment(&mut self, node: &Arc<Node>) {
        if !self.set_common_js_module_indicator(node) {
            return;
        }
        let Some(file_sym) = self.parent_symbol.clone() else {
            return;
        };
        let NodeData::BinaryExpression(bin) = &node.data else {
            return;
        };
        let flags = if expression_is_alias(&bin.right) {
            SymbolFlags::Alias
        } else {
            SymbolFlags::Property
        };
        let symbol = self.declare_symbol_into(
            node,
            flags,
            SymbolFlags::None,
            DeclareTarget::Exports(file_sym),
        );
        let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
        unsafe {
            (*symbol_mut).value_declaration = Some(Arc::clone(node));
        }
    }

    /// Go bindExportsOrObjectDefineProperty：exports.name = expr 与
    /// Object.defineProperty(exports, 'name', ...) 的具名导出
    pub(crate) fn bind_exports_or_object_define_property(&mut self, node: &Arc<Node>) {
        if !self.set_common_js_module_indicator(node) {
            return;
        }
        let Some(file_sym) = self.parent_symbol.clone() else {
            return;
        };
        let flags = match &node.data {
            NodeData::BinaryExpression(bin) if expression_is_alias(&bin.right) => {
                SymbolFlags::Alias
            }
            _ => SymbolFlags::FunctionScopedVariable,
        };
        self.declare_symbol_into(
            node,
            flags,
            SymbolFlags::FunctionScopedVariableExcludes,
            DeclareTarget::Exports(file_sym),
        );
    }

    pub(crate) fn set_common_js_module_indicator(&mut self, node: &Arc<Node>) -> bool {
        let Some(file) = self.current_source_file.clone() else {
            return false;
        };
        if let Some(indicator) = &file.external_module_indicator
            && !Arc::ptr_eq(indicator, &file.node)
        {
            return false;
        }
        if file.common_js_module_indicator.is_none() {
            let file_mut = Arc::as_ptr(&file) as *mut SourceFile;
            unsafe {
                (*file_mut).common_js_module_indicator = Some(Arc::clone(node));
            }
        }
        true
    }
}
