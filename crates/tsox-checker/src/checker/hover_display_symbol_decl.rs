//! Go hover.go writeSymbol 的声明类分支：类/接口/枚举/命名空间。

use std::sync::Arc;

use super::hover_display_parts::HoverPartsBuilder;
use crate::checker::nodebuilder::*;
use tsox_frontend::ast::{Symbol, SymbolFlags, SyntaxKind};

impl Checker {
    pub(crate) fn hover_write_class_or_interface(
        &mut self,
        b: &mut HoverPartsBuilder,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) {
        let is_class = symbol.flags.intersects(SymbolFlags::Class);
        // ThisKeyword/ThisType：符号为容器类/接口时只显示 this
        if node.kind == SyntaxKind::ThisKeyword || node.kind == SyntaxKind::ThisType {
            b.write_new_line();
            b.write_keyword("this");
            return;
        }
        if node.kind == SyntaxKind::ConstructorKeyword
            && let Some(parent) = node.parent.as_ref()
            && matches!(
                parent.kind,
                SyntaxKind::Constructor | SyntaxKind::ConstructSignature
            )
        {
            let t = self.get_type_of_function_like(parent);
            if let Some(sig) = t.as_structured().and_then(|s| s.call_signatures().first().cloned())
            {
                self.hover_write_signatures(b, &[sig], "constructor ", false, symbol, None);
                let doc = self.constructor_jsdoc_documentation(symbol);
                if !doc.is_empty() {
                    b.write_space("\n\n");
                    b.write_text(&doc, DisplayPartKind::Text);
                }
            }
            return;
        }
        // 构造位：旧 constructor_display_parts（含 new C<any> 显式类型实参形态）
        if let Some(parts) = self.constructor_display_parts(symbol, node) {
            b.extend(parts);
            let doc = self.constructor_jsdoc_documentation(symbol);
            if !doc.is_empty() {
                b.write_space("\n\n");
                b.write_text(&doc, DisplayPartKind::Text);
            }
            return;
        }
        // 构造位：new C() 的 C → 构造签名
        let call = self.hover_call_or_new_expression(node);
        if is_class && let Some(call) = &call {
            let t = self.get_type_of_symbol(symbol);
            let ctors = t
                .as_structured()
                .map(|s| s.construct_signatures().to_vec())
                .unwrap_or_default();
            if !ctors.is_empty()
                && let Some(resolved) = self.resolved_call_signature(call)
            {
                self.hover_write_signatures(
                    b, &[resolved], "constructor ", false, symbol, Some(call),
                );
                let doc = self.constructor_jsdoc_documentation(symbol);
                if !doc.is_empty() {
                    b.write_space("\n\n");
                    b.write_text(&doc, DisplayPartKind::Text);
                }
                return;
            }
        }
        b.write_new_line();
        if is_class {
            let is_local = symbol
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::ClassExpression);
            if is_local {
                b.write_punctuation("(");
                b.write_text("local class", DisplayPartKind::Text);
                b.write_punctuation(") ");
            } else {
                if symbol.declarations.iter().any(|d| {
                    d.kind == SyntaxKind::ClassDeclaration
                        && d.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Abstract)
                }) {
                    b.write_keyword("abstract ");
                }
                b.write_keyword("class ");
            }
            let kind = DisplayPartKind::ClassName;
            b.write_text(&self.qualified_symbol_name(symbol), kind);
            self.hover_write_declared_type_params(b, symbol);
        } else {
            b.write_keyword("interface ");
            b.write_text(&self.qualified_symbol_name(symbol), DisplayPartKind::InterfaceName);
            self.hover_write_declared_type_params(b, symbol);
        }
    }

    pub(crate) fn hover_write_enum(&mut self, b: &mut HoverPartsBuilder, symbol: &Arc<Symbol>) {
        b.write_new_line();
        if symbol.declarations.iter().any(|d| {
            d.kind == SyntaxKind::EnumDeclaration
                && d.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Const)
        }) {
            b.write_keyword("const ");
        }
        b.write_keyword("enum ");
        b.write_text(&symbol.name, DisplayPartKind::EnumName);
    }

    pub(crate) fn hover_write_module(&mut self, b: &mut HoverPartsBuilder, symbol: &Arc<Symbol>) {
        b.write_new_line();
        let is_module = symbol
            .value_declaration
            .as_ref()
            .is_some_and(|d| d.kind == SyntaxKind::SourceFile || tsox_frontend::ast::is_ambient_module(d));
        b.write_keyword(if is_module { "module " } else { "namespace " });
        b.write_text(&self.hover_symbol_name(symbol, None), DisplayPartKind::Text);
    }

}
