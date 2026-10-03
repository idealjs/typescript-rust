#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_core::core;
use tsox_core::tspath;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5q2b_2::get_line_of_position_m5q2b;
use super::m5q2b_3::lsproto;

use super::m5q2::{
    ImportStatementCompletionInfo, MemberCompletionEntry, get_jsdoc_param_name_with_initializer,
    generate_jsdoc_param_tags_for_destructuring, SORT_TEXT_GLOBALS_OR_KEYWORDS,
};
use super::m5q_3::{
    create_snippet_printer, escape_snippet_text, get_completions_symbol_kind, get_dot_accessor,
    client_supports_item_snippet, create_snippet_tab_stop_body, SnippetPrinter,
    SORT_TEXT_LOCATION_PRIORITY,
};
use super::m5r::{
    get_dot_accessor as get_dot_accessor_m5r, get_word_length_and_start,
    is_module_specifier_missing_or_empty, get_potentially_invalid_import_specifier,
    could_be_type_only_import_specifier, is_non_contextual_keyword, str_ptr_to,
    COMPLETION_SOURCE_SWITCH_CASES,
};
use super::m5q2::get_filter_text;

pub type SortText = String;


impl crate::ls::language_service::LanguageService {
    pub fn filter_content_mapped_auto_imports(
        &self,
        program: &tsox_compile::compiler::Program,
        file: &Arc<SourceFile>,
        mut list: Option<&mut lsproto::CompletionList>,
    ) { ::tsox_core::fntrace::enter("filter_content_mapped_auto_imports"); 
        let Some(list) = list.as_deref_mut() else {
            return;
        };
        for item in list.items.iter_mut() {
            let Some(data) = item.data.as_ref() else {
                continue;
            };
            let Some(auto_import) = data.auto_import.as_ref() else {
                continue;
            };
            let fix = crate::ls::autoimport_fix::Fix {
                auto_import_fix: auto_import_fix_from_lsproto(auto_import),
                module_specifier_kind: Default::default(),
                is_re_export: false,
                module_file_name: String::new(),
                type_only_alias_declaration: None,
            };
            let (edits, description) = fix.edits(
                file,
                program.options(),
                &self.format_options(),
                self.user_preferences(),
            );
            item.additional_text_edits = Some(edits);
            item.detail = Some(description);
        }
    }

