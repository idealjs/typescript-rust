#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_jsx_namespace(&self) -> Option<Arc<crate::ast::Symbol>> {
        let file_id = self.current_file_id as usize;
        if let Some(cached) = self.jsx_implicit_namespace.get(&file_id)
            && let Some(ns) = cached
        {
            return Some(Arc::clone(ns));
        }
        self.globals.get(JsxNames::JSX).cloned()
    }

    pub fn get_jsx_type(&self, name: &str) -> Option<Arc<crate::ast::Symbol>> {
        let ns = self.get_jsx_namespace()?;
        ns.members
            .get(name)
            .or_else(|| ns.exports.get(name))
            .cloned()
            .or_else(|| self.ambient_namespace_local(&ns, name))
    }

    pub fn get_jsx_element_type(&self) -> Option<Arc<crate::ast::Symbol>> {
        self.get_jsx_type(JsxNames::ELEMENT)
    }

    pub fn get_jsx_intrinsic_elements(&self) -> Option<Arc<crate::ast::Symbol>> {
        self.get_jsx_type(JsxNames::INTRINSIC_ELEMENTS)
    }

    pub fn is_jsx_enabled(&self) -> bool {
        self.compiler_options.jsx != crate::core::compiler_options::JsxEmit::None
    }

    pub fn check_jsx_preconditions(&mut self, error_node: &Arc<Node>) {
        if !self.is_jsx_enabled() {
            self.grammar_error_on_node(error_node, &CANNOT_USE_JSX_UNLESS_THE_JSX_FLAG_IS_PROVIDED);
        }

        if self.no_implicit_any && self.get_jsx_namespace().is_none() {
            self.grammar_error_on_node(
                error_node,
                &JSX_ELEMENT_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_THE_GLOBAL_TYPE_JSX_ELEMENT_DOES_NOT_EXIST,
            );
        }
    }

    pub(crate) fn ensure_jsx_implicit_container(&mut self, error_node: &Arc<Node>) {
        use crate::core::compiler_options::JsxEmit;
        let file_id = self.current_file_id as usize;
        if self.jsx_implicit_namespace.contains_key(&file_id) {
            return;
        }
        let resolved: Option<std::sync::Arc<crate::ast::Symbol>> = match self.compiler_options.jsx {
            JsxEmit::ReactJSX | JsxEmit::ReactJSXDev => {
                let source = if self.compiler_options.jsx_import_source.is_empty() {
                    "react"
                } else {
                    self.compiler_options.jsx_import_source.as_str()
                };
                let module_ref = if self.compiler_options.jsx == JsxEmit::ReactJSXDev {
                    format!("{source}/jsx-dev-runtime")
                } else {
                    format!("{source}/jsx-runtime")
                };
                match self
                    .resolve_module_file_symbol(&module_ref)
                    .or_else(|| self.resolve_jsx_runtime_by_path(&module_ref))
                {
                    Some(module_sym) => {
                        let ns = module_sym
                            .exports
                            .get(JsxNames::JSX)
                            .or_else(|| module_sym.members.get(JsxNames::JSX))
                            .cloned();
                        ns
                    }
                    None => {
                        let mut span = error_node.loc;
                        let mut node: &Arc<Node> = error_node;
                        while let Some(parent) = node.parent.as_ref() {
                            match parent.kind {
                                crate::ast::SyntaxKind::JsxElement
                                | crate::ast::SyntaxKind::JsxSelfClosingElement
                                | crate::ast::SyntaxKind::JsxFragment => {
                                    span = parent.loc;
                                    node = parent;
                                }
                                _ => break,
                            }
                        }
                        self.pending_jsx_2875 = Some((span, module_ref));
                        None
                    }
                }
            }

            _ => None,
        };
        self.jsx_implicit_namespace.insert(file_id, resolved);
    }

    pub(crate) fn resolve_jsx_runtime_by_path(
        &self,
        module_ref: &str,
    ) -> Option<Arc<crate::ast::Symbol>> {
        let containing = self
            .current_file
            .as_ref()
            .map(|f| f.file_name.clone())
            .unwrap_or_default();

        let mode = crate::compiler::implied_node_format_of_file(&containing, &|p| {
            self.program.read_file(p)
        });
        let path = self
            .program
            .resolve_external_module_path(module_ref, &containing, mode)?;
        let sf = self.program.get_source_file(&path)?;
        self.program.symbol_map().symbol_of(&sf.node).cloned()
    }

    pub fn check_jsx_intrinsic_element(&mut self, opening: &Arc<Node>) {
        let tag_name = match jsx_tag_name(opening) {
            Some(t) => t,
            None => return,
        };
        let tag_text = tag_name.text().to_string();

        let intrinsic_elements = match self.get_jsx_intrinsic_elements() {
            Some(sym) => sym,
            None => {
                if self.no_implicit_any {
                    self.grammar_error_on_node_with_args(
                        opening,
                        &JSX_ELEMENT_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_NO_INTERFACE_JSX_0_EXISTS,
                        &[JsxNames::INTRINSIC_ELEMENTS.to_string()],
                    );
                }
                return;
            }
        };

        let member = intrinsic_elements
            .members
            .get(&tag_text)
            .or_else(|| intrinsic_elements.exports.get(&tag_text));

        if member.is_none() {
            let has_index_signature = intrinsic_elements.declarations.iter().any(|d| {
                matches!(&d.data, crate::ast::NodeData::InterfaceDeclaration(id) if id
                    .members
                    .iter()
                    .any(|m| m.kind == SyntaxKind::IndexSignature))
            });
            if intrinsic_elements.members.is_empty()
                && intrinsic_elements.exports.is_empty()
                && !has_index_signature
            {
                if self.no_implicit_any {
                    self.grammar_error_on_node_with_args(
                        opening,
                        &JSX_ELEMENT_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_NO_INTERFACE_JSX_0_EXISTS,
                        &[JsxNames::INTRINSIC_ELEMENTS.to_string()],
                    );
                }
            }
        }
    }

    pub fn check_jsx_component(&mut self, opening: &Arc<Node>) {
        let tag_name = match jsx_tag_name(opening) {
            Some(t) => t,
            None => return,
        };

        self.check_expression(&tag_name);

        let tag_type = self.get_type_of_node(&tag_name);

        if tag_type
            .flags
            .contains(crate::checker::types::TypeFlags::Any)
        {
            return;
        }

        let has_call_sigs = !self
            .get_signatures_of_type(&tag_type, crate::checker::SignatureKind::Call)
            .is_empty();
        let has_construct_sigs = !self
            .get_signatures_of_type(&tag_type, crate::checker::SignatureKind::Construct)
            .is_empty();

        if !has_call_sigs && !has_construct_sigs {
            let text = tag_name.text().to_string();
            self.grammar_error_on_node_with_args(
                &tag_name,
                &JSX_ELEMENT_TYPE_0_DOES_NOT_HAVE_ANY_CONSTRUCT_OR_CALL_SIGNATURES,
                &[text],
            );
        }
    }

    pub(crate) fn jsx_factory_namespace_in_scope(&self, name: &str) -> bool {
        use crate::ast::SymbolFlags;
        let symbol_map = self.program.symbol_map();
        let value = |sym: &std::sync::Arc<crate::ast::Symbol>| {
            if sym.flags.intersects(SymbolFlags::Alias) {
                match self.follow_alias(sym) {
                    Some(t) if std::sync::Arc::ptr_eq(&t, sym) => true,
                    Some(t) => t.flags.intersects(SymbolFlags::VALUE),
                    None => true,
                }
            } else {
                sym.flags.intersects(SymbolFlags::VALUE)
            }
        };
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id)
                && let Some(sym) = locals.get(name)
                && value(sym)
            {
                return true;
            }
            if let Some(cs) = symbol_map.symbols.get(&container_id)
                && (!cs.flags.intersects(SymbolFlags::Class)
                    || cs.flags.intersects(SymbolFlags::Function))
                && let Some(sym) = cs.members.get(name)
                && value(sym)
            {
                return true;
            }
            if let Some(cs) = symbol_map.symbols.get(&container_id)
                && cs.flags.intersects(SymbolFlags::MODULE)
                && !cs.flags.intersects(SymbolFlags::Class)
                && let Some(sym) = cs.exports.get(name)
                && value(sym)
            {
                return true;
            }
        }
        self.globals
            .get(name)
            .is_some_and(|g| g.flags.intersects(SymbolFlags::VALUE))
    }

    pub(crate) fn local_jsx_pragma_factory(&self, pragma: &str) -> Option<String> {
        let file = self.current_file.as_ref()?;
        let ranges = crate::scanner::get_leading_comment_ranges(&file.text, 0);
        let mut result = None;
        for r in ranges {
            if r.kind != crate::scanner::CommentRangeKind::MultiLine {
                continue;
            }
            let comment = &file.text[r.pos..r.end];
            let comment = comment.strip_suffix("*/").unwrap_or(comment);
            for line in comment.split('\n') {
                let Some(at) = line.find('@') else {
                    continue;
                };
                let after = &line[at + 1..];
                let name_end = after.find(char::is_whitespace).unwrap_or(after.len());
                let name = &after[..name_end];
                if !name.eq_ignore_ascii_case(pragma) {
                    continue;
                }
                let args = after[name_end..].trim_start();
                let arg_end = args.find(char::is_whitespace).unwrap_or(args.len());
                if arg_end > 0 {
                    result = Some(args[..arg_end].to_string());
                }
            }
        }
        result
    }
}
