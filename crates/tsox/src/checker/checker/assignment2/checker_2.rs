#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_type_of_meta_property(&mut self, node: &Arc<Node>) -> Arc<Type> {
        use crate::core::compiler_options::ModuleKind;
        let (keyword_token, name) = match &node.data {
            crate::ast::NodeData::MetaProperty(d) => (d.keyword_token, &d.name),
            _ => return self.error_type(),
        };
        match keyword_token {
            SyntaxKind::NewKeyword => self.any_type(),
            SyntaxKind::ImportKeyword => {
                if name.text() == "defer" {
                    return self.error_type();
                }
                if name.text() == "meta" {
                    match self.compiler_options.module {
                        ModuleKind::Node16
                        | ModuleKind::Node18
                        | ModuleKind::Node20
                        | ModuleKind::NodeNext => {
                            let esm = self
                                .current_file
                                .as_ref()
                                .map(|f| {
                                    self.program_implied_format(&f.file_name) == ModuleKind::ESNext
                                })
                                .unwrap_or(false);
                            if !esm {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        THE_IMPORT_META_META_PROPERTY_IS_NOT_ALLOWED_IN_FILES_WHICH_WILL_BUILD_INTO_COMMONJS_OUTPUT,
                                    Vec::new(),
                                ));
                            }
                        }
                        m => {
                            let es2020_or_later = matches!(
                                m,
                                ModuleKind::ES2020
                                    | ModuleKind::ES2022
                                    | ModuleKind::ESNext
                                    | ModuleKind::Preserve
                                    | ModuleKind::Node16
                                    | ModuleKind::Node18
                                    | ModuleKind::Node20
                                    | ModuleKind::NodeNext
                            );
                            if !es2020_or_later && m != ModuleKind::System {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        THE_IMPORT_META_META_PROPERTY_IS_ONLY_ALLOWED_WHEN_THE_MODULE_OPTION_IS_ES2020_ES2022_ESNEXT_SYSTEM_NODE16_NODE18_NODE20_OR_NODENEXT,
                                    Vec::new(),
                                ));
                            }
                        }
                    }

                    if let Some(sym) = self.globals.get("ImportMeta").cloned() {
                        return self.resolve_interface_type(&sym, None);
                    }
                    self.any_type()
                } else {
                    self.error_type()
                }
            }
            _ => self.error_type(),
        }
    }

    pub(crate) fn program_implied_format(
        &self,
        file_name: &str,
    ) -> crate::core::compiler_options::ModuleKind {
        use crate::core::compiler_options::ModuleKind;
        match self.program.get_emit_module_format_of_file(file_name) {
            ModuleKind::None => crate::compiler::implied_node_format_of_file(file_name, &|p| {
                self.program.read_file(p)
            }),
            ModuleKind::ES2020 | ModuleKind::ESNext => ModuleKind::ESNext,
            _ => ModuleKind::CommonJS,
        }
    }

    pub(crate) fn syntactic_truthy_semantics(&mut self, node: &Arc<Node>) -> (bool, bool) {
        let mut n: Arc<Node> = Arc::clone(node);
        loop {
            match &n.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => n = Arc::clone(&p.expression),
                crate::ast::NodeData::NonNullExpression(p) => n = Arc::clone(&p.expression),
                crate::ast::NodeData::AsExpression(p) => n = Arc::clone(&p.expression),
                crate::ast::NodeData::TypeAssertion(p) => n = Arc::clone(&p.expression),
                _ => break,
            }
        }
        use SyntaxKind::*;
        match n.kind {
            NumericLiteral => {
                let t = n.text();
                if t == "0" || t == "1" {
                    (true, true)
                } else {
                    (true, false)
                }
            }
            ArrayLiteralExpression
            | ArrowFunction
            | BigIntLiteral
            | ClassExpression
            | FunctionExpression
            | JsxElement
            | JsxSelfClosingElement
            | ObjectLiteralExpression
            | RegularExpressionLiteral => (true, false),
            VoidExpression | NullKeyword => (false, true),
            NoSubstitutionTemplateLiteral | StringLiteral => {
                if !n.text().is_empty() {
                    (true, false)
                } else {
                    (false, true)
                }
            }
            ConditionalExpression => {
                if let crate::ast::NodeData::ConditionalExpression(d) = &n.data {
                    let (a1, n1) = self.syntactic_truthy_semantics(&d.when_true);
                    let (a2, n2) = self.syntactic_truthy_semantics(&d.when_false);
                    (a1 || a2, n1 || n2)
                } else {
                    (true, true)
                }
            }
            Identifier => {
                if let Some(sym) = self.resolve_identifier(&n)
                    && self.is_undefined_symbol(&sym)
                {
                    return (false, true);
                }
                (true, true)
            }
            _ => (true, true),
        }
    }

    pub(crate) fn check_truthiness_of_type(&mut self, node: &Arc<Node>) {
        let (always, never) = self.syntactic_truthy_semantics(node);
        let message = if always && !never {
            crate::diagnostics::messages_generated::THIS_KIND_OF_EXPRESSION_IS_ALWAYS_TRUTHY
        } else if never && !always {
            crate::diagnostics::messages_generated::THIS_KIND_OF_EXPRESSION_IS_ALWAYS_FALSY
        } else {
            return;
        };
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            node.loc,
            message,
            Vec::new(),
        ));
    }
}
