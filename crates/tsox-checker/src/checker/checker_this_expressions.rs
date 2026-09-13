use std::sync::Arc;

use super::checker::Checker;
use super::checker::Signature;
use super::checker_this_container::{
    get_this_container, get_this_parameter, is_in_parameter_initializer_before_containing_function,
};
use super::types::{ContextFlags, Type, TypeFlags};
use tsox_frontend::ast::{
    Node, NodeData, SyntaxKind, has_static_modifier, is_class_like, is_function_like_kind,
};

impl Checker {
    /// Go checkThisExpression 的类型解析主干（诊断与 flow 收窄不在此层）
    pub(crate) fn this_expression_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let container = get_this_container(node, false, false);
        if is_function_like_kind(container.kind)
            && (!is_in_parameter_initializer_before_containing_function(node)
                || get_this_parameter(&container).is_some())
        {
            if let Some(this_param) = get_this_parameter(&container) {
                return self.type_of_this_parameter_node(&this_param);
            }
            if let Some(t) = self.contextual_this_parameter_type(&container) {
                return t;
            }
        }
        if let Some(parent) = container.parent()
            && is_class_like(&parent)
        {
            if has_static_modifier(&container) {
                return self.get_type_of_class_declaration(&parent);
            }
            let instance = self.container_instance_type_of(&parent);
            return self.create_this_type(&parent, instance);
        }
        if container.kind == SyntaxKind::SourceFile {
            if self.container_file_is_external_module(&container) {
                return self.undefined_type();
            }
            if let Some(t) = self.global_this_type_value() {
                return t;
            }
        }
        self.get_any_type()
    }

    /// Go getContextualThisParameterType
    fn contextual_this_parameter_type(&mut self, container: &Arc<Node>) -> Option<Arc<Type>> {
        if container.kind == SyntaxKind::ArrowFunction {
            return None;
        }
        if (container.kind == SyntaxKind::FunctionExpression || is_object_literal_method(container))
            && self.is_context_sensitive(container)
            && let Some(contextual_signature) = self.contextual_signature_for(container)
            && let Some(this_param) = &contextual_signature.this_parameter
        {
            let t = self.get_type_of_symbol(this_param);
            // Go 对显式 this 参数的 this 型类型参数取其约束（接口 this 经
            // contextual 通道显示为接口本身而非 "this"）
            if t.flags.contains(TypeFlags::TypeParameter)
                && let Some(constraint) = self.get_constraint_of_type_parameter(&t)
            {
                return Some(constraint);
            }
            return Some(t);
        }
        if self.no_implicit_this || tsox_frontend::ast::is_in_js_file(container) {
            if let Some(literal) = containing_object_literal(container) {
                return match self.get_contextual_type(&literal, ContextFlags::None) {
                    Some(t) => Some(self.get_non_nullable_type_of(&t)),
                    None => {
                        let literal_type = self.get_type_of_node(&literal);
                        Some(self.get_widened_type(&literal_type))
                    }
                };
            }
            if let Some(assigned_object) = assignment_target_object(container) {
                let t = self.get_type_of_node(&assigned_object);
                return Some(self.get_widened_type(&t));
            }
        }
        None
    }

    /// Go getContextualSignature：对象字面量方法经所在字面量的 contextual type
    /// 取方法属性的调用签名；其余走常规 contextual signature
    fn contextual_signature_for(&mut self, container: &Arc<Node>) -> Option<Arc<Signature>> {
        if is_object_literal_method(container) {
            let literal = container.parent()?;
            let name_node = container.name()?;
            let name = self.get_property_name_from_node(&name_node);
            let ctx = self.get_contextual_type(&literal, ContextFlags::Signature)?;
            let prop_type = self.get_type_of_property_of_contextual_type(&ctx, &name)?;
            return self
                .get_signatures_of_type(&prop_type, crate::checker::SignatureKind::Call)
                .into_iter()
                .next();
        }
        self.get_contextual_signature(container)
    }

    fn type_of_this_parameter_node(&mut self, param: &Arc<Node>) -> Arc<Type> {
        if let Some(symbol) = self.get_symbol_of_declaration(param) {
            return self.get_type_of_symbol(&symbol);
        }
        self.get_any_type()
    }

    fn global_this_type_value(&mut self) -> Option<Arc<Type>> {
        if let Some(t) = self.global_this_type.get() {
            return Some(Arc::clone(t));
        }
        let symbol = self.global_this_symbol.clone()?;
        let t = self.get_type_of_symbol(&symbol);
        let _ = self.global_this_type.set(Arc::clone(&t));
        Some(t)
    }
}

fn is_object_literal_method(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::MethodDeclaration
        && node
            .parent()
            .as_ref()
            .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
}

/// Go getContainingObjectLiteral：函数/方法所在的最近对象字面量（不跨函数边界）
fn containing_object_literal(fn_node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = fn_node.parent();
    while let Some(n) = current {
        match n.kind {
            SyntaxKind::ObjectLiteralExpression => return Some(n),
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::SourceFile => return None,
            _ => {}
        }
        current = n.parent();
    }
    None
}

/// `obj.xxx = function(...) {...}` 形态下 obj 的表达式节点
fn assignment_target_object(fn_node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut parent = fn_node.parent()?;
    while parent.kind == SyntaxKind::ParenthesizedExpression {
        parent = parent.parent()?;
    }
    let NodeData::BinaryExpression(d) = &parent.data else {
        return None;
    };
    if d.operator_token.kind != SyntaxKind::EqualsToken {
        return None;
    }
    let target = &d.left;
    match &target.data {
        NodeData::PropertyAccessExpression(pd) => Some(Arc::clone(&pd.expression)),
        NodeData::ElementAccessExpression(ed) => Some(Arc::clone(&ed.expression)),
        _ => None,
    }
}

impl Checker {
    fn container_file_is_external_module(&self, source_file_node: &Arc<Node>) -> bool {
        if let Some(f) = &self.current_file {
            return f.external_module_indicator.is_some();
        }
        match &source_file_node.data {
            NodeData::SourceFile(d) => d.statements.iter().any(is_module_indicator),
            _ => false,
        }
    }
}

fn is_module_indicator(statement: &Arc<Node>) -> bool {
    match &statement.data {
        NodeData::ExportDeclaration(d) => d.export_clause.is_some() || d.module_specifier.is_some(),
        NodeData::ImportEqualsDeclaration(d) => {
            matches!(&d.module_reference.data, NodeData::ExternalModuleReference(_))
        }
        _ => matches!(statement.kind, SyntaxKind::ImportDeclaration),
    }
}
