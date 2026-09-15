//! Go hover.go getDocumentationForSymbol 的顺序移植：
//! 签名文档 → root symbols 文档 → 声明 JSDoc → alias 目标文档。

use std::sync::Arc;

use crate::checker::nodebuilder::*;
use tsox_frontend::ast::{Node, Symbol, SyntaxKind};

impl Checker {
    /// hover 文档聚合（输出附加在 quickinfo 文本之后）
    pub(crate) fn hover_documentation_for_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
        declaration: Option<&Arc<Node>>,
    ) -> String {
        if let Some(doc) = self.hover_doc_from_signature(node) {
            return doc;
        }
        if let Some(doc) = self.hover_doc_from_root_symbols(symbol, declaration) {
            return doc;
        }
        // hover_documentation：调用位签名声明文档 + 声明层文档（语句层上探）+ alias 链
        self.hover_documentation(symbol, node)
    }

    /// 调用位的签名声明文档（Go documentationFromSignature）
    fn hover_doc_from_signature(&mut self, node: &Arc<Node>) -> Option<String> {
        let call = self.hover_call_or_new_expression(node)?;
        let sig = self.get_resolved_signature(&call)?;
        let decl = sig.declaration.as_ref()?;
        if !matches!(
            decl.kind,
            SyntaxKind::CallSignature | SyntaxKind::ConstructSignature
        ) {
            return None;
        }
        let doc = self.declaration_jsdoc_text(decl);
        if doc.is_empty() {
            None
        } else {
            Some(doc)
        }
    }

    /// 合并符号（interface+interface 等）多 root 声明文档聚合（Go documentationFromRootSymbols）
    fn hover_doc_from_root_symbols(
        &mut self,
        symbol: &Arc<Symbol>,
        declaration: Option<&Arc<Node>>,
    ) -> Option<String> {
        let roots = self.get_root_symbols(symbol);
        if roots.len() <= 1 {
            return None;
        }
        let mut docs: Vec<String> = Vec::new();
        for root in &roots {
            let decls: Vec<Arc<Node>> = if root.declarations.is_empty() {
                root.value_declaration.iter().cloned().collect()
            } else {
                root.declarations.clone()
            };
            for decl in &decls {
                let doc = self.declaration_jsdoc_text(decl).trim_end().to_string();
                if !doc.is_empty() && !docs.contains(&doc) {
                    docs.push(doc);
                }
            }
        }
        if docs.is_empty() {
            return None;
        }
        // declaration 文档作为候选之一仍可能为空：只要有聚合结果即返回
        let _ = declaration;
        Some(docs.join("\n"))
    }

    /// alias 目标声明文档（Go documentationFromAlias）
    #[allow(dead_code)]
    fn hover_doc_from_alias(&mut self, symbol: &Arc<Symbol>) -> String {
        if !symbol.flags.intersects(tsox_frontend::ast::SymbolFlags::Alias) {
            return String::new();
        }
        let Some(aliased) = self.follow_alias(symbol) else {
            return String::new();
        };
        let mut candidates: Vec<Arc<Symbol>> = vec![Arc::clone(&aliased)];
        if let Some(export_symbol) = &aliased.export_symbol {
            candidates.push(Arc::clone(export_symbol));
        }
        for candidate in candidates {
            let decl = candidate
                .value_declaration
                .clone()
                .or_else(|| candidate.declarations.first().cloned());
            if let Some(decl) = decl {
                let doc = self.declaration_jsdoc_text(&decl);
                if !doc.is_empty() {
                    return doc;
                }
            }
        }
        String::new()
    }

    #[allow(dead_code)]
    fn declaration_has_typedef_tag(&self, decl: &Arc<Node>) -> bool {
        let file = match self.get_source_file_of_node(decl) {
            Some(f) => f,
            None => return false,
        };
        for jsdoc in decl.jsdoc(&file) {
            if jsdoc.kind != SyntaxKind::JSDoc {
                continue;
            }
            if let tsox_frontend::ast::NodeData::JSDoc(d) = &jsdoc.data
                && let Some(tags) = &d.tags
            {
                for tag in &tags.nodes {
                    if matches!(
                        tag.kind,
                        SyntaxKind::JSDocTypedefTag | SyntaxKind::JSDocCallbackTag
                    ) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
