#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;
use crate::ls::mig::m5v_3::m5v3_ext::M5v3NodeExt;
use crate::mig::m5u_conv::M5uScript;

use crate::ls::find_all_references::{EntryKind, ReferenceEntry};
use crate::ls::language_service::LanguageService;
use crate::lsp::lsproto_lsp::{DocumentUri, Range, TextEdit};

pub type M5vDiagnosticMessage = tsox_core::diagnostics::Message;

fn m5v5_m5u_converters() -> crate::mig::m5u_conv::M5uConverters { ::tsox_core::fntrace::enter("m5v5_m5u_converters"); 
    crate::mig::m5u_conv::new_converters(
        crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    )
}

pub struct MappedRenameEdit {
    pub uri: DocumentUri,
    pub edit: TextEdit,
}

struct RenameEditKey {
    uri: DocumentUri,
    text_range: Range,
}

pub struct M5vRenameInfo {
    pub can_rename: bool,
    pub display_name: String,
    pub full_display_name: String,
    pub kind: String,
    pub kind_modifiers: String,
    pub trigger_span: Range,
    pub localized_error_message: String,
}

impl M5vRenameInfo {
    fn error(message: String) -> Self { ::tsox_core::fntrace::enter("error"); 
        M5vRenameInfo {
            can_rename: false,
            display_name: String::new(),
            full_display_name: String::new(),
            kind: String::new(),
            kind_modifiers: String::new(),
            trigger_span: Range::default(),
            localized_error_message: message,
        }
    }
}

pub fn deduplicate_rename_edits(
    mapped_edits: &[MappedRenameEdit],
) -> (Option<HashMap<DocumentUri, Vec<TextEdit>>>, bool) { ::tsox_core::fntrace::enter("deduplicate_rename_edits"); 
    let mut edit_texts: HashMap<RenameEditKeyComparable, String> = HashMap::new();
    let mut unique_edits: Vec<&MappedRenameEdit> = Vec::with_capacity(mapped_edits.len());
    for mapped_edit in mapped_edits {
        let key = RenameEditKeyComparable {
            uri: mapped_edit.uri.0.clone(),
            start_line: mapped_edit.edit.range.start.line,
            start_character: mapped_edit.edit.range.start.character,
            end_line: mapped_edit.edit.range.end.line,
            end_character: mapped_edit.edit.range.end.character,
        };
        if let Some(existing_text) = edit_texts.get(&key) {
            if *existing_text != mapped_edit.edit.new_text {
                return (None, false);
            }
            continue;
        }
        edit_texts.insert(key, mapped_edit.edit.new_text.clone());
        unique_edits.push(mapped_edit);
    }
    let mut changes: HashMap<DocumentUri, Vec<TextEdit>> = HashMap::new();
    for mapped_edit in unique_edits {
        changes
            .entry(DocumentUri(mapped_edit.uri.0.clone()))
            .or_default()
            .push(TextEdit {
                range: mapped_edit.edit.range.clone(),
                new_text: mapped_edit.edit.new_text.clone(),
            });
    }
    (Some(changes), true)
}

#[derive(PartialEq, Eq, Hash)]
struct RenameEditKeyComparable {
    uri: String,
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
}

fn source_file_of_node(
    program: &tsox_compile::compiler::Program,
    node: &Arc<Node>,
) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("source_file_of_node"); 
    let file_node = ast::get_source_file_of_node(node)?;
    program
        .source_files()
        .iter()
        .find(|f| Arc::ptr_eq(&f.node, &file_node))
        .cloned()
}

pub fn is_defined_in_library_file(
    program: &tsox_compile::compiler::Program,
    declaration: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_defined_in_library_file"); 
    let decl_source_file = source_file_of_node(program, declaration)
        .unwrap_or_else(|| panic!("is_defined_in_library_file: declaration not contained in a SourceFile"));
    program.is_source_file_default_library(&decl_source_file.file_name)
        && tsox_core::tspath::is_declaration_file_name(&decl_source_file.file_name)
}

