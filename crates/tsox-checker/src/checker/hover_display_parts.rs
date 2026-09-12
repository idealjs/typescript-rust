//! display parts 容器 + 签名/类型参数渲染（Go writeSignatures/writeTypeParams 移植）。

use std::sync::Arc;

use crate::checker::nodebuilder::*;
use crate::checker::types::Signature;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

pub(crate) struct HoverPartsBuilder {
    pub parts: Vec<SymbolDisplayPart>,
    pub alias_level: usize,
    pub declaration: Option<Arc<Node>>,
    visited_aliases: Vec<u64>,
}

impl HoverPartsBuilder {
    pub(crate) fn new() -> Self {
        Self {
            parts: Vec::new(),
            alias_level: 0,
            declaration: None,
            visited_aliases: Vec::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Go writeNewLine：已有内容先换行；alias 链层加 (alias) 前缀
    pub(crate) fn write_new_line(&mut self) {
        if !self.parts.is_empty() {
            push_space(&mut self.parts, "\n");
        }
        if self.alias_level != 0 {
            push_punctuation(&mut self.parts, "(");
            push_part(&mut self.parts, "alias", DisplayPartKind::Text);
            push_punctuation(&mut self.parts, ") ");
        }
    }

    pub(crate) fn visit_alias(&mut self, id: u64) -> bool {
        if self.visited_aliases.contains(&id) {
            return false;
        }
        self.visited_aliases.push(id);
        true
    }

    pub(crate) fn write_keyword(&mut self, text: &str) {
        push_keyword(&mut self.parts, text);
    }

    pub(crate) fn write_punctuation(&mut self, text: &str) {
        push_punctuation(&mut self.parts, text);
    }

    pub(crate) fn write_text(&mut self, text: &str, kind: DisplayPartKind) {
        push_part(&mut self.parts, text, kind);
    }

    pub(crate) fn write_space(&mut self, text: &str) {
        push_space(&mut self.parts, text);
    }

    pub(crate) fn extend(&mut self, other: Vec<SymbolDisplayPart>) {
        self.parts.extend(other);
    }
}

impl Checker {
    /// Go writeSignatures：重载逐条；3 条后截断为 “// +N more overloads”
    pub(crate) fn hover_write_signatures(
        &mut self,
        b: &mut HoverPartsBuilder,
        signatures: &[Arc<Signature>],
        prefix: &str,
        parenthesized: bool,
        symbol: &Arc<Symbol>,
        call: Option<&Arc<Node>>,
    ) {
        let total = signatures.len();
        for (i, sig) in signatures.iter().enumerate() {
            b.write_new_line();
            if i == 3 && total >= 5 {
                b.write_text(
                    &format!("// +{} more overloads", total - 3),
                    DisplayPartKind::Text,
                );
                break;
            }
            if parenthesized {
                b.write_punctuation("(");
                b.write_text(prefix, DisplayPartKind::Text);
                b.write_punctuation(") ");
            } else {
                b.write_keyword(prefix);
            }
            b.write_text(&self.qualified_symbol_name(symbol), DisplayPartKind::Text);
            if symbol.flags.contains(SymbolFlags::Optional) {
                b.write_punctuation("?");
            }
            self.hover_write_signature(b, sig, call);
        }
    }

    /// 单条签名：调用位类型实参推断 + 参数 + 返回段
    pub(crate) fn hover_write_signature(
        &mut self,
        b: &mut HoverPartsBuilder,
        sig: &Arc<Signature>,
        call: Option<&Arc<Node>>,
    ) {
        let mut sig = Arc::clone(sig);
        if let Some(call) = call
            && !sig.type_parameters.is_empty()
        {
            let args: Vec<Arc<Node>> = match &call.data {
                tsox_frontend::ast::NodeData::CallExpression(d) => {
                    d.arguments.iter().cloned().collect()
                }
                tsox_frontend::ast::NodeData::NewExpression(d) => d
                    .arguments
                    .as_ref()
                    .map(|a| a.iter().cloned().collect())
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
            let inferred = self.infer_call_type_arguments(call, &sig, &args);
            if !inferred.is_empty() {
                let names: Vec<String> = inferred
                    .iter()
                    .map(|t| {
                        self.type_to_string_ex(
                            t,
                            crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::MULTILINE_OBJECT_LITERALS
                                .union(
                                    crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE,
                                ),
                        )
                    })
                    .collect();
                b.write_punctuation("<");
                b.write_text(&names.join(", "), DisplayPartKind::Text);
                b.write_punctuation(">");
                sig = self.get_signature_instantiation(&sig, &inferred);
            } else {
                self.hover_write_type_params(b, &sig.type_parameters);
            }
        } else if !sig.type_parameters.is_empty() {
            self.hover_write_type_params(b, &sig.type_parameters);
        }
        b.write_punctuation("(");
        self.append_signature_parameter_parts(&mut b.parts, &sig);
        b.write_punctuation(")");
        self.append_signature_return_parts(&mut b.parts, &sig);
    }

    /// Go writeTypeParams：类型参数列表（含 extends 约束 / = default）
    pub(crate) fn hover_write_type_params(
        &mut self,
        b: &mut HoverPartsBuilder,
        params: &[Arc<crate::checker::types::Type>],
    ) {
        if params.is_empty() {
            return;
        }
        b.write_punctuation("<");
        for (i, tp) in params.iter().enumerate() {
            if i != 0 {
                b.write_punctuation(", ");
            }
            if let Some(sym) = &tp.symbol {
                b.write_text(&sym.name, DisplayPartKind::TypeParameterName);
            }
            if let Some(c) = self.get_constraint_of_type_parameter(tp) {
                b.write_keyword(" extends ");
                b.extend(self.type_to_display_parts(&c));
            }
            if let Some(d) = self.get_default_from_type_parameter(tp) {
                b.write_space(" = ");
                b.extend(self.type_to_display_parts(&d));
            }
        }
        b.write_punctuation(">");
    }

    /// 从 TypeParameterDeclaration 声明直接渲染 `<T extends C = D>`
    pub(crate) fn hover_write_type_params_from_decls(
        &mut self,
        b: &mut HoverPartsBuilder,
        tps: &tsox_frontend::ast::NodeList,
    ) {
        if tps.nodes.is_empty() {
            return;
        }
        b.write_punctuation("<");
        for (i, tp) in tps.nodes.iter().enumerate() {
            if i != 0 {
                b.write_punctuation(", ");
            }
            let tsox_frontend::ast::NodeData::TypeParameterDeclaration(d) = &tp.data else {
                continue;
            };
            b.write_text(&d.name.text(), DisplayPartKind::TypeParameterName);
            if let Some(constraint) = &d.constraint {
                b.write_keyword(" extends ");
                let t = self.get_type_from_type_node(constraint);
                b.write_text(&self.type_to_string(&t), DisplayPartKind::Text);
            }
            if let Some(default_type) = &d.default_type {
                b.write_space(" = ");
                let t = self.get_type_from_type_node(default_type);
                b.write_text(&self.type_to_string(&t), DisplayPartKind::Text);
            }
        }
        b.write_punctuation(">");
    }

    /// Go getSignaturesAtLocation：类型的签名集；调用/构造位收敛为已解析签名。
    /// 返回 (签名集, 调用位节点)
    pub(crate) fn hover_get_signatures_at_location(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) -> (Vec<Arc<Signature>>, Option<Arc<Node>>) {
        let t = self.get_type_of_symbol(symbol);
        let signatures = t
            .as_structured()
            .map(|s| s.call_signatures().to_vec())
            .unwrap_or_default();
        let call = self.hover_call_or_new_expression(node);
        if signatures.len() > 1
            || signatures.len() == 1 && !signatures[0].type_parameters.is_empty()
        {
            if let Some(ref call_node) = call
                && let Some(resolved) = self.resolved_call_signature(call_node)
            {
                return (vec![resolved], call);
            }
        }
        (signatures, call)
    }

    /// Go getCallOrNewExpression：属性访问名字段提升 + 父级调用/构造位
    pub(crate) fn hover_call_or_new_expression(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut node = Arc::clone(node);
        if let Some(parent) = node.parent().as_ref()
            && parent.kind == SyntaxKind::PropertyAccessExpression
            && let tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) = &parent.data
            && Arc::ptr_eq(&pa.name, &node)
        {
            node = Arc::clone(parent);
        }
        let parent = node.parent()?;
        match parent.kind {
            SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
                let expr = parent.expression()?;
                if Arc::ptr_eq(&expr, &node) {
                    Some(Arc::clone(&parent))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