    pub fn get_entry_for_member_completion(
        &self,
        type_checker: &mut Checker,
        symbol: &Arc<Symbol>,
        name: &str,
        location: &Arc<Node>,
        position: usize,
        context_token: Option<&Arc<Node>>,
        file: &Arc<SourceFile>,
    ) -> Option<Option<MemberCompletionEntry>> { ::tsox_core::fntrace::enter("get_entry_for_member_completion"); 
        let class_like_declaration = ast::find_ancestor_kind(location, SyntaxKind::ClassDeclaration)
            .or_else(|| ast::find_ancestor_kind(location, SyntaxKind::ClassExpression));
        let Some(class_like_declaration) = class_like_declaration else {
            return Some(None);
        };

        let program = self.get_program();
        // ImportAdder::new 需 Arc<Checker>（progress_notes_r59A.md 已登记的 clone_arc 缺口），暂以 None 缺省
        let mut import_adder: Option<crate::ls::autoimport_import_adder::ImportAdder> = None;

        let present_modifiers = self.get_present_member_modifiers(context_token, file, position);
        let abstract_present = present_modifiers.modifiers.intersects(ast::ModifierFlags::Abstract)
            && class_like_declaration
                .syntactic_modifier_flags()
                .intersects(ast::ModifierFlags::Abstract);
        let is_snippet = client_supports_item_snippet(&crate::mig::m5m::ResolvedClientCapabilitiesContext {
            capabilities: None,
        });
        let factory = NodeFactory::new();
        let mut body = new_block_m5q2b(&factory, factory.new_node_list(Vec::new()), true);
        if is_snippet {
            let mut emit_context = tsox_emit::printer::EmitContext::default();
            body = create_snippet_tab_stop_body(&factory, &mut emit_context)?;
        }

        let fixer = self.new_missing_member_fixer(&program, type_checker);
        let nodes = fixer.create_member_from_symbol(symbol, &class_like_declaration, file, ast::ModifierFlags::empty().bits());
        let mut additional_text_edits: Vec<lsproto::TextEdit> = Vec::new();
        if let Some(adder) = import_adder.as_mut() {
            if crate::ls::autoimport_import_adder::ImportAdderTrait::has_fixes(adder) {
                additional_text_edits.extend(
                    crate::ls::autoimport_import_adder::ImportAdderTrait::edits(adder),
                );
            }
        }
        if let Some(erase_range) = &present_modifiers.erase_range {
            additional_text_edits.push(lsproto::TextEdit {
                range: erase_range.clone(),
                new_text: String::new(),
            });
        }

        let mut modifiers = ast::ModifierFlags::empty();
        let mut completion_nodes: Vec<Arc<Node>> = Vec::with_capacity(nodes.len());
        for node in nodes.iter() {
            if completion_nodes.is_empty() {
                modifiers = node.syntactic_modifier_flags();
                if abstract_present {
                    modifiers |= ast::ModifierFlags::Abstract;
                }
                if ast::is_class_element(node)
                    && type_checker.get_member_override_modifier_status(
                        &class_like_declaration,
                        node,
                        Some(symbol),
                    ) == tsox_checker::checker::MemberOverrideStatus::NeedsOverride
                {
                    modifiers |= ast::ModifierFlags::Override;
                }
            }
            completion_nodes.push(Arc::clone(node));
        }

        if completion_nodes.is_empty() {
            return Some(Some(MemberCompletionEntry {
                insert_text: name.to_string(),
                filter_text: name.to_string(),
                is_snippet,
                additional_text_edits,
            }));
        }

        let mut allowed_modifiers = modifiers | ast::ModifierFlags::Override | ast::ModifierFlags::Public;
        if symbol.flags.intersects(ast::SymbolFlags::Method) {
            allowed_modifiers |= ast::ModifierFlags::Async;
        } else {
            allowed_modifiers |= ast::ModifierFlags::Ambient | ast::ModifierFlags::Readonly;
        }

        let allowed_and_present = present_modifiers.modifiers & allowed_modifiers;
        if !allowed_modifiers.contains(present_modifiers.modifiers) {
            return Some(None);
        }

        if modifiers.intersects(ast::ModifierFlags::Protected)
            && allowed_and_present.intersects(ast::ModifierFlags::Public)
        {
            modifiers.remove(ast::ModifierFlags::Protected);
        }

        if !allowed_and_present.is_empty()
            && !allowed_and_present.intersects(ast::ModifierFlags::Public)
        {
            modifiers.remove(ast::ModifierFlags::Public);
        }

        modifiers |= allowed_and_present;
        let new_line = self.format_options().new_line_character.clone();
        let mut snippet_printer = create_snippet_printer(
            tsox_frontend::format::mig::m4o_2::PrinterOptions {
                remove_comments: true,
                new_line: core::mig::m3j_3::get_new_line_kind(&new_line),
                omit_trailing_semicolon: false,
                no_emit_helpers: false,
                target: program.options().get_emit_script_target(),
                source_map: false,
                inline_source_map: false,
                inline_sources: false,
                omit_brace_source_map_positions: false,
                only_print_jsdoc_style: false,
                never_ascii_escape: false,
                preserve_source_newlines: false,
                terminate_unterminated_literals: false,
            },
            None,
        );

        let mut decorated_node: Option<&Arc<Node>> = None;
        if !present_modifiers.decorators.is_empty() {
            let last_node_index = completion_nodes.len() - 1;
            if ast::can_have_decorators(&completion_nodes[last_node_index]) {
                decorated_node = Some(&completion_nodes[last_node_index]);
            }
        }

        let mut texts: Vec<String> = Vec::with_capacity(completion_nodes.len());
        for node in completion_nodes.iter() {
            let decorators = if decorated_node.is_some_and(|dn| Arc::ptr_eq(dn, node)) {
                Some(present_modifiers.decorators.as_slice())
            } else {
                None
            };
            let mut mod_nodes: Vec<Arc<Node>> = decorators
                .map(|ds| {
                    ds.iter()
                        .map(|d| tsox_frontend::ast::deep_clone_node(d))
                        .collect()
                })
                .unwrap_or_default();
            mod_nodes.extend(tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags(
                modifiers,
                |kind| factory.new_modifier(kind),
            ));
            let modifier_array = if mod_nodes.is_empty() {
                None
            } else {
                Some(Arc::new(factory.new_modifier_list(mod_nodes)))
            };
            let node = ast::mig::m3g_3::replace_modifiers(&factory, node, modifier_array);
            let text = snippet_printer.print_and_format_node_with_settings(
                &node,
                file,
                &crate::ls::change_tracker_impl::get_format_code_settings_for_writing(
                    self.format_options().clone(),
                    file,
                ),
            );
            texts.push(text);
        }

        let insert_text = texts.join(&new_line);
        if insert_text.is_empty() {
            return Some(None);
        }

        Some(Some(MemberCompletionEntry {
            insert_text,
            filter_text: name.to_string(),
            is_snippet,
            additional_text_edits,
        }))
    }