pub fn would_rename_in_other_node_modules(
    original_file: &Arc<SourceFile>,
    symbol: &Arc<Symbol>,
    ch: &mut Checker,
    preferences: &crate::ls::lsutil::UserPreferences,
) -> Option<M5vDiagnosticMessage> { ::tsox_core::fntrace::enter("would_rename_in_other_node_modules"); 
    let mut sym = Arc::clone(symbol);
    if !preferences.use_aliases_for_rename.is_true_or_unknown()
        && symbol.flags.contains(ast::SymbolFlags::Alias)
    {
        let import_specifier = symbol
            .declarations
            .iter()
            .find(|d| ast::is_import_specifier(d));
        if let Some(import_specifier) = import_specifier {
            if import_specifier.as_import_specifier().property_name.is_none() {
                sym = ch.get_aliased_symbol(&sym);
            }
        }
    }

    let declarations = sym.declarations.clone();
    if declarations.is_empty() {
        return None;
    }

    let original_package =
        tsox_tsoptions::module::parse_node_module_from_path(&original_file.file_name, false);
    if original_package.is_empty() {
        for declaration in &declarations {
            let Some(decl_file) = ch.get_source_file_of_node(declaration) else {
                continue;
            };
            if crate::ls::mig::m5x_2::is_inside_node_modules(&decl_file.file_name) {
                return Some(
                    tsox_core::diagnostics::YOU_CANNOT_RENAME_ELEMENTS_THAT_ARE_DEFINED_IN_A_NODE_MODULES_FOLDER,
                );
            }
        }
        return None;
    }

    for declaration in &declarations {
        let Some(decl_file) = ch.get_source_file_of_node(declaration) else {
            continue;
        };
        let decl_package =
            tsox_tsoptions::module::parse_node_module_from_path(&decl_file.file_name, false);
        if !decl_package.is_empty() && decl_package != original_package {
            return Some(
                tsox_core::diagnostics::YOU_CANNOT_RENAME_ELEMENTS_THAT_ARE_DEFINED_IN_ANOTHER_NODE_MODULES_FOLDER,
            );
        }
    }
    None
}

