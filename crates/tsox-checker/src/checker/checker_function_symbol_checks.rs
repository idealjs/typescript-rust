#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, Symbol, SyntaxKind};

use crate::checker::checker::*;
use crate::checker::grammarchecks_is_this_parameter_2::is_optional_declaration;

const FLAGS_TO_CHECK: ModifierFlags = ModifierFlags::Export
    .union(ModifierFlags::Ambient)
    .union(ModifierFlags::Private)
    .union(ModifierFlags::Protected)
    .union(ModifierFlags::Abstract);

impl Checker {
    fn emit_node_error(&mut self, node: &Arc<Node>, message: tsox_core::diagnostics::Message) {
        let file = self
            .get_source_file_of_node(node)
            .or_else(|| self.current_file.clone());
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,
            node.loc,
            message,
            Vec::new(),
        ));
    }

    fn emit_node_error_with_args(
        &mut self,
        node: &Arc<Node>,
        message: tsox_core::diagnostics::Message,
        args: Vec<String>,
    ) {
        let file = self
            .get_source_file_of_node(node)
            .or_else(|| self.current_file.clone());
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,
            node.loc,
            message,
            args,
        ));
    }

    pub fn check_function_or_constructor_symbol(&mut self, symbol: &Arc<Symbol>) {
        let already = self
            .value_symbol_links
            .get_or_default(symbol)
            .function_or_constructor_checked;
        if already {
            return;
        }
        self.value_symbol_links
            .get_or_default(symbol)
            .function_or_constructor_checked = true;
        self.check_function_or_constructor_symbol_worker(symbol);
    }

    fn effective_declaration_flags(&self, node: &Arc<Node>) -> ModifierFlags {
        let mut flags = node.syntactic_modifier_flags();
        let parent_is_classish = node.parent().is_some_and(|p| {
            matches!(
                p.kind,
                SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
            )
        });
        if !parent_is_classish && node.flags.contains(NodeFlags::Ambient) {
            flags |= ModifierFlags::Ambient;
        }
        flags & FLAGS_TO_CHECK
    }

    /// declare namespace/module 祖先（ambient 语境，成员无自身标志）
    fn has_ambient_ancestor(node: &Arc<Node>) -> bool {
        let mut cur = node.parent();
        while let Some(n) = cur {
            match n.kind {
                SyntaxKind::ModuleDeclaration => {
                    if n.has_syntactic_modifier(ModifierFlags::Ambient)
                        || n.flags.contains(NodeFlags::Ambient)
                    {
                        return true;
                    }
                }
                SyntaxKind::SourceFile => return false,
                _ => {}
            }
            cur = n.parent();
        }
        false
    }

    fn is_function_like_declaration_kind(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::Constructor
        )
    }

    fn check_function_or_constructor_symbol_worker(&mut self, symbol: &Arc<Symbol>) {
        let declarations = symbol.declarations.clone();
        let is_constructor = symbol.flags.contains(SymbolFlags::Constructor);
        let mut some_node_flags = ModifierFlags::empty();
        let mut all_node_flags = FLAGS_TO_CHECK;
        let mut some_have_question_token = false;
        let mut all_have_question_token = true;
        let mut has_overloads = false;
        let mut body_declaration: Option<Arc<Node>> = None;
        let mut previous_declaration: Option<Arc<Node>> = None;
        let mut last_seen_non_ambient: Option<Arc<Node>> = None;
        let mut duplicate_function_declaration = false;
        let mut multiple_constructor_implementation = false;
        let mut function_declarations: Vec<Arc<Node>> = Vec::new();

        for node in &declarations {
            let in_ambient_context = node.flags.contains(NodeFlags::Ambient)
                || node.has_syntactic_modifier(ModifierFlags::Ambient)
                || self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.is_declaration_file)
                || node.parent().is_some_and(|cls| {
                    (cls.flags.contains(NodeFlags::Ambient)
                        || cls.has_syntactic_modifier(ModifierFlags::Ambient))
                        && matches!(
                            cls.kind,
                            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                        )
                })
                || Self::has_ambient_ancestor(node);
            let in_ambient_or_interface = in_ambient_context
                || node.parent().is_some_and(|p| {
                    matches!(
                        p.kind,
                        SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeLiteral
                    )
                });
            if Self::is_function_like_declaration_kind(node.kind) {
                let current_flags = self.effective_declaration_flags(node);
                some_node_flags |= current_flags;
                all_node_flags &= current_flags;
                let optional = is_optional_declaration(node);
                some_have_question_token = some_have_question_token || optional;
                all_have_question_token = all_have_question_token && optional;
                let body_is_present = self.node_body_is_present(node);
                if body_is_present && body_declaration.is_some() {
                    if is_constructor {
                        multiple_constructor_implementation = true;
                    } else {
                        duplicate_function_declaration = true;
                    }
                } else if let Some(prev) = &previous_declaration {
                    let prev_same_parent = prev
                        .parent()
                        .zip(node.parent())
                        .is_some_and(|(a, b)| Arc::ptr_eq(&a, &b));
                    if prev_same_parent && prev.loc.end() != node.loc.pos() {
                        let prev = Arc::clone(prev);
                        self.report_implementation_expected_error_for_symbol(&prev);
                    }
                }
                if body_is_present {
                    if body_declaration.is_none() {
                        body_declaration = Some(Arc::clone(node));
                    }
                } else {
                    has_overloads = true;
                }
                if !in_ambient_or_interface {
                    last_seen_non_ambient = Some(Arc::clone(node));
                    previous_declaration = Some(Arc::clone(node));
                } else {
                    previous_declaration = None;
                }
                function_declarations.push(Arc::clone(node));
            }
        }

        if multiple_constructor_implementation {
            for d in &function_declarations {
                self.emit_node_error(
                    d,
                    tsox_core::diagnostics::messages_generated::
                        MULTIPLE_CONSTRUCTOR_IMPLEMENTATIONS_ARE_NOT_ALLOWED,
                );
            }
        }
        if duplicate_function_declaration {
            for d in &function_declarations {
                let name = d.name().unwrap_or(d);
                self.emit_node_error(
                    &name,
                    tsox_core::diagnostics::messages_generated::DUPLICATE_FUNCTION_IMPLEMENTATION,
                );
            }
        }
        if let Some(last) = &last_seen_non_ambient
            && self.node_body_is_present(last) == false
            && !last.has_syntactic_modifier(ModifierFlags::Abstract)
            && !is_optional_declaration(last)
        {
            self.report_implementation_expected_error_for_symbol(last);
        }
        if has_overloads {
            self.check_flag_agreement_between_overloads(
                &declarations,
                body_declaration.as_ref(),
                some_node_flags,
                all_node_flags,
            );
            self.check_question_token_agreement_between_overloads(
                &declarations,
                body_declaration.as_ref(),
                some_have_question_token,
                all_have_question_token,
            );
        }
    }

    /// Go NodeIsPresent：零宽节点（parse 恢复产物）视为缺失 body
    fn node_body_is_present(&self, node: &Arc<Node>) -> bool {
        let body = match &node.data {
            NodeData::FunctionDeclaration(d) => d.body.as_ref(),
            NodeData::MethodDeclaration(d) => d.body.as_ref(),
            NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
            _ => return false,
        };
        match body {
            Some(b) => b.loc.pos() < b.loc.end(),
            None => false,
        }
    }

    fn canonical_overload<'a>(
        overloads: &[&'a Arc<Node>],
        implementation: Option<&'a Arc<Node>>,
    ) -> &'a Arc<Node> {
        let shares_container = implementation
            .zip(overloads.first())
            .and_then(|(impl_, first)| {
                impl_
                    .parent()
                    .zip(first.parent())
                    .map(|(a, b)| Arc::ptr_eq(&a, &b))
            })
            .unwrap_or(false);
        if shares_container {
            implementation.unwrap()
        } else {
            overloads[0]
        }
    }

    fn check_flag_agreement_between_overloads(
        &mut self,
        declarations: &[Arc<Node>],
        implementation: Option<&Arc<Node>>,
        some_flags: ModifierFlags,
        all_flags: ModifierFlags,
    ) {
        let some_but_not_all = some_flags ^ all_flags;
        if some_but_not_all.is_empty() {
            return;
        }
        let overloads: Vec<&Arc<Node>> = declarations
            .iter()
            .filter(|d| Self::is_function_like_declaration_kind(d.kind))
            .collect();
        if overloads.is_empty() {
            return;
        }
        let canonical = Self::canonical_overload(&overloads, implementation);
        let canonical_flags = self.effective_declaration_flags(canonical);
        let mut groups: Vec<(Arc<Node>, Vec<&Arc<Node>>)> = Vec::new();
        for o in &overloads {
            let file = self
                .get_source_file_of_node(o)
                .map(|f| Arc::clone(&f.node))
                .unwrap_or_else(|| Arc::clone(*o));
            match groups.iter_mut().find(|(k, _)| Arc::ptr_eq(&k, &file)) {
                Some((_, g)) => g.push(o),
                None => groups.push((file, vec![*o])),
            }
        }
        for (_, overloads_in_file) in &groups {
            let canonical_in_file = Self::canonical_overload(overloads_in_file, implementation);
            let canonical_flags_in_file = self.effective_declaration_flags(canonical_in_file);
            for o in overloads_in_file {
                let deviation = self.effective_declaration_flags(o) ^ canonical_flags;
                let deviation_in_file = self.effective_declaration_flags(o) ^ canonical_flags_in_file;
                let name = o.name().unwrap_or(o);
                if deviation_in_file.contains(ModifierFlags::Export) {
                    self.emit_node_error(
                        &name,
                        tsox_core::diagnostics::messages_generated::
                            OVERLOAD_SIGNATURES_MUST_ALL_BE_EXPORTED_OR_NON_EXPORTED,
                    );
                } else if deviation_in_file.contains(ModifierFlags::Ambient) {
                    self.emit_node_error(
                        &name,
                        tsox_core::diagnostics::messages_generated::
                            OVERLOAD_SIGNATURES_MUST_ALL_BE_AMBIENT_OR_NON_AMBIENT,
                    );
                } else if deviation
                    .intersects(ModifierFlags::Private | ModifierFlags::Protected)
                {
                    self.emit_node_error(
                        &name,
                        tsox_core::diagnostics::messages_generated::
                            OVERLOAD_SIGNATURES_MUST_ALL_BE_PUBLIC_PRIVATE_OR_PROTECTED,
                    );
                } else if deviation.contains(ModifierFlags::Abstract) {
                    self.emit_node_error(
                        &name,
                        tsox_core::diagnostics::messages_generated::
                            OVERLOAD_SIGNATURES_MUST_ALL_BE_ABSTRACT_OR_NON_ABSTRACT,
                    );
                }
            }
        }
    }

    fn check_question_token_agreement_between_overloads(
        &mut self,
        declarations: &[Arc<Node>],
        implementation: Option<&Arc<Node>>,
        some_have_question_token: bool,
        all_have_question_token: bool,
    ) {
        if some_have_question_token == all_have_question_token {
            return;
        }
        let overloads: Vec<&Arc<Node>> = declarations
            .iter()
            .filter(|d| Self::is_function_like_declaration_kind(d.kind))
            .collect();
        if overloads.is_empty() {
            return;
        }
        let canonical = Self::canonical_overload(&overloads, implementation);
        let canonical_has = is_optional_declaration(canonical);
        for o in &overloads {
            if is_optional_declaration(o) != canonical_has
                && let Some(name) = o.name()
            {
                self.emit_node_error(
                    &name,
                    tsox_core::diagnostics::messages_generated::
                        OVERLOAD_SIGNATURES_MUST_ALL_BE_OPTIONAL_OR_REQUIRED,
                );
            }
        }
    }

    fn report_implementation_expected_error_for_symbol(&mut self, node: &Arc<Node>) {
        use tsox_core::diagnostics::messages_generated as msg;
        let Some(name) = node.name() else {
            return;
        };
        if Self::node_is_missing(&name) {
            return;
        }
        let Some(parent) = node.parent() else {
            return;
        };
        let mut children: Vec<Arc<Node>> = Vec::new();
        tsox_frontend::ast::node_data_generated::for_each_child(&parent, |child| {
            children.push(Arc::clone(child));
            false
        });
        let pos_idx = children.iter().position(|c| Arc::ptr_eq(c, node));
        let Some(i) = pos_idx else { return };
        let subsequent = children.get(i + 1);
        if let Some(sub) = subsequent {
            if sub.loc.pos() == node.loc.end() && sub.kind == node.kind {
                if Self::declaration_names_match_for_merge(node, sub) {
                    let report = matches!(node.kind, SyntaxKind::MethodDeclaration)
                        && node.has_syntactic_modifier(ModifierFlags::Static)
                            != sub.has_syntactic_modifier(ModifierFlags::Static);
                    if report {
                        let error_node = sub.name().unwrap_or(sub);
                        let diagnostic = if node.has_syntactic_modifier(ModifierFlags::Static) {
                            msg::FUNCTION_OVERLOAD_MUST_BE_STATIC
                        } else {
                            msg::FUNCTION_OVERLOAD_MUST_NOT_BE_STATIC
                        };
                        self.emit_node_error(&error_node, diagnostic);
                    }
                    return;
                }
                if self.node_body_is_present(sub) {
                    let decl_name = self.node_text(&name);
                    let sub_name: Arc<Node> =
                        sub.name().map(Arc::clone).unwrap_or_else(|| Arc::clone(sub));
                    let already = self
                        .diagnostics
                        .get_all()
                        .iter()
                        .any(|d| d.code == 2389 && d.loc == sub_name.loc);
                    if already {
                        return;
                    }
                    self.emit_node_error_with_args(
                        &sub_name,
                        msg::FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![decl_name],
                    );
                    return;
                }
            }
        }
        let error_node = node.name().unwrap_or(node);
        if node.kind == SyntaxKind::Constructor {
            self.emit_node_error(&error_node, msg::CONSTRUCTOR_IMPLEMENTATION_IS_MISSING);
        } else if node.has_syntactic_modifier(ModifierFlags::Abstract) {
            self.emit_node_error(
                &error_node,
                msg::ALL_DECLARATIONS_OF_AN_ABSTRACT_METHOD_MUST_BE_CONSECUTIVE,
            );
        } else {
            self.emit_node_error(
                &error_node,
                msg::FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
            );
        }
    }

    fn node_is_missing(node: &Arc<Node>) -> bool {
        node.loc.pos() >= node.loc.end()
    }

    /// Go reportImplementationExpectedError 同名判定：计算名按类型同一性
    fn declaration_names_match_for_merge(a: &Arc<Node>, b: &Arc<Node>) -> bool {
        let (na, nb) = match (a.name(), b.name()) {
            (Some(x), Some(y)) => (x, y),
            _ => return false,
        };
        if na.kind == SyntaxKind::ComputedPropertyName && nb.kind == SyntaxKind::ComputedPropertyName
        {
            let same_well_known = crate::binder::symbols_binder_4::well_known_symbol_member_name(&na)
                .zip(crate::binder::symbols_binder_4::well_known_symbol_member_name(&nb))
                .is_some_and(|(x, y)| x == y);
            return same_well_known || na.text() == nb.text();
        }
        na.text() == nb.text()
    }
}
