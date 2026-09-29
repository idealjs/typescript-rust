use super::m3n::*;
use super::m3n_5::create_diagnostic_for_node;
use super::m3n_7::DeclarationTransformer;
use super::m4e_2::{can_reuse_modifier_nodes, get_binding_name_visible, is_always_type, mask_modifier_flags};
use crate::mig::m4e::{R37K1DataExt, R38K1NodeExt, R38K1NodeVisitorExt, R39K01DataExt};
use tsox_core::core::core::filter;
use tsox_frontend::ast::mig::m3g_3::{
    is_var_await_using, is_var_using, is_variable_declaration_initialized_to_require,
};
use tsox_frontend::ast::mig::m3g::is_modifier;
use tsox_frontend::ast::mig::x6a::is_implicitly_exported_jsdoc_declaration;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as diag_msgs;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::get_name_of_declaration;
use tsox_frontend::ast::mig::m3b::parameters;
use tsox_frontend::ast::mig::m3g_2::is_parse_tree_node;
use tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{get_combined_modifier_flags, get_source_file_of_node, ModifierList, SourceFile};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::CommentRange;
use tsox_frontend::scanner::mig::w5::get_text_of_jsdoc_comment;
use tsox_frontend::scanner::{
    get_leading_comment_ranges, get_trailing_comment_ranges, skip_trivia_ex, SkipTriviaOptions,
};
use tsox_core::diagnostics::Message;

impl DeclarationTransformer {
    pub fn should_strip_internal(&self, node: Option<&Arc<Node>>) -> bool {
        match node {
            Some(node) => {
                self.state.strip_internal
                    && self.is_internal_declaration(node, self.state.current_source_file.as_deref())
            }
            None => false,
        }
    }

    pub fn is_internal_declaration(
        &self,
        node: &Arc<Node>,
        source_file: Option<&SourceFile>,
    ) -> bool {
        let Some(source_file) = source_file else {
            return false;
        };
        let parse_tree_node = self.emit_context().most_original(node);
        if !is_parse_tree_node(&parse_tree_node) {
            return false;
        }
        if parse_tree_node.kind == SyntaxKind::Parameter {
            let Some(parent) = parse_tree_node.parent() else {
                return false;
            };
            let params = parameters(&parent);
            let param_idx = params.iter().position(|p| Arc::ptr_eq(p, &parse_tree_node));
            let previous_sibling = param_idx
                .and_then(|idx| idx.checked_sub(1))
                .and_then(|idx| params.get(idx))
                .cloned();

            let text = &source_file.text;
            let mut comment_ranges: Vec<CommentRange> = Vec::new();

            match previous_sibling {
                Some(previous_sibling) => {
                    let trailing_pos = skip_trivia_ex(
                        text,
                        previous_sibling.end() + 1,
                        &SkipTriviaOptions {
                            stop_at_comments: true,
                            in_jsdoc: false,
                            stop_after_line_break: false,
                        },
                        None,
                    );
                    comment_ranges.extend(get_trailing_comment_ranges(text, trailing_pos));
                    comment_ranges.extend(get_leading_comment_ranges(text, node.pos()));
                }
                None => {
                    let trailing_pos = skip_trivia_ex(
                        text,
                        node.pos(),
                        &SkipTriviaOptions {
                            stop_at_comments: true,
                            in_jsdoc: false,
                            stop_after_line_break: false,
                        },
                        None,
                    );
                    comment_ranges.extend(get_trailing_comment_ranges(text, trailing_pos));
                }
            }

            if let Some(last) = comment_ranges.last() {
                return super::m3n_7::has_internal_annotation(*last, source_file);
            }
            return false;
        }

        for comment_range in self.get_leading_comment_ranges_of_node(&parse_tree_node, source_file)
        {
            if super::m3n_7::has_internal_annotation(comment_range, source_file) {
                return true;
            }
        }
        false
    }