pub fn client_supports_will_rename_files(
    ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
) -> bool { ::tsox_core::fntrace::enter("client_supports_will_rename_files"); 
    crate::mig::m5m::get_client_capabilities(ctx)
        .raw
        .pointer("/workspace/fileOperations/willRename")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub fn client_supports_document_changes(
    ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
) -> bool { ::tsox_core::fntrace::enter("client_supports_document_changes"); 
    crate::mig::m5m::get_client_capabilities(ctx)
        .raw
        .pointer("/workspace/workspaceEdit/documentChanges")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub fn client_supports_rename_resource_operations(
    ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
) -> bool { ::tsox_core::fntrace::enter("client_supports_rename_resource_operations"); 
    crate::mig::m5m::get_client_capabilities(ctx)
        .raw
        .pointer("/workspace/workspaceEdit/resourceOperations")
        .and_then(|v| v.as_array())
        .map_or(false, |ops| {
            ops.iter().any(|op| op.as_str() == Some("rename"))
        })
}

pub fn get_quote_from_preference(quote_preference: crate::ls::lsutil::QuotePreference) -> String { ::tsox_core::fntrace::enter("get_quote_from_preference"); 
    if quote_preference == crate::ls::lsutil::QuotePreference::Single {
        return "'".to_string();
    }
    "\"".to_string()
}

pub fn get_rename_info_error(
    ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
    message: M5vDiagnosticMessage,
) -> M5vRenameInfo { ::tsox_core::fntrace::enter("get_rename_info_error"); 
    M5vRenameInfo::error(message.localize(&locale_from_ctx(ctx), &[]))
}

fn locale_from_ctx(_ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext) -> tsox_core::locale::Locale { ::tsox_core::fntrace::enter("locale_from_ctx"); 
    tsox_core::locale::Locale(String::new())
}

pub fn get_rename_info_success(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    display_name: &str,
    converters: &crate::mig::m5u_conv::M5uConverters,
) -> M5vRenameInfo { ::tsox_core::fntrace::enter("get_rename_info_success"); 
    let mut start = astnav::get_start_of_node(node, source_file, false);
    let mut end = node.end();
    if ast::is_string_literal_like(node) {
        start += 1;
        end -= 1;
    }
    let script = crate::mig::m5u_conv::SourceFileScriptView {
        file: Arc::clone(source_file),
    };
    let (trigger_span, fidelity) =
        converters.to_lsp_range(&script, tsox_core::core::text::TextRange::new(start, end));
    if fidelity != crate::mig::m5u_conv::SPANMAP_FIDELITY_EXACT {
        return M5vRenameInfo {
            can_rename: false,
            ..M5vRenameInfo::error(String::new())
        };
    }
    M5vRenameInfo {
        can_rename: true,
        display_name: display_name.to_string(),
        full_display_name: String::new(),
        kind: String::new(),
        kind_modifiers: String::new(),
        trigger_span,
        localized_error_message: String::new(),
    }
}

impl LanguageService {
    pub fn m5v_get_rename_info_for_module(
        &self,
        ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
        new_name: &str,
        specifier: &Arc<Node>,
        source_file: &Arc<SourceFile>,
        module_symbol: &Arc<Symbol>,
    ) -> (M5vRenameInfo, bool) { ::tsox_core::fntrace::enter("m5v_get_rename_info_for_module"); 
        if !tsox_core::tspath::is_external_module_name_relative(&specifier.text()) {
            return (
                get_rename_info_error(
                    ctx,
                    tsox_core::diagnostics::YOU_CANNOT_RENAME_A_MODULE_VIA_A_GLOBAL_IMPORT,
                ),
                true,
            );
        }
        if !client_supports_document_changes(ctx) || !client_supports_rename_resource_operations(ctx)
        {
            return (
                get_rename_info_error(ctx, tsox_core::diagnostics::FILE_RENAME_IS_NOT_SUPPORTED_BY_THE_EDITOR),
                true,
            );
        }
        (M5vRenameInfo::error(String::new()), false)
    }

    pub fn m5v_get_new_file_name_for_module_rename(
        &self,
        old_path: &str,
        specifier_text: &str,
        new_name: &str,
    ) -> String { ::tsox_core::fntrace::enter("m5v_get_new_file_name_for_module_rename"); 
        let mut new_path = tsox_core::tspath::combine_paths(
            &tsox_core::tspath::get_directory_path(old_path),
            &[new_name],
        );
        let ignore_case = !self.host.use_case_sensitive_file_names();
        let old_ext = if tsox_core::tspath::is_declaration_file_name(old_path) {
            tsox_core::tspath::get_declaration_file_extension(old_path)
        } else {
            tsox_core::tspath::get_any_extension_from_path(old_path, &[], ignore_case)
        };
        if !tsox_core::tspath::has_extension(&new_path) {
            new_path = format!("{}{}", new_path, old_ext);
        } else if tsox_core::tspath::get_any_extension_from_path(&new_path, &[], ignore_case)
            == tsox_core::tspath::get_any_extension_from_path(specifier_text, &[], ignore_case)
        {
            new_path = tsox_core::tspath::mig::m3i::change_any_extension(&new_path, &old_ext, &[], ignore_case);
        }
        new_path
    }

    pub fn m5v_rename_edit_range(&self, entry: &mut ReferenceEntry) -> (Range, bool) { ::tsox_core::fntrace::enter("m5v_rename_edit_range"); 
        self.resolve_entry(entry);
        let Some(node) = entry.node.clone() else {
            let program = self.get_program();
            let Some(source_file) = program
                .source_files()
                .iter()
                .find(|f| f.file_name == entry.file_name)
                .cloned()
            else {
                return (Range::default(), false);
            };
            let (location, fidelity) =
                self.source_file_range_to_lsp_location(&source_file, entry.text_range.unwrap());
            return (
                location.range,
                matches!(fidelity, crate::ls::mig::m5s::SpanFidelity::Exact),
            );
        };
        let program = self.get_program();
        let Some(source_file) = crate::ls::mig::m5t_3::source_file_of_node(&program, &node) else {
            return (self.get_range_of_entry(entry), true);
        };
        let script = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(&source_file),
        };
        if script.span_map().is_none() {
            return (self.get_range_of_entry(entry), true);
        }
        let (lsp_range, fidelity) = m5v5_m5u_converters()
            .to_lsp_range(&script, entry.text_range.unwrap());
        (
            lsp_range,
            crate::ls::mig::m5u_3::fidelity_is_exact(&fidelity),
        )
    }

    pub fn m5v_rename_blocked_reason(
        &self,
        source_file: &Arc<SourceFile>,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        ch: &mut Checker,
        program: &tsox_compile::compiler::Program,
    ) -> Option<M5vDiagnosticMessage> { ::tsox_core::fntrace::enter("m5v_rename_blocked_reason"); 
        for declaration in &symbol.declarations {
            if is_defined_in_library_file(program, &declaration) {
                return Some(
                    tsox_core::diagnostics::YOU_CANNOT_RENAME_ELEMENTS_THAT_ARE_DEFINED_IN_THE_STANDARD_TYPESCRIPT_LIBRARY,
                );
            }
        }

        if ast::is_identifier(node)
            && node.text() == "default"
            && symbol.parent().is_some()
            && symbol
                .parent()
                .unwrap()
                .flags
                .contains(ast::SymbolFlags::MODULE)
        {
            return Some(tsox_core::diagnostics::YOU_CANNOT_RENAME_THIS_ELEMENT);
        }

        would_rename_in_other_node_modules(
            source_file,
            symbol,
            ch,
            &self.user_preferences(),
        )
    }
}
