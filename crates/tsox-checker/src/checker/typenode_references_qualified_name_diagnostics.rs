#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    // Go tryGetQualifiedNameAsValue：限定名按值含义逐段取属性类型，
    // 命中即说明该名字作为值存在（类型位解析失败时用于 typeof 建议）
    pub(crate) fn try_get_qualified_name_as_value(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let id = crate::checker::checker::base_identifier_of(node);
        let mut symbol = self.resolve_identifier_with_meaning(&id, SymbolFlags::VALUE)?;
        let mut n = id;
        while let Some(parent) = n.parent()
            && parent.kind == SyntaxKind::QualifiedName
        {
            let t = self.get_type_of_symbol(&symbol);
            let right = match &parent.data {
                tsox_frontend::ast::NodeData::QualifiedName(q) => q.right.text().to_string(),
                _ => break,
            };
            symbol = self.get_property_of_type(&t, &right)?;
            n = parent;
        }
        Some(symbol)
    }

    pub(crate) fn report_qualified_name_resolution_failure(
        &mut self,
        type_name: &Arc<Node>,
        segment: &Arc<Node>,
        ns_path: String,
        member: String,
    ) {
        let attributed_file = self
            .get_source_file_of_node(type_name)
            .or_else(|| self.current_file.clone());
        let reportable = type_name.kind == SyntaxKind::QualifiedName;
        if reportable
            && self.ts2304_reporting_allowed_for(type_name)
            && attributed_file
                .as_ref()
                .is_some_and(|f| !f.file_name.starts_with("bundled://"))
        {
            let file = attributed_file;
            if ns_path.is_empty() {
                // Go getSuggestedSymbolForNonexistentSymbol：带拼写建议的
                // 2503 变体（TS2833）
                let name_text = segment.text().to_string();
                if let Some(sugg) = self.find_name_suggestion(&name_text, SymbolFlags::NAMESPACE)
                {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        segment.loc,
                        tsox_core::diagnostics::messages_generated::
                            CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1,
                        vec![name_text, sugg],
                    ));
                } else {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        segment.loc,
                        tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0,
                        vec![name_text],
                    ));
                }
            } else {
                let leftmost = crate::checker::checker::base_identifier_of(type_name);
                let left_hit = self
                    .resolve_identifier(&leftmost)
                    .map(|s| self.resolve_alias_base(s));
                let left_non_namespace = left_hit
                    .as_ref()
                    .is_some_and(|b| !b.flags.intersects(SymbolFlags::NAMESPACE));
                if left_non_namespace {
                    let name_text = leftmost.text().to_string();
                    if let Some(sugg) =
                        self.find_name_suggestion(&name_text, SymbolFlags::NAMESPACE)
                    {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    file,
                                    leftmost.loc,
                                    tsox_core::diagnostics::messages_generated::
                                        CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1,
                                    vec![name_text.clone(), sugg],
                                ));
                    } else if left_hit
                        .as_ref()
                        .is_some_and(|b| b.flags.intersects(SymbolFlags::TYPE))
                    {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    file,
                                    leftmost.loc,
                                    tsox_core::diagnostics::messages_generated::
                                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                    vec![name_text],
                                ));
                    } else {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            leftmost.loc,
                            tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0,
                            vec![name_text],
                        ));
                    }
                } else {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                segment.loc,
                                tsox_core::diagnostics::messages_generated::
                                    NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                vec![ns_path.clone(), member.clone()],
                            ));
                }
            }
        }
    }
}