    pub fn get_leading_comment_ranges_of_node<'a>(
        &'a self,
        node: &'a Arc<Node>,
        source_file: &'a SourceFile,
    ) -> Vec<CommentRange> {
        if node.kind == SyntaxKind::JsxText {
            return Vec::new();
        }
        get_leading_comment_ranges(&source_file.text, node.pos())
    }

    pub fn rewrite_module_specifier(
        &mut self,
        parent: &Arc<Node>,
        input: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let input = input?;
        self.result_has_external_module_indicator = self.result_has_external_module_indicator
            || (parent.kind != SyntaxKind::ModuleDeclaration && parent.kind != SyntaxKind::ImportType);
        Some(Arc::clone(input))
    }

    pub fn preserve_js_doc(&self, updated: &Arc<Node>, original: &Arc<Node>) {
        self.emit_context().assign_comment_range(updated, original);
    }

    pub fn preserve_partial_js_doc(&mut self, updated: &Arc<Node>, original: &Arc<Node>) {
        if !original.flags.contains(NodeFlags::Reparsed) {
            return;
        }
        let Some(source_file) = self.state.current_source_file.clone() else {
            return;
        };
        let Some(jsdoc) = source_file.eager_jsdoc(original).first().map(Arc::clone) else {
            return;
        };
        let description = jsdoc_comment_text(&jsdoc);
        if description.is_empty() {
            return;
        }
        let comment = "*\n * ".to_string() + &description.replace('\n', "\n * ") + "\n ";
        self.emit_context().add_synthetic_leading_comment(
            updated,
            SyntaxKind::MultiLineCommentTrivia,
            &comment,
            true,
        );
    }

    pub fn remove_all_comments(&self, node: &Arc<Node>) {
        self.emit_context().add_emit_flags(node, EmitFlags::NO_COMMENTS);
    }

    pub fn check_entity_name_visibility(
        &mut self,
        entity_name: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) {
        let visibility_result = self
            .resolver
            .is_entity_name_visible(entity_name, enclosing_declaration);
        self.tracker.handle_symbol_accessibility_error(visibility_result);
    }

    pub fn check_name(&mut self, node: &Arc<Node>) {
        let old_diag = self.state.get_symbol_accessibility_diagnostic.take();
        if !self.suppress_new_diagnostic_contexts {
            self.state.get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node_name(node));
        }
        self.state.error_name_node = node.name().cloned();
        let entity_name = node.name().and_then(|n| n.expression()).expect("dynamic name");
        let enclosing = self.enclosing_declaration.clone();
        self.check_entity_name_visibility(&entity_name, enclosing.as_ref());
        if !self.suppress_new_diagnostic_contexts {
            self.state.get_symbol_accessibility_diagnostic = old_diag;
        }
        self.state.error_name_node = None;
    }

    pub fn ensure_modifiers(&self, node: &Arc<Node>) -> Option<Arc<ModifierList>> {
        let current_flags = self
            .emit_context()
            .parse_node(node)
            .map(|pn| get_combined_modifier_flags(&pn))
            .unwrap_or_default()
            & ModifierFlags::all();
        let new_flags = self.ensure_modifier_flags(node);
        if current_flags == new_flags {
            let mods = node.modifiers();
            let Some(mods) = mods else {
                return None;
            };
            if can_reuse_modifier_nodes(&mods.nodes) {
                return Some(
                    self.factory()
                        .new_modifier_list(filter(&mods.nodes, |n| is_modifier(n))),
                );
            }
        }
        let result =
            create_modifiers_from_modifier_flags(new_flags, |k| self.factory().new_modifier(k));
        if result.is_empty() {
            return None;
        }
        Some(self.factory().new_modifier_list(result))
    }

    pub fn transform_variable_statement(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let declaration_list = input.as_variable_statement().declaration_list.clone();
        let declarations = &declaration_list.as_variable_declaration_list().declarations.nodes;
        let mut visible = false;
        for decl in declarations {
            visible = get_binding_name_visible(&self.resolver, decl);
            if visible {
                break;
            }
        }
        if !visible {
            return None;
        }

        let mut input_nodes: Vec<Arc<Node>> = declarations.clone();
        let mut extra_imports: Vec<Arc<Node>> = Vec::new();
        if self
            .state
            .current_source_file
            .as_ref()
            .map(|f| f.common_js_module_indicator.is_some())
            .unwrap_or(false)
        {
            let mut normal_declarations: Vec<Arc<Node>> = Vec::new();
            let mut imports: Vec<Arc<Node>> = Vec::new();
            for n in &input_nodes {
                if is_variable_declaration_initialized_to_require(n) {
                    imports.push(n.clone());
                } else {
                    normal_declarations.push(n.clone());
                }
            }
            input_nodes = normal_declarations;
            let (visited, _) = self.visitor().visit_slice(imports);
            extra_imports = visited;
        }

        let (nodes, _) = self.visitor().visit_slice(input_nodes);
        if nodes.is_empty() {
            if !extra_imports.is_empty() {
                return Some(self.factory().new_syntax_list(extra_imports));
            }
            return None;
        }
        let node_list = self.factory().new_node_list(nodes);

        let modifiers = self.ensure_modifiers(input);

        let decl_list = if is_var_using(&declaration_list) || is_var_await_using(&declaration_list)
        {
            let mut dl = self
                .factory()
                .new_variable_declaration_list(&node_list, NodeFlags::Const);
            self.emit_context().set_original(&dl, &declaration_list);
            self.emit_context()
                .set_comment_range(&dl, declaration_list.loc());
            if let Some(dl_node) = Arc::get_mut(&mut dl) {
                dl_node.loc = declaration_list.loc();
            }
            dl
        } else {
            self.factory().update_variable_declaration_list(
                &declaration_list,
                &node_list,
                declaration_list.flags,
            )
        };
        let res = self
            .factory()
            .update_variable_statement(input, modifiers, decl_list);
        if !extra_imports.is_empty() {
            extra_imports.push(res);
            return Some(self.factory().new_syntax_list(extra_imports));
        }
        Some(res)
    }

    pub fn ensure_modifier_flags(&self, node: &Arc<Node>) -> ModifierFlags {
        let mut mask = ModifierFlags::all()
            - ModifierFlags::Public
            - ModifierFlags::Async
            - ModifierFlags::Override;
        let mut additions = ModifierFlags::empty();
        if self.needs_declare && !is_always_type(node) {
            additions = ModifierFlags::Ambient;
        }
        let parent_is_file = matches!(node.parent(), Some(p) if p.kind == SyntaxKind::SourceFile);
        if !parent_is_file {
            mask = mask - ModifierFlags::Ambient;
            additions = ModifierFlags::empty();
        }
        if is_implicitly_exported_jsdoc_declaration(node) {
            additions |= ModifierFlags::Export;
        }
        mask_modifier_flags(node, mask, additions)
    }

}

pub fn jsdoc_comment_text(jsdoc: &Arc<Node>) -> String {
    match &jsdoc.data {
        NodeData::JSDoc(data) => get_text_of_jsdoc_comment(Some(data.comment.as_ref())),
        _ => String::new(),
    }
}

pub fn has_any_binding_initializers(binding_pattern: &Arc<Node>) -> bool {
    let elements = match &binding_pattern.data {
        NodeData::BindingPattern(data) => &data.elements.nodes,
        _ => return false,
    };
    for elem in elements {
        if !is_binding_element(elem) {
            continue;
        }
        if elem.initializer().is_some() {
            return true;
        }
        if let Some(name) = elem.name() {
            if is_binding_pattern(name) && has_any_binding_initializers(name) {
                return true;
            }
        }
    }
    false
}