    pub fn get_present_member_modifiers(
        &self,
        context_token: Option<&Arc<Node>>,
        file: &Arc<SourceFile>,
        position: usize,
    ) -> PresentMemberModifiers { ::tsox_core::fntrace::enter("get_present_member_modifiers"); 
        let Some(context_token) = context_token else {
            return PresentMemberModifiers::default();
        };
        if get_line_of_position_m5q2b(file, position) > get_line_of_position_m5q2b(file, context_token.end()) {
            return PresentMemberModifiers::default();
        }

        let mut modifiers = ast::ModifierFlags::empty();
        let mut decorators: Vec<Arc<Node>> = Vec::new();
        let mut range_pos = position;
        let mut range_end = position;

        if let Some(parent) = context_token.parent() {
            if ast::is_property_declaration(&parent) {
                let context_modifier_kind = super::m5r::modifier_like_kind(Some(context_token));
                if context_modifier_kind == SyntaxKind::Unknown {
                    return PresentMemberModifiers::default();
                }

                let modifier_nodes = parent.modifier_nodes();
                if !modifier_nodes.is_empty() {
                    modifiers |= ast::modifiers_to_flags(modifier_nodes) & ast::ModifierFlags::Modifier;
                    for modifier in modifier_nodes.iter() {
                        if ast::is_decorator(modifier) {
                            decorators.push(Arc::clone(modifier));
                        }
                        range_pos = range_pos.min(scanner::mig::x5a::get_token_pos_of_node(modifier, file, false));
                    }
                }

                let context_modifier_flag = ast::modifier_to_flag(context_modifier_kind);
                if !modifiers.intersects(context_modifier_flag) {
                    modifiers |= context_modifier_flag;
                    range_pos = range_pos.min(astnav::get_start_of_node(context_token, file, false));
                }

                let parent_name = parent.name();
                if !parent_name.is_some_and(|n| Arc::ptr_eq(n, context_token)) {
                    if let Some(name_node) = parent_name {
                        range_end = astnav::get_start_of_node(name_node, file, false);
                    }
                }
            }
        }

        let mut erase_range: Option<lsproto::Range> = None;
        if range_pos < range_end {
            let script = crate::ls::language_service::ScriptInfo {
                file_name: file.file_name.clone(),
                text: file.text.clone(),
            };
            erase_range = Some(self.create_lsp_range_from_bounds(range_pos, range_end, &script));
        }

        PresentMemberModifiers {
            modifiers,
            decorators,
            erase_range,
        }
    }

}

fn new_block_m5q2b(
    _factory: &NodeFactory,
    statements: tsox_frontend::ast::node_node_list::NodeList,
    multi_line: bool,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_block_m5q2b"); 
    Arc::new(Node::new(
        SyntaxKind::Block,
        tsox_frontend::ast::node_data_generated::NodeData::Block(
            tsox_frontend::ast::node_data_generated::BlockData {
                statements: Arc::new(statements),
                multi_line,
            },
        ),
    ))
}

#[derive(Default, Clone)]
pub struct PresentMemberModifiers {
    pub modifiers: ast::ModifierFlags,
    pub decorators: Vec<Arc<Node>>,
    pub erase_range: Option<lsproto::Range>,
}

fn auto_import_fix_from_lsproto(fix: &lsproto::AutoImportFix) -> crate::ls::autoimport::AutoImportFix { ::tsox_core::fntrace::enter("auto_import_fix_from_lsproto"); 
    crate::ls::autoimport::AutoImportFix {
        kind: match fix.kind {
            0 => crate::ls::autoimport::AutoImportFixKind::UseNamespace,
            1 => crate::ls::autoimport::AutoImportFixKind::JsdocTypeImport,
            2 => crate::ls::autoimport::AutoImportFixKind::AddToExisting,
            4 => crate::ls::autoimport::AutoImportFixKind::PromoteTypeOnly,
            _ => crate::ls::autoimport::AutoImportFixKind::AddNew,
        },
        import_kind: match fix.import_kind {
            0 => crate::ls::autoimport::ImportKind::Named,
            1 => crate::ls::autoimport::ImportKind::Default,
            2 => crate::ls::autoimport::ImportKind::Namespace,
            _ => crate::ls::autoimport::ImportKind::CommonJS,
        },
        module_specifier: fix.module_specifier.clone(),
        name: fix.name.clone(),
        use_require: fix.use_require,
        add_as_type_only: match fix.add_as_type_only {
            1 => crate::ls::autoimport::AddAsTypeOnly::Allowed,
            2 => crate::ls::autoimport::AddAsTypeOnly::Required,
            _ => crate::ls::autoimport::AddAsTypeOnly::NotAllowed,
        },
        import_index: fix.import_index,
        usage_position: fix.usage_position.clone(),
        namespace_prefix: fix.namespace_prefix.clone(),
    }
}
