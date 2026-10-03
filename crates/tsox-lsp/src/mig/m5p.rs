#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::ls::language_service::LanguageService;
use crate::ls::find_all_references::{EntryKind, ReferenceEntry, SymbolAndEntriesData, SymbolEntryTransformOptions};
use crate::ls::autoimport_registry::Registry;
use crate::mig::m5n::LspError;
use crate::lsp::lsproto;
use tsox_frontend::ast::{self, Node, SourceFile};
use tsox_core::core::text::TextRange;
use tsox_core::core::tristate::Tristate;

use super::m5p_support::{CallHierarchyDeclarationResult, IncomingEntry, CallSite, CallSiteCollector};
use crate::mig::m5u_conv::M5uFidelityExt;

/// 本文件内引用的 lsproto 缺失类型：Go lsproto 包的最小等价移植，
/// 统一收口在本地 mod 以复用 `lsproto_lsp::` 路径写法；合并期归位到 crate::lsp::lsproto。
mod lsproto_lsp {
    pub use crate::lsp::lsproto_lsp::*;

    pub type SymbolTag = u32;

    #[derive(Debug, Clone, Default)]
    pub struct CallHierarchyItem {
        pub name: String,
        pub kind: crate::ls::types::SymbolKind,
        pub tags: Option<Vec<SymbolTag>>,
        pub detail: Option<String>,
        pub uri: DocumentUri,
        pub range: Range,
        pub selection_range: Range,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CallHierarchyIncomingCall {
        pub from: CallHierarchyItem,
        pub from_ranges: Vec<Range>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CallHierarchyOutgoingCall {
        pub to: CallHierarchyItem,
        pub from_ranges: Vec<Range>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CallHierarchyPrepareResponse {
        pub call_hierarchy_items: Option<Vec<CallHierarchyItem>>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CallHierarchyIncomingCallsResponse {
        pub call_hierarchy_incoming_calls: Option<Vec<CallHierarchyIncomingCall>>,
    }
}

pub type Checker = tsox_checker::checker::Checker;
pub type Program = tsox_compile::compiler::Program;

pub const FEATURE_CALL_HIERARCHY: u32 = 1 << 9;

fn get_conditions(
    options: &tsox_core::core::compiler_options::CompilerOptions,
    resolution_mode: tsox_core::core::compiler_options_kinds::ModuleKind,
) -> Vec<String> { ::tsox_core::fntrace::enter("get_conditions"); 
    let mut conditions = Vec::new();
    if resolution_mode == tsox_core::core::compiler_options_kinds::ModuleKind::ESNext {
        conditions.push("import".to_string());
    } else {
        conditions.push("require".to_string());
    }
    if !options.no_dts_resolution.is_true() {
        conditions.push("types".to_string());
    }
    if options.get_module_resolution_kind() != tsox_core::core::compiler_options_kinds::ModuleResolutionKind::Bundler {
        conditions.push("node".to_string());
    }
    for custom in &options.custom_conditions {
        conditions.push(custom.clone());
    }
    conditions
}

fn get_assigned_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_assigned_name"); 
    use tsox_frontend::ast::node_data_generated::NodeData;
    let parent = node.parent()?;
    match parent.kind {
        ast::SyntaxKind::PropertyAssignment => parent.name().map(Arc::clone),
        ast::SyntaxKind::BindingElement => parent.name().map(Arc::clone),
        ast::SyntaxKind::BinaryExpression => {
            let NodeData::BinaryExpression(d) = &parent.data else { return None };
            if !Arc::ptr_eq(node, &d.right) {
                return None;
            }
            match d.left.kind {
                ast::SyntaxKind::Identifier => Some(d.left.clone()),
                ast::SyntaxKind::PropertyAccessExpression => {
                    let NodeData::PropertyAccessExpression(p) = &d.left.data else { return None };
                    Some(p.name.clone())
                }
                ast::SyntaxKind::ElementAccessExpression => {
                    let NodeData::ElementAccessExpression(e) = &d.left.data else { return None };
                    let arg = tsox_frontend::ast::mig::m3g_3::skip_parentheses(&e.argument_expression);
                    if ast::is_string_or_numeric_literal_like(&arg) {
                        Some(arg)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        ast::SyntaxKind::VariableDeclaration => {
            let name = parent.name()?;
            if ast::is_identifier(name) {
                Some(Arc::clone(name))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn new_view(
    registry: Arc<Registry>,
    importing_file: Arc<SourceFile>,
    project_key: tsox_core::tspath::Path,
    program: Arc<Program>,
    preferences: tsox_tsoptions::modulespecifiers::UserPreferences,
) -> crate::ls::autoimport_view::View { ::tsox_core::fntrace::enter("new_view"); 
    let conditions = tsox_core::collections::set::Set::from_items(get_conditions(
        program.options(),
        program.get_default_resolution_mode_for_file(&importing_file),
    ));
    let should_use_uri_style_node_core_modules = crate::ls::lsutil_utilities::should_use_uri_style_node_core_modules(
        &importing_file,
        &program,
    );
    crate::ls::autoimport_view::View {
        registry,
        importing_file,
        program,
        preferences,
        project_key,
        allowed_endings: None,
        conditions,
        should_use_uri_style_node_core_modules,
        existing_imports: None,
        should_use_require_for_fixes: None,
    }
}

fn node_or_none(node: &Option<Arc<Node>>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("node_or_none"); 
    node.as_ref()
}

fn is_named_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_named_expression"); 
    if !ast::is_function_expression(node) && !ast::is_class_expression(node) {
        return false;
    }
    let name = node.name();
    name.map(|n| ast::is_identifier(n)).unwrap_or(false)
}

fn is_variable_like(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_variable_like"); 
    ast::is_property_declaration(node) || ast::is_variable_declaration(node)
}

pub fn is_assigned_expression(node: &Option<Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_assigned_expression"); 
    let Some(node) = node else { return false };
    if !(ast::is_function_expression(node) || ast::is_arrow_function(node) || ast::is_class_expression(node)) {
        return false;
    }
    if node.name().is_some() {
        return false;
    }
    let Some(parent) = node.parent() else { return false };
    if !is_variable_like(&parent) {
        return false;
    }
    if parent.initializer().map(|i| Arc::ptr_eq(i, node)) != Some(true) {
        return false;
    }
    let name = parent.name();
    if !name.map(|n| ast::is_identifier(n)).unwrap_or(false) {
        return false;
    }
    (ast::get_combined_node_flags(&parent) & ast::NodeFlags::Const) != ast::NodeFlags::empty()
        || ast::is_property_declaration(&parent)
}

pub fn is_possible_call_hierarchy_declaration(node: &Option<Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_possible_call_hierarchy_declaration"); 
    let Some(node) = node else { return false };
    ast::is_source_file(node)
        || ast::is_module_declaration(node)
        || ast::is_function_declaration(node)
        || ast::is_function_expression(node)
        || ast::is_class_declaration(node)
        || ast::is_class_expression(node)
        || ast::is_class_static_block_declaration(node)
        || ast::is_method_declaration(node)
        || ast::is_method_signature_declaration(node)
        || ast::is_get_accessor_declaration(node)
        || ast::is_set_accessor_declaration(node)
}

pub fn is_valid_call_hierarchy_declaration(node: &Option<Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_valid_call_hierarchy_declaration"); 
    let Some(node) = node else { return false };
    if ast::is_source_file(node) {
        return true;
    }
    if ast::is_module_declaration(node) {
        return node.name().map(|n| ast::is_identifier(n)).unwrap_or(false);
    }
    ast::is_function_declaration(node)
        || ast::is_class_declaration(node)
        || ast::is_class_static_block_declaration(node)
        || ast::is_method_declaration(node)
        || ast::is_method_signature_declaration(node)
        || ast::is_get_accessor_declaration(node)
        || ast::is_set_accessor_declaration(node)
        || is_named_expression(node)
        || is_assigned_expression(&Some(node.clone()))
}

pub fn get_call_hierarchy_declaration_reference_node(node: &Option<Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_call_hierarchy_declaration_reference_node"); 
    let node = node.as_ref()?;
    if ast::is_source_file(node) {
        return Some(node.clone());
    }
    if let Some(name) = node.name() {
        return Some(name.clone());
    }
    if is_assigned_expression(&Some(node.clone())) {
        return node.parent().and_then(|p| p.name().cloned());
    }
    if let Some(modifiers) = node.modifiers() {
        for m in &modifiers.list.nodes {
            if m.kind == ast::SyntaxKind::DefaultKeyword {
                return Some(m.clone());
            }
        }
    }
    None
}

pub fn get_symbol_of_call_hierarchy_declaration(c: &Checker, node: &Arc<Node>) -> Option<Arc<ast::Symbol>> { ::tsox_core::fntrace::enter("get_symbol_of_call_hierarchy_declaration"); 
    if ast::is_class_static_block_declaration(node) {
        return None;
    }
    let location = get_call_hierarchy_declaration_reference_node(&Some(node.clone()))?;
    c.get_symbol_at_location(&location)
}

pub fn get_call_hierarchy_item_name(program: &Program, node: &Arc<Node>) -> (String, usize, usize) { ::tsox_core::fntrace::enter("get_call_hierarchy_item_name"); 
    if ast::is_source_file(node) {
        let file_name = crate::ls::mig::m5u::node_as_source_file(node)
            .map(|f| f.file_name.clone())
            .unwrap_or_default();
        return (file_name, 0, 0);
    }

    if (ast::is_function_declaration(node) || ast::is_class_declaration(node)) && node.name().is_none() {
        if let Some(modifiers) = node.modifiers() {
            for m in &modifiers.list.nodes {
                if m.kind == ast::SyntaxKind::DefaultKeyword {
                    let source_file = ast::get_source_file_of_node(node).unwrap();
                    let start = tsox_frontend::scanner::skip_trivia(source_file.text(), m.pos());
                    return ("default".to_string(), start, m.end());
                }
            }
        }
    }

    if ast::is_class_static_block_declaration(node) {
        let source_file = ast::get_source_file_of_node(node).unwrap();
        let file = crate::ls::mig::m5u::node_as_source_file(&source_file).unwrap();
        let pos = tsox_frontend::scanner::skip_trivia(source_file.text(), move_range_past_modifiers(node).pos());
        let end = pos + 6;
        let mut c = program.get_type_checker_for_file(&file);
        let symbol = c.get_symbol_at_location(node.parent().as_ref().unwrap());
        let prefix = match &symbol {
            Some(s) => format!("{} ", c.symbol_to_string(s)),
            None => String::new(),
        };
        return (format!("{}static {{}}", prefix), pos, end);
    }

    let decl_name = if is_assigned_expression(&Some(node.clone())) {
        node.parent().and_then(|p| p.name().cloned())
    } else {
        ast::get_name_of_declaration(node)
    };

    if decl_name.is_none() || !ast::node_is_present(decl_name.as_ref()) {
        let source_file = ast::get_source_file_of_node(node).unwrap();
        if ast::is_function_declaration(node) || ast::is_function_expression(node) {
            let kw_pos = tsox_frontend::scanner::skip_trivia(source_file.text(), move_range_past_modifiers(node).pos());
            return ("(anonymous)".to_string(), kw_pos, kw_pos + 8);
        } else if ast::is_class_declaration(node) || ast::is_class_expression(node) {
            let kw_pos = tsox_frontend::scanner::skip_trivia(source_file.text(), move_range_past_modifiers(node).pos());
            return ("(anonymous)".to_string(), kw_pos, kw_pos + 5);
        }
    }

    let text = get_text_of_call_hierarchy_name(program, node, decl_name.as_ref().unwrap(), node);
    let source_file = ast::get_source_file_of_node(node).unwrap();
    let name_pos = tsox_frontend::scanner::skip_trivia(source_file.text(), decl_name.as_ref().unwrap().pos());
    (text, name_pos, decl_name.as_ref().unwrap().end())
}

pub fn get_text_of_call_hierarchy_name(program: &Program, source_node: &Arc<Node>, name: &Arc<Node>, print_node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_text_of_call_hierarchy_name"); 
    if ast::is_identifier(name) || ast::is_string_or_numeric_literal_like(name) {
        return name.text().to_string();
    }
    if ast::is_computed_property_name(name) {
        let expr = name.expression().unwrap().clone();
        if ast::is_string_or_numeric_literal_like(&expr) {
            return expr.text().to_string();
        }
    }

    let source_file = ast::get_source_file_of_node(source_node).unwrap();
    let file = crate::ls::mig::m5u::node_as_source_file(&source_file).unwrap();
    let mut c = program.get_type_checker_for_file(&file);
    let symbol = c.get_symbol_at_location(name);
    if let Some(symbol) = symbol {
        let text = c.symbol_to_string(&symbol);
        if !text.is_empty() {
            return text;
        }
    }

    let text_writer = tsox_frontend::format::mig::m4o_2::new_text_writer("\n".to_string(), 0);
    let mut p = tsox_frontend::format::mig::m4o_2::new_printer(
        tsox_frontend::format::mig::m4o_2::PrinterOptions {
            remove_comments: true,
            new_line: tsox_core::core::compiler_options_kinds::NewLineKind::default(),
            omit_trailing_semicolon: false,
            no_emit_helpers: false,
            target: tsox_core::core::compiler_options_kinds::ScriptTarget::default(),
            source_map: false,
            inline_source_map: false,
            inline_sources: false,
            omit_brace_source_map_positions: false,
            only_print_jsdoc_style: false,
            never_ascii_escape: false,
            preserve_source_newlines: false,
            terminate_unterminated_literals: false,
        },
        tsox_frontend::format::mig::m4o_2::PrintHandlers {
            has_global_name: None,
            map_source_position: None,
            on_before_emit_node: None,
            on_after_emit_node: None,
            on_before_emit_node_list: None,
            on_after_emit_node_list: None,
            on_before_emit_token: None,
            on_after_emit_token: None,
        },
        tsox_frontend::format::mig::m4o_2::EmitContext,
    );
    p.write(print_node, Some(&file), text_writer.clone(), None);
    text_writer.string()
}

pub fn get_call_hierarchy_item_container_name(program: &Program, node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_call_hierarchy_item_container_name"); 
    if is_assigned_expression(&Some(node.clone())) {
        let parent = node.parent().unwrap();
        let grand = parent.parent().unwrap();
        if ast::is_property_declaration(&parent) && ast::is_class_like(&grand) {
            if ast::is_class_expression(&grand) {
                if let Some(assigned_name) = get_assigned_name(&grand) {
                    return get_text_of_call_hierarchy_name(program, node, &assigned_name, &assigned_name);
                }
            } else if let Some(name) = grand.name() {
                return get_text_of_call_hierarchy_name(program, node, name, name);
            }
        }
        if let Some(gg) = grand.parent().and_then(|p| p.parent()) {
            if ast::is_module_block(&gg) {
                if let Some(mod_parent) = gg.parent() {
                    if ast::is_module_declaration(&mod_parent) {
                        if let Some(name) = mod_parent.name() {
                            if ast::is_identifier(name) {
                                return name.text().to_string();
                            }
                        }
                    }
                }
            }
        }
        return String::new();
    }

    match node.kind {
        ast::SyntaxKind::GetAccessor | ast::SyntaxKind::SetAccessor | ast::SyntaxKind::MethodDeclaration => {
            let parent = node.parent().unwrap();
            if parent.kind == ast::SyntaxKind::ObjectLiteralExpression {
                if let Some(assigned_name) = get_assigned_name(&parent) {
                    return get_text_of_call_hierarchy_name(program, node, &assigned_name, &assigned_name);
                }
            }
            if let Some(name) = ast::get_name_of_declaration(&parent) {
                return get_text_of_call_hierarchy_name(program, node, &name, &name);
            }
        }
        ast::SyntaxKind::FunctionDeclaration | ast::SyntaxKind::ClassDeclaration | ast::SyntaxKind::ModuleDeclaration => {
            let parent = node.parent().unwrap();
            if ast::is_module_block(&parent) {
                if let Some(gp) = parent.parent() {
                    if ast::is_module_declaration(&gp) {
                        if let Some(name) = gp.name() {
                            if ast::is_identifier(name) {
                                return name.text().to_string();
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    String::new()
}

pub fn move_range_past_modifiers(node: &Arc<Node>) -> TextRange { ::tsox_core::fntrace::enter("move_range_past_modifiers"); 
    if let Some(modifiers) = node.modifiers() {
        let nodes = &modifiers.list.nodes;
        if !nodes.is_empty() {
            let last_mod = &nodes[nodes.len() - 1];
            return TextRange::new(last_mod.end(), node.end());
        }
    }
    TextRange::new(node.pos(), node.end())
}

pub fn find_implementation(c: &Checker, node: &Option<Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_implementation"); 
    let node = node.as_ref()?;
    if !ast::is_function_like_declaration(node) {
        return Some(node.clone());
    }
    if node.body().is_some() {
        return Some(node.clone());
    }
    if ast::is_constructor_declaration(node) {
        return tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body(&node.parent().unwrap());
    }
    if ast::is_function_declaration(node) || ast::is_method_declaration(node) {
        let symbol = get_symbol_of_call_hierarchy_declaration(c, node);
        if let Some(symbol) = symbol {
            if let Some(value_decl) = symbol.value_declaration.as_ref() {
                if ast::is_function_like_declaration(value_decl) && value_decl.body().is_some() {
                    return Some(Arc::clone(value_decl));
                }
            }
        }
        return None;
    }
    Some(node.clone())
}

pub fn find_all_initial_declarations(c: &Checker, node: &Arc<Node>) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("find_all_initial_declarations"); 
    if ast::is_class_static_block_declaration(node) {
        return None;
    }
    let symbol = get_symbol_of_call_hierarchy_declaration(c, node)?;
    let declarations = &symbol.declarations;
    if declarations.is_empty() {
        return None;
    }

    let mut indices: Vec<usize> = (0..declarations.len()).collect();
    let keys: Vec<(String, usize)> = declarations
        .iter()
        .map(|d| {
            let file_name = ast::get_source_file_of_node(d)
                .and_then(|n| crate::ls::mig::m5u::node_as_source_file(&n))
                .map(|f| f.file_name.clone())
                .unwrap_or_default();
            (file_name, d.pos())
        })
        .collect();
    indices.sort_by(|&a, &b| {
        if keys[a].0 != keys[b].0 {
            keys[a].0.cmp(&keys[b].0)
        } else {
            keys[a].1.cmp(&keys[b].1)
        }
    });

    let mut result: Vec<Arc<Node>> = Vec::new();
    let mut last_decl: Option<Arc<Node>> = None;
    for &i in &indices {
        let decl = &declarations[i];
        if is_valid_call_hierarchy_declaration(&Some(decl.clone())) {
            let keep = match &last_decl {
                None => true,
                Some(last) => !Arc::ptr_eq(&last.parent().unwrap_or_else(|| last.clone()), decl.parent().as_ref().unwrap_or(decl)) || last.end() != decl.pos(),
            };
            if keep {
                result.push(decl.clone());
            }
            last_decl = Some(decl.clone());
        }
    }
    Some(result)
}

pub fn find_implementation_or_all_initial_declarations(c: &Checker, node: &Arc<Node>) -> CallHierarchyDeclarationResult { ::tsox_core::fntrace::enter("find_implementation_or_all_initial_declarations"); 
    if ast::is_class_static_block_declaration(node) {
        return CallHierarchyDeclarationResult::Node(node.clone());
    }
    if ast::is_function_like_declaration(node) {
        if let Some(impl_) = find_implementation(c, &Some(node.clone())) {
            return CallHierarchyDeclarationResult::Node(impl_);
        }
        if let Some(decls) = find_all_initial_declarations(c, node) {
            return CallHierarchyDeclarationResult::Nodes(decls);
        }
        return CallHierarchyDeclarationResult::Node(node.clone());
    }
    if let Some(decls) = find_all_initial_declarations(c, node) {
        return CallHierarchyDeclarationResult::Nodes(decls);
    }
    CallHierarchyDeclarationResult::Node(node.clone())
}

pub fn resolve_call_hierarchy_declaration(program: &Program, location: &Arc<Node>) -> Option<CallHierarchyDeclarationResult> { ::tsox_core::fntrace::enter("resolve_call_hierarchy_declaration"); 
    let c = program.get_type_checker();

    let mut following_symbol = false;
    let mut location = location.clone();

    loop {
        if is_valid_call_hierarchy_declaration(&Some(location.clone())) {
            return Some(find_implementation_or_all_initial_declarations(&c, &location));
        }

        if is_possible_call_hierarchy_declaration(&Some(location.clone())) {
            let ancestor = find_ancestor_valid_call_hierarchy_declaration(&location);
            if let Some(ancestor) = ancestor {
                return Some(find_implementation_or_all_initial_declarations(&c, &ancestor));
            }
        }

        if tsox_frontend::ast::mig::m3f_4::is_declaration_name(&location) {
            let parent = location.parent().unwrap();
            if is_valid_call_hierarchy_declaration(&Some(parent.clone())) {
                return Some(find_implementation_or_all_initial_declarations(&c, &parent));
            }
            if is_possible_call_hierarchy_declaration(&Some(parent.clone())) {
                let ancestor = find_ancestor_valid_call_hierarchy_declaration(&parent);
                if let Some(ancestor) = ancestor {
                    return Some(find_implementation_or_all_initial_declarations(&c, &ancestor));
                }
            }
            if is_variable_like(&parent) {
                if let Some(initializer) = parent.initializer() {
                    if is_assigned_expression(&Some(Arc::clone(initializer))) {
                        return Some(CallHierarchyDeclarationResult::Node(Arc::clone(initializer)));
                    }
                }
            }
            return None;
        }

        if ast::is_constructor_declaration(&location) {
            let parent = location.parent().unwrap();
            if is_valid_call_hierarchy_declaration(&Some(parent.clone())) {
                return Some(CallHierarchyDeclarationResult::Node(parent));
            }
            return None;
        }

        if location.kind == ast::SyntaxKind::StaticKeyword && location.parent().map(|p| ast::is_class_static_block_declaration(&p)).unwrap_or(false) {
            location = location.parent().unwrap();
            continue;
        }

        if ast::is_variable_declaration(&location) {
            if let Some(initializer) = location.initializer() {
                if is_assigned_expression(&Some(Arc::clone(initializer))) {
                    return Some(CallHierarchyDeclarationResult::Node(Arc::clone(initializer)));
                }
            }
        }

        if !following_symbol {
            let mut symbol = c.get_symbol_at_location(&location);
            if let Some(s) = symbol.clone() {
                if (s.flags & ast::SymbolFlags::Alias) != ast::SymbolFlags::empty() {
                    symbol = Some(c.get_aliased_symbol(&s));
                }
                if let Some(s) = symbol {
                    if let Some(value_decl) = s.value_declaration.as_ref() {
                        following_symbol = true;
                        location = Arc::clone(value_decl);
                        continue;
                    }
                }
            }
        }

        return None;
    }
}

impl LanguageService {
    pub fn mig_create_call_hierarchy_item(&self, program: &Program, node: &Arc<Node>) -> Option<lsproto_lsp::CallHierarchyItem> { ::tsox_core::fntrace::enter("mig_create_call_hierarchy_item"); 
        let source_file = ast::get_source_file_of_node(node).unwrap();
        let file = crate::ls::mig::m5u::node_as_source_file(&source_file)
            .unwrap_or_else(|| panic!("call hierarchy item: source file node without backing file"));
        let script = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(&file) };
        let (name_text, name_pos, name_end) = get_call_hierarchy_item_name(program, node);
        let container_name = get_call_hierarchy_item_container_name(program, node);

        let kind = crate::ls::symbols::symbol_kind_from_node(node.kind);

        let full_start = tsox_frontend::scanner::skip_trivia_ex(
            source_file.text(),
            node.pos(),
            &tsox_frontend::scanner::SkipTriviaOptions { stop_at_comments: true, ..Default::default() },
            None,
        );
        let (span, span_fidelity) = self.converters.to_lsp_range_for_feature(
            &script,
            TextRange::new(full_start, node.end()),
            FEATURE_CALL_HIERARCHY,
        );
        let (selection_span, selection_fidelity) = self.converters.to_lsp_range_for_feature(
            &script,
            TextRange::new(name_pos, name_end),
            FEATURE_CALL_HIERARCHY,
        );
        if !selection_fidelity.is_single_segment() {
            return None;
        }
        let mut span = span;
        if span_fidelity.is_none()
            || (crate::mig::m5u_conv::source_file_span_map(&file).is_some()
                && !lsp_range_contains(&span, &selection_span))
        {
            span = selection_span.clone();
        }

        let mut item = lsproto_lsp::CallHierarchyItem {
            name: name_text,
            kind,
            uri: lsproto_lsp::DocumentUri(crate::ls::lsconv_converters::file_name_to_document_uri(
                crate::mig::m5u_conv::source_file_original_file_name(&file),
            )),
            range: span,
            selection_range: selection_span,
            detail: None,
            tags: None,
        };
        if !container_name.is_empty() {
            item.detail = Some(container_name);
        }
        Some(item)
    }

    pub fn mig_convert_call_site_group_to_incoming_call(
        &self,
        program: &Program,
        entries: &[CallSite],
    ) -> Option<lsproto_lsp::CallHierarchyIncomingCall> { ::tsox_core::fntrace::enter("mig_convert_call_site_group_to_incoming_call"); 
        let mut from_ranges: Vec<lsproto_lsp::Range> = Vec::with_capacity(entries.len());
        for entry in entries {
            let script = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(&entry.source_file) };
            let (lsp_range, fidelity) = self.converters.to_lsp_range_for_feature(&script, entry.text_range, FEATURE_CALL_HIERARCHY);
            if !fidelity.is_none() {
                from_ranges.push(lsp_range);
            }
        }
        let from = self.mig_create_call_hierarchy_item(program, &entries[0].declaration)?;
        if from_ranges.is_empty() {
            return None;
        }
        from_ranges.sort_by(|a, b| crate::lsp::lsproto_util::compare_ranges(a, b));
        Some(lsproto_lsp::CallHierarchyIncomingCall { from: from.into(), from_ranges })
    }

    pub fn mig_get_incoming_calls(
        &self,
        program: &Program,
        declaration: &Arc<Node>,
        orchestrator: &dyn crate::ls::cross_project::CrossProjectOrchestrator,
    ) -> Result<lsproto_lsp::CallHierarchyIncomingCallsResponse, LspError> { ::tsox_core::fntrace::enter("mig_get_incoming_calls"); 
        if ast::is_source_file(declaration) || ast::is_module_declaration(declaration) || ast::is_class_static_block_declaration(declaration) {
            return Ok(lsproto_lsp::CallHierarchyIncomingCallsResponse::default());
        }

        let location = match get_call_hierarchy_declaration_reference_node(&Some(declaration.clone())) {
            Some(l) => l,
            None => return Ok(lsproto_lsp::CallHierarchyIncomingCallsResponse::default()),
        };
        let location_node = ast::get_source_file_of_node(&location).unwrap();
        let location_file = crate::ls::mig::m5u::node_as_source_file(&location_node).unwrap();
        let location_start = tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(&location, &location_file, false);
        let location_script = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(&location_file) };
        let (_, fidelity) = self.converters.to_lsp_position_for_feature(&location_script, location_start, FEATURE_CALL_HIERARCHY);
        if fidelity.is_none() {
            return Ok(lsproto_lsp::CallHierarchyIncomingCallsResponse::default());
        }

        let incoming_entry = IncomingEntry::new(self.clone_handle(), location);

        let mut result = crate::ls::mig::wt2_cp::handle_cross_project(
            self,
            &incoming_entry,
            orchestrator,
            |l, params, data, options| l.mig_symbol_and_entries_to_incoming_calls(params, data, options),
            combine_incoming_calls,
            false,
            false,
            SymbolEntryTransformOptions::default(),
            None,
        )?;
        if let Some(calls) = result.call_hierarchy_incoming_calls.as_mut() {
            calls.sort_by(|a, b| {
                let uri_comp = a.from.uri.0.cmp(&b.from.uri.0);
                if uri_comp != std::cmp::Ordering::Equal {
                    return uri_comp;
                }
                if a.from_ranges.is_empty() || b.from_ranges.is_empty() {
                    return std::cmp::Ordering::Equal;
                }
                crate::lsp::lsproto_util::compare_ranges(&a.from_ranges[0], &b.from_ranges[0])
            });
        }
        Ok(result)
    }

    pub fn mig_symbol_and_entries_to_incoming_calls(
        &self,
        params: &IncomingEntry,
        data: &SymbolAndEntriesData,
        options: &SymbolEntryTransformOptions,
    ) -> Result<lsproto_lsp::CallHierarchyIncomingCallsResponse, LspError> { ::tsox_core::fntrace::enter("mig_symbol_and_entries_to_incoming_calls"); 
        let program = self.get_program();
        let mut ref_entries: Vec<&ReferenceEntry> = Vec::new();
        for symbol_and_entry in &data.symbols_and_entries {
            for r in &symbol_and_entry.references {
                ref_entries.push(r);
            }
        }

        let mut call_sites: Vec<CallSite> = Vec::new();
        for entry in &ref_entries {
            if let Some(site) = convert_entry_to_call_site(entry) {
                call_sites.push(site);
            }
        }

        if call_sites.is_empty() {
            return Ok(lsproto_lsp::CallHierarchyIncomingCallsResponse::default());
        }

        let mut grouped: HashMap<u64, Vec<CallSite>> = HashMap::new();
        for site in call_sites {
            let key = get_call_site_group_key(&site);
            grouped.entry(key).or_default().push(site);
        }

        let mut result: Vec<lsproto_lsp::CallHierarchyIncomingCall> = Vec::new();
        for sites in grouped.values() {
            if let Some(incoming_call) = self.mig_convert_call_site_group_to_incoming_call(&program, sites) {
                result.push(incoming_call);
            }
        }
        Ok(lsproto_lsp::CallHierarchyIncomingCallsResponse {
            call_hierarchy_incoming_calls: Some(result),
        })
    }

    pub fn mig_convert_call_site_group_to_outgoing_call(
        &self,
        program: &Program,
        entries: &[CallSite],
    ) -> Option<lsproto_lsp::CallHierarchyOutgoingCall> { ::tsox_core::fntrace::enter("mig_convert_call_site_group_to_outgoing_call"); 
        let mut from_ranges: Vec<lsproto_lsp::Range> = Vec::with_capacity(entries.len());
        for entry in entries {
            let script = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(&entry.source_file) };
            let (lsp_range, fidelity) = self.converters.to_lsp_range_for_feature(&script, entry.text_range, FEATURE_CALL_HIERARCHY);
            if !fidelity.is_none() {
                from_ranges.push(lsp_range);
            }
        }
        let to = self.mig_create_call_hierarchy_item(program, &entries[0].declaration)?;
        if from_ranges.is_empty() {
            return None;
        }
        from_ranges.sort_by(|a, b| crate::lsp::lsproto_util::compare_ranges(a, b));
        Some(lsproto_lsp::CallHierarchyOutgoingCall { to: to.into(), from_ranges })
    }

    pub fn mig_get_outgoing_calls(&self, program: &Arc<Program>, declaration: &Arc<Node>) -> Option<Vec<lsproto_lsp::CallHierarchyOutgoingCall>> { ::tsox_core::fntrace::enter("mig_get_outgoing_calls"); 
        if (ast::get_combined_node_flags(declaration) & ast::NodeFlags::Ambient) != ast::NodeFlags::empty()
            || ast::is_method_signature_declaration(declaration)
        {
            return None;
        }

        let c = program.get_type_checker();
        let call_sites = collect_call_sites(program, &c, declaration);
        if call_sites.is_empty() {
            return None;
        }

        let mut grouped: HashMap<u64, Vec<CallSite>> = HashMap::new();
        for site in call_sites {
            let key = get_call_site_group_key(&site);
            grouped.entry(key).or_default().push(site);
        }

        let mut result: Vec<lsproto_lsp::CallHierarchyOutgoingCall> = Vec::new();
        for sites in grouped.values() {
            if let Some(outgoing_call) = self.mig_convert_call_site_group_to_outgoing_call(program, sites) {
                result.push(outgoing_call);
            }
        }

        result.sort_by(|a, b| {
            let uri_comp = a.to.uri.0.cmp(&b.to.uri.0);
            if uri_comp != std::cmp::Ordering::Equal {
                return uri_comp;
            }
            if a.from_ranges.is_empty() || b.from_ranges.is_empty() {
                return std::cmp::Ordering::Equal;
            }
            crate::lsp::lsproto_util::compare_ranges(&a.from_ranges[0], &b.from_ranges[0])
        });

        Some(result)
    }

    pub fn mig_provide_prepare_call_hierarchy(
        &self,
        document_uri: &lsproto::DocumentUri,
        position: &lsproto::Position,
    ) -> Result<lsproto_lsp::CallHierarchyPrepareResponse, LspError> { ::tsox_core::fntrace::enter("mig_provide_prepare_call_hierarchy"); 
        let (program, file) = self.get_program_and_file(document_uri);
        let declarations = self.mig_call_hierarchy_declarations(&file, position, &program, false);
        let mut items: Vec<lsproto_lsp::CallHierarchyItem> = Vec::new();
        let mut seen: std::collections::HashSet<(String, u32, u32)> = Default::default();
        for declaration in &declarations {
            if let Some(item) = self.mig_create_call_hierarchy_item(&program, declaration) {
                let key = (
                    item.uri.0.clone(),
                    item.selection_range.start.line,
                    item.selection_range.start.character,
                );
                if seen.insert(key) {
                    items.push(item);
                }
            }
        }
        if items.is_empty() {
            return Ok(lsproto_lsp::CallHierarchyPrepareResponse::default());
        }
        Ok(lsproto_lsp::CallHierarchyPrepareResponse { call_hierarchy_items: Some(items) })
    }

    pub fn mig_call_hierarchy_declarations(
        &self,
        file: &Arc<SourceFile>,
        position: &lsproto::Position,
        program: &Program,
        allow_source_file: bool,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("mig_call_hierarchy_declarations"); 
        let positions = self.converters.from_lsp_position_for_source_file_m5u(file, position.clone(), FEATURE_CALL_HIERARCHY);
        let mut declarations: Vec<Arc<Node>> = Vec::new();
        let mut seen: tsox_core::collections::set::Set<u64> = Default::default();
        for mapped in &positions {
            if !mapped.fidelity.is_single_segment() {
                continue;
            }
            let file = &mapped.file;
            let pos = mapped.position;
            let node = if pos != 0 {
                tsox_frontend::astnav::get_touching_property_name(&file.node, pos)
            } else {
                Some(Arc::clone(&file.node))
            };
            let Some(node) = node else { continue };
            if !allow_source_file && node.kind == ast::SyntaxKind::SourceFile {
                continue;
            }
            if let Some(declaration) = resolve_call_hierarchy_declaration(program, &node) {
                match declaration {
                    CallHierarchyDeclarationResult::Node(d) => {
                        if seen.add_if_absent(tsox_frontend::ast::mig::m3f::get_node_id(&d)) {
                            declarations.push(d);
                        }
                    }
                    CallHierarchyDeclarationResult::Nodes(ds) => {
                        for d in ds {
                            if seen.add_if_absent(tsox_frontend::ast::mig::m3f::get_node_id(&d)) {
                                declarations.push(d);
                            }
                        }
                    }
                }
            }
        }
        declarations
    }
}

pub fn convert_entry_to_call_site(entry: &ReferenceEntry) -> Option<CallSite> { ::tsox_core::fntrace::enter("convert_entry_to_call_site"); 
    if entry.kind != EntryKind::Node {
        return None;
    }
    let node = entry.node.as_ref().unwrap();
    if !tsox_frontend::ast::mig::m3f_3::is_call_or_new_expression_target(node, true, true)
        && !tsox_frontend::ast::mig::m3g::is_tagged_template_tag(node, true, true)
        && !tsox_frontend::ast::mig::m3f_4::is_decorator_target(node, true, true)
        && !tsox_frontend::ast::mig::m3g::is_jsx_opening_like_element_tag_name(node, true, true)
        && !tsox_frontend::ast::mig::m3g_2::is_right_side_of_property_access(node)
        && !tsox_frontend::ast::mig::m3f_3::is_argument_expression_of_element_access(node)
    {
        return None;
    }

    let source_file_node = ast::get_source_file_of_node(node).unwrap();
    let mut ancestor = find_ancestor_valid_call_hierarchy_declaration(node);
    let ancestor = match ancestor {
        Some(a) => a,
        None => source_file_node.clone(),
    };
    let source_file = crate::ls::mig::m5u::node_as_source_file(&source_file_node).unwrap();

    let start = tsox_frontend::scanner::skip_trivia(source_file_node.text(), node.pos());
    Some(CallSite {
        declaration: ancestor,
        text_range: TextRange::new(start, node.end()),
        source_file,
    })
}

pub fn get_call_site_group_key(site: &CallSite) -> u64 { ::tsox_core::fntrace::enter("get_call_site_group_key"); 
    tsox_frontend::ast::mig::m3f::get_node_id(&site.declaration)
}

impl CallSiteCollector {
    pub fn mig_record_call_site(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("mig_record_call_site"); 
        let target: Option<Arc<Node>> = if ast::is_tagged_template_expression(node) {
            Some(Arc::clone(&node.as_tagged_template_expression().tag))
        } else if ast::is_jsx_opening_element(node) {
            Some(Arc::clone(tsox_frontend::ast::mig::m3c::tag_name(node)))
        } else if ast::is_jsx_self_closing_element(node) {
            Some(Arc::clone(tsox_frontend::ast::mig::m3c::tag_name(node)))
        } else if ast::is_property_access_expression(node) || ast::is_element_access_expression(node) {
            Some(node.clone())
        } else if ast::is_class_static_block_declaration(node) {
            Some(node.clone())
        } else if ast::is_call_expression(node) {
            node.expression().cloned()
        } else if ast::is_new_expression(node) {
            node.expression().cloned()
        } else if ast::is_decorator(node) {
            node.expression().cloned()
        } else {
            None
        };

        let Some(target) = target else { return };

        let declaration = match resolve_call_hierarchy_declaration(&self.program, &target) {
            Some(d) => d,
            None => return,
        };

        let source_file_node = ast::get_source_file_of_node(&target).unwrap();
        let source_file = crate::ls::mig::m5u::node_as_source_file(&source_file_node).unwrap();
        let start = tsox_frontend::scanner::skip_trivia(source_file_node.text(), target.pos());
        let text_range = TextRange::new(start, target.end());

        match declaration {
            CallHierarchyDeclarationResult::Node(decl) => {
                self.call_sites.push(CallSite { declaration: decl, text_range, source_file });
            }
            CallHierarchyDeclarationResult::Nodes(decls) => {
                for d in decls {
                    self.call_sites.push(CallSite { declaration: d, text_range, source_file: source_file.clone() });
                }
            }
        }
    }

    pub fn mig_collect(&mut self, node: &Option<Arc<Node>>) { ::tsox_core::fntrace::enter("mig_collect"); 
        let Some(node) = node else { return };

        if (node.flags & ast::NodeFlags::Ambient) != ast::NodeFlags::empty() {
            return;
        }

        if is_valid_call_hierarchy_declaration(&Some(node.clone())) {
            if ast::is_class_like(node) {
                for member in tsox_frontend::ast::mig::m3b::members(node) {
                    if let Some(name) = member.name() {
                        if ast::is_computed_property_name(name) {
                            self.mig_collect(&Some(name.expression().unwrap().clone()));
                        }
                    }
                }
            }
            return;
        }

        match node.kind {
            ast::SyntaxKind::Identifier
            | ast::SyntaxKind::ImportEqualsDeclaration
            | ast::SyntaxKind::ImportDeclaration
            | ast::SyntaxKind::ExportDeclaration
            | ast::SyntaxKind::InterfaceDeclaration
            | ast::SyntaxKind::TypeAliasDeclaration => {}
            ast::SyntaxKind::ClassStaticBlockDeclaration => {
                self.mig_record_call_site(node);
            }
            ast::SyntaxKind::TypeAssertionExpression | ast::SyntaxKind::AsExpression => {
                self.mig_collect(&node.expression().cloned());
            }
            ast::SyntaxKind::VariableDeclaration | ast::SyntaxKind::Parameter => {
                self.mig_collect(&node.name().cloned());
                self.mig_collect(&node.initializer().cloned());
            }
            ast::SyntaxKind::CallExpression => {
                self.mig_record_call_site(node);
                self.mig_collect(&node.expression().cloned());
                for arg in ast::mig::x1a::arguments(node) {
                    self.mig_collect(&Some(Arc::clone(arg)));
                }
            }
            ast::SyntaxKind::NewExpression => {
                self.mig_record_call_site(node);
                self.mig_collect(&node.expression().cloned());
                for arg in ast::mig::x1a::arguments(node) {
                    self.mig_collect(&Some(Arc::clone(arg)));
                }
            }
            ast::SyntaxKind::TaggedTemplateExpression => {
                self.mig_record_call_site(node);
                let tagged_template = node.as_tagged_template_expression();
                self.mig_collect(&Some(Arc::clone(&tagged_template.tag)));
                self.mig_collect(&Some(Arc::clone(&tagged_template.template)));
            }
            ast::SyntaxKind::JsxOpeningElement | ast::SyntaxKind::JsxSelfClosingElement => {
                self.mig_record_call_site(node);
                self.mig_collect(&Some(Arc::clone(tsox_frontend::ast::mig::m3c::tag_name(node))));
                self.mig_collect(&Some(Arc::clone(node.attributes())));
            }
            ast::SyntaxKind::Decorator => {
                self.mig_record_call_site(node);
                self.mig_collect(&node.expression().cloned());
            }
            ast::SyntaxKind::PropertyAccessExpression | ast::SyntaxKind::ElementAccessExpression => {
                self.mig_record_call_site(node);
                tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
                    self.mig_collect(&Some(Arc::clone(child)));
                    false
                });
            }
            ast::SyntaxKind::SatisfiesExpression => {
                self.mig_collect(&node.expression().cloned());
            }
            _ => {
                if tsox_frontend::ast::mig::m3g_3::is_part_of_type_node(node) {
                    return;
                }
                tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
                    self.mig_collect(&Some(Arc::clone(child)));
                    false
                });
            }
        }
    }
}

pub fn collect_call_sites(program: &Arc<Program>, c: &Checker, node: &Arc<Node>) -> Vec<CallSite> { ::tsox_core::fntrace::enter("collect_call_sites"); 
    let mut collector = CallSiteCollector { program: program.clone(), call_sites: Vec::new() };

    match node.kind {
        ast::SyntaxKind::SourceFile => {
            for stmt in tsox_frontend::ast::mig::m3c::statements(node) {
                collector.mig_collect(&Some(stmt.clone()));
            }
        }
        ast::SyntaxKind::ModuleDeclaration => {
            let body = node.body();
            if (node.syntactic_modifier_flags() & ast::ModifierFlags::Ambient) == ast::ModifierFlags::empty()
                && body.is_some()
                && ast::is_module_block(body.as_ref().unwrap())
            {
                for stmt in tsox_frontend::ast::mig::m3c::statements(body.unwrap()) {
                    collector.mig_collect(&Some(stmt.clone()));
                }
            }
        }
        ast::SyntaxKind::FunctionDeclaration
        | ast::SyntaxKind::FunctionExpression
        | ast::SyntaxKind::ArrowFunction
        | ast::SyntaxKind::MethodDeclaration
        | ast::SyntaxKind::GetAccessor
        | ast::SyntaxKind::SetAccessor => {
            let impl_ = find_implementation(c, &Some(node.clone()));
            if let Some(impl_) = impl_ {
                for param in ast::mig::m3b::parameters(&impl_) {
                    collector.mig_collect(&Some(Arc::clone(param)));
                }
                collector.mig_collect(&impl_.body().cloned());
            }
        }
        ast::SyntaxKind::ClassDeclaration | ast::SyntaxKind::ClassExpression => {
            if let Some(modifiers) = node.modifiers() {
                for m in &modifiers.list.nodes {
                    collector.mig_collect(&Some(m.clone()));
                }
            }
            let heritage = ast::get_class_extends_heritage_element(node);
            if let Some(heritage) = heritage {
                collector.mig_collect(&Some(heritage.expression().unwrap().clone()));
            }
            for member in tsox_frontend::ast::mig::m3b::members(node) {
                if ast::can_have_modifiers(member) && member.modifiers().is_some() {
                    for m in &member.modifiers().unwrap().list.nodes {
                        collector.mig_collect(&Some(m.clone()));
                    }
                }
                if ast::is_property_declaration(member) {
                    collector.mig_collect(&member.initializer().cloned());
                } else if ast::is_constructor_declaration(member) {
                    if let Some(body) = member.body() {
                        for param in ast::mig::m3b::parameters(member) {
                            collector.mig_collect(&Some(Arc::clone(param)));
                        }
                        collector.mig_collect(&Some(Arc::clone(body)));
                    }
                } else if ast::is_class_static_block_declaration(member) {
                    collector.mig_collect(&Some(member.clone()));
                }
            }
        }
        ast::SyntaxKind::ClassStaticBlockDeclaration => {
            let static_block = node.as_class_static_block_declaration();
            collector.mig_collect(&Some(Arc::clone(&static_block.body)));
        }
        _ => {}
    }

    collector.call_sites
}

pub fn lsp_range_contains(outer: &lsproto_lsp::Range, inner: &lsproto_lsp::Range) -> bool { ::tsox_core::fntrace::enter("lsp_range_contains"); 
    use crate::lsp::lsproto_util::compare_positions;
    compare_positions(&outer.start, &inner.start) != std::cmp::Ordering::Greater
        && compare_positions(&outer.end, &inner.end) != std::cmp::Ordering::Less
}

pub fn combine_incoming_calls(
    results: &[lsproto_lsp::CallHierarchyIncomingCallsResponse],
) -> lsproto_lsp::CallHierarchyIncomingCallsResponse { ::tsox_core::fntrace::enter("combine_incoming_calls"); 
    let mut combined: Vec<lsproto_lsp::CallHierarchyIncomingCall> = Vec::new();
    let mut seen_calls: std::collections::HashSet<(String, u32, u32)> = Default::default();
    for resp in results {
        if let Some(calls) = &resp.call_hierarchy_incoming_calls {
            for call in calls {
                let key = (
                    call.from.uri.0.clone(),
                    call.from.selection_range.start.line,
                    call.from.selection_range.start.character,
                );
                if seen_calls.insert(key) {
                    combined.push(call.clone());
                }
            }
        }
    }
    lsproto_lsp::CallHierarchyIncomingCallsResponse {
        call_hierarchy_incoming_calls: Some(combined),
    }
}

impl LanguageService {
    pub fn clone_handle(&self) -> LanguageService { ::tsox_core::fntrace::enter("clone_handle"); 
        unimplemented!(
            "LanguageService clone_handle: Go 在此共享指针, Rust 侧 host 为 Box<dyn Host> 无法克隆, 需 Host::clone_box 支持 (见 progress_notes_r60k15.md 交接)"
        )
    }
}

fn find_ancestor_valid_call_hierarchy_declaration(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_ancestor_valid_call_hierarchy_declaration"); 
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        if is_valid_call_hierarchy_declaration(&Some(Arc::clone(&n))) {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

trait M5pNodeExt {
    fn as_tagged_template_expression(&self) -> &ast::node_data_generated::TaggedTemplateExpressionData;
    fn as_class_static_block_declaration(&self) -> &ast::node_data_generated::ClassStaticBlockDeclarationData;
    fn attributes(&self) -> &Arc<Node>;
}

impl M5pNodeExt for Node {
    fn as_tagged_template_expression(&self) -> &ast::node_data_generated::TaggedTemplateExpressionData { ::tsox_core::fntrace::enter("as_tagged_template_expression"); 
        match &self.data {
            ast::node_data_generated::NodeData::TaggedTemplateExpression(d) => d,
            _ => panic!("as_tagged_template_expression on non-TaggedTemplateExpression node"),
        }
    }

    fn as_class_static_block_declaration(&self) -> &ast::node_data_generated::ClassStaticBlockDeclarationData { ::tsox_core::fntrace::enter("as_class_static_block_declaration"); 
        match &self.data {
            ast::node_data_generated::NodeData::ClassStaticBlockDeclaration(d) => d,
            _ => panic!("as_class_static_block_declaration on non-ClassStaticBlockDeclaration node"),
        }
    }

    fn attributes(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("attributes"); 
        match &self.data {
            ast::node_data_generated::NodeData::JsxOpeningElement(d) => &d.attributes,
            ast::node_data_generated::NodeData::JsxSelfClosingElement(d) => &d.attributes,
            _ => panic!("attributes on non-JSX element node"),
        }
    }
}
