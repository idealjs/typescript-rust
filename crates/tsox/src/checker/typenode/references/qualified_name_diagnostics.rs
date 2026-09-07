#![allow(unused_imports)]

use super::*;

impl Checker {
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
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    segment.loc,
                    crate::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0,
                    vec![segment.text().to_string()],
                ));
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
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    leftmost.loc,
                                    crate::diagnostics::messages_generated::
                                        CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1,
                                    vec![name_text.clone(), sugg],
                                ));
                    } else if left_hit
                        .as_ref()
                        .is_some_and(|b| b.flags.intersects(SymbolFlags::TYPE))
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    leftmost.loc,
                                    crate::diagnostics::messages_generated::
                                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                    vec![name_text],
                                ));
                    } else {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            leftmost.loc,
                            crate::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0,
                            vec![name_text],
                        ));
                    }
                } else {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                segment.loc,
                                crate::diagnostics::messages_generated::
                                    NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                vec![ns_path, member],
                            ));
                }
            }
        }
    }
}
