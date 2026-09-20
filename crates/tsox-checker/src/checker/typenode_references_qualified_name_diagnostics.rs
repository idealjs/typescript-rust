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
                // Go resolveQualifiedName 失败分支 canSuggestTypeof：限定名按值
                // 含义可解析（如枚举成员链上的方法）时优先 TS2749 typeof 建议
                let mut containing = Arc::clone(segment);
                while let Some(p) = containing.parent() {
                    if p.kind == SyntaxKind::QualifiedName {
                        containing = p;
                    } else {
                        break;
                    }
                }
                if containing.kind == SyntaxKind::QualifiedName
                    && self.try_get_qualified_name_as_value(&containing).is_some()
                {
                    let name_text =
                        crate::checker::checker::qualified_name_text(&containing);
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        containing.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                        vec![name_text.clone(), name_text],
                    ));
                    return;
                }
                // Go resolveQualifiedName 失败分支（meaning=Namespace）：左段限定名
                // 以 Type 含义存在时（Err 携带空 member、ns_path 长于最左段），
                // 报 2713 于外层右段
                if member.is_empty() && ns_path != segment.text() {
                    let qn1 = segment.parent();
                    let qn2 = qn1.as_ref().and_then(|q| q.parent());
                    let pair = qn1.zip(qn2).and_then(|(q1, q2)| {
                        let (r1, r2) = match (&q1.data, &q2.data) {
                            (
                                tsox_frontend::ast::NodeData::QualifiedName(a),
                                tsox_frontend::ast::NodeData::QualifiedName(b),
                            ) => (a.right.clone(), b.right.clone()),
                            _ => return None,
                        };
                        Some((r1, r2))
                    });
                    if let Some((member_node, outer_right)) = pair {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            outer_right.loc,
                            tsox_core::diagnostics::messages_generated::
                                CANNOT_ACCESS_0_1_BECAUSE_0_IS_A_TYPE_BUT_NOT_A_NAMESPACE_DID_YOU_MEAN_TO_RETRIEVE_THE_TYPE_OF_THE_PROPERTY_1_IN_0_WITH_0_1,
                            vec![
                                member_node.text().to_string(),
                                outer_right.text().to_string(),
                            ],
                        ));
                        return;
                    }
                }
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
                        // Go checkAndReportErrorForUsingTypeAsNamespace：类型符号
                        // 声明类型上存在该属性时报 2713（给出 [] 取法建议），否则 2702
                        let prop_name = leftmost.parent().and_then(|p| match &p.data {
                            tsox_frontend::ast::NodeData::QualifiedName(q) => {
                                Some(q.right.text().to_string())
                            }
                            _ => None,
                        });
                        let prop_hit = prop_name.as_deref().and_then(|pn| {
                            let sym = left_hit.as_ref().unwrap();
                            let decl = if sym.flags.contains(SymbolFlags::TypeAlias) {
                                self.resolve_type_alias_reference(sym, None)
                            } else {
                                self.get_declared_type_of_symbol(sym)
                            };
                            self.get_property_of_type(&decl, pn)
                        });
                        if prop_hit.is_some() {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        tsox_core::diagnostics::messages_generated::
                                            CANNOT_ACCESS_0_1_BECAUSE_0_IS_A_TYPE_BUT_NOT_A_NAMESPACE_DID_YOU_MEAN_TO_RETRIEVE_THE_TYPE_OF_THE_PROPERTY_1_IN_0_WITH_0_1,
                                        vec![name_text.clone(), prop_name.clone().unwrap_or_default()],
                                    ));
                        } else {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        tsox_core::diagnostics::messages_generated::
                                            X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                        vec![name_text],
                                    ));
                        }
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
