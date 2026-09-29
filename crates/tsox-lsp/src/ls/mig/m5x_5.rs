#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};

use super::m5x_4::is_object_binding_element_without_property_name;
use super::m5x_3::{is_right_side_of_property_access, range_contains_range};

pub fn symbol_flags_have_meaning(
    flags: ast::SymbolFlags,
    meaning: ast::mig::m3e_4::SemanticMeaning,
) -> bool {
    if meaning == ast::mig::m3e_4::SemanticMeaning::ALL {
        return true;
    }
    if meaning.contains(ast::mig::m3e_4::SemanticMeaning::VALUE) {
        return flags.contains(ast::SymbolFlags::VALUE);
    }
    if meaning.contains(ast::mig::m3e_4::SemanticMeaning::TYPE) {
        return flags.contains(ast::SymbolFlags::TYPE);
    }
    if meaning.contains(ast::mig::m3e_4::SemanticMeaning::NAMESPACE) {
        return flags.contains(ast::SymbolFlags::NAMESPACE);
    }
    false
}

pub fn get_meaning_from_declaration(node: &Arc<Node>) -> ast::mig::m3e_4::SemanticMeaning {
    match node.kind {
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::BindingElement
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::ShorthandPropertyAssignment
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::Constructor
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::CatchClause
        | SyntaxKind::JsxAttribute => ast::mig::m3e_4::SemanticMeaning::VALUE,
        SyntaxKind::TypeParameter
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::TypeLiteral => ast::mig::m3e_4::SemanticMeaning::TYPE,
        SyntaxKind::EnumMember | SyntaxKind::ClassDeclaration => {
            ast::mig::m3e_4::SemanticMeaning::VALUE.union(ast::mig::m3e_4::SemanticMeaning::TYPE)
        }
        SyntaxKind::ModuleDeclaration => {
            if ast::is_ambient_module(node) {
                ast::mig::m3e_4::SemanticMeaning::NAMESPACE.union(ast::mig::m3e_4::SemanticMeaning::VALUE)
            } else if ast::get_module_instance_state(node) == ast::ModuleInstanceState::Instantiated {
                ast::mig::m3e_4::SemanticMeaning::NAMESPACE.union(ast::mig::m3e_4::SemanticMeaning::VALUE)
            } else {
                ast::mig::m3e_4::SemanticMeaning::NAMESPACE
            }
        }
        SyntaxKind::EnumDeclaration
        | SyntaxKind::NamedImports
        | SyntaxKind::ImportSpecifier
        | SyntaxKind::ImportEqualsDeclaration
        | SyntaxKind::ImportDeclaration
        | SyntaxKind::JSImportDeclaration
        | SyntaxKind::ExportAssignment
        | SyntaxKind::ExportDeclaration => ast::mig::m3e_4::SemanticMeaning::ALL,
        SyntaxKind::SourceFile => {
            ast::mig::m3e_4::SemanticMeaning::NAMESPACE.union(ast::mig::m3e_4::SemanticMeaning::VALUE)
        }
        _ => ast::mig::m3e_4::SemanticMeaning::ALL,
    }
}

pub fn get_intersecting_meaning_from_declarations(
    node: Option<&Arc<Node>>,
    symbol: &tsox_frontend::ast::Symbol,
    default_meaning: ast::mig::m3e_4::SemanticMeaning,
) -> ast::mig::m3e_4::SemanticMeaning {
    let Some(node) = node else {
        return default_meaning;
    };
    let mut meaning = crate::ls::mig::m5x_6::get_meaning_from_location(node);
    let declarations: Vec<Arc<Node>> = symbol.declarations.clone();
    if declarations.is_empty() {
        return meaning;
    }
    let mut last_iteration_meaning = meaning;
    loop {
        for declaration in &declarations {
            let declaration_meaning = get_meaning_from_declaration(declaration);
            if declaration_meaning.0 & meaning.0 != 0 {
                meaning = meaning.union(declaration_meaning);
            }
        }
        if meaning == last_iteration_meaning {
            break;
        }
        last_iteration_meaning = meaning;
    }
    meaning
}

pub fn get_all_super_type_nodes(node: &Arc<Node>) -> Vec<Arc<Node>> {
    if ast::is_interface_declaration(node) {
        return ast::get_extends_heritage_clause_elements(node);
    }
    if ast::is_class_like(node) {
        let mut result = Vec::new();
        if let Some(extends) = ast::get_class_extends_heritage_element(node) {
            result.push(extends);
        }
        result.extend(ast::get_implements_heritage_clause_elements(node));
        return result;
    }
    Vec::new()
}

pub fn get_parent_symbols_of_property_access(
    location: &Arc<Node>,
    symbol: &tsox_frontend::ast::Symbol,
    ch: &mut tsox_checker::checker::Checker,
) -> Vec<Arc<tsox_frontend::ast::Symbol>> {
    if !is_right_side_of_property_access(location) {
        return Vec::new();
    }
    let Some(parent) = location.parent() else {
        return Vec::new();
    };
    let Some(expr) = crate::ls::mig::m5x_3::access_expression_expression(&parent) else {
        return Vec::new();
    };
    let lhs_type = ch.get_type_at_location(&expr);
    let mut possible_symbols: Vec<Arc<tsox_checker::checker::types::Type>> = Vec::new();
    if lhs_type
        .flags
        .intersects(tsox_checker::checker::types::TypeFlags::UNION_OR_INTERSECTION)
    {
        possible_symbols = lhs_type.types().unwrap_or_default().to_vec();
    } else if lhs_type
        .symbol()
        .map_or(true, |s| symbol.parent().map_or(true, |p| !Arc::ptr_eq(s, &p)))
    {
        possible_symbols = vec![Arc::clone(&lhs_type)];
    }
    possible_symbols
        .into_iter()
        .filter_map(|t| {
            t.symbol().and_then(|s| {
                if s.flags
                    .intersects(ast::SymbolFlags::Class | ast::SymbolFlags::Interface)
                {
                    Some(Arc::clone(s))
                } else {
                    None
                }
            })
        })
        .collect()
}

pub fn get_property_symbols_from_base_types(
    symbol: &tsox_frontend::ast::Symbol,
    property_name: &str,
    checker: &mut tsox_checker::checker::Checker,
    cb: &mut dyn FnMut(&Arc<tsox_frontend::ast::Symbol>) -> Option<Arc<tsox_frontend::ast::Symbol>>,
) -> Option<Arc<tsox_frontend::ast::Symbol>> {
    fn recur(
        symbol: &tsox_frontend::ast::Symbol,
        property_name: &str,
        checker: &mut tsox_checker::checker::Checker,
        cb: &mut dyn FnMut(&Arc<tsox_frontend::ast::Symbol>) -> Option<Arc<tsox_frontend::ast::Symbol>>,
        seen: &mut std::collections::HashSet<(u64, usize, usize)>,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> {
        if !symbol
            .flags
            .intersects(ast::SymbolFlags::Class | ast::SymbolFlags::Interface)
        {
            return None;
        }
        let key = symbol_key(symbol);
        if !seen.insert(key) {
            return None;
        }
        for declaration in &symbol.declarations {
            for type_reference in get_all_super_type_nodes(declaration) {
                let property_type = checker.get_type_at_location(&type_reference);
                {
                    if property_type.symbol().is_some() {
                        if let Some(property_symbol) =
                            checker.get_property_of_type(&property_type, property_name)
                        {
                            for root_symbol in checker.get_root_symbols(&property_symbol) {
                                if let Some(result) = cb(&root_symbol) {
                                    return Some(result);
                                }
                            }
                        }
                        if let Some(result) = recur(
                            &property_type.symbol().unwrap(),
                            property_name,
                            checker,
                            cb,
                            seen,
                        ) {
                            return Some(result);
                        }
                    }
                }
            }
        }
        None
    }
    let mut seen = std::collections::HashSet::new();
    recur(
        symbol,
        property_name,
        checker,
        cb,
        &mut seen,
    )
}

fn symbol_key(symbol: &tsox_frontend::ast::Symbol) -> (u64, usize, usize) {
    (
        symbol
            .declarations
            .first()
            .map(crate::ls::mig::m5w_6::node_root_file_id)
            .unwrap_or(0),
        symbol.declarations.first().map(|d| d.pos()).unwrap_or(0),
        symbol.declarations.first().map(|d| d.end()).unwrap_or(0),
    )
}

pub fn get_property_symbol_from_binding_element(
    checker: &mut tsox_checker::checker::Checker,
    binding_element: &Arc<Node>,
) -> Option<Arc<tsox_frontend::ast::Symbol>> {
    let parent = binding_element.parent()?;
    let type_of_pattern = checker.get_type_at_location(&parent);
    if let Some(name) = crate::ls::mig::m5x_3::binding_element_name(binding_element) {
        return checker.get_property_of_type(&type_of_pattern, &ast::node_text(&name));
    }
    None
}

pub fn get_property_symbol_of_object_binding_pattern_without_property_name(
    symbol: &tsox_frontend::ast::Symbol,
    checker: &mut tsox_checker::checker::Checker,
) -> Option<Arc<tsox_frontend::ast::Symbol>> {
    let binding_element = ast::mig::m3e_4::get_declaration_of_kind(symbol, SyntaxKind::BindingElement)?;
    if is_object_binding_element_without_property_name(&binding_element) {
        return get_property_symbol_from_binding_element(checker, &binding_element);
    }
    None
}

pub fn skip_constraint(
    t: &Arc<tsox_checker::checker::types::Type>,
    type_checker: &tsox_checker::checker::Checker,
) -> Arc<tsox_checker::checker::types::Type> {
    if t.is_type_parameter() {
        if let Some(c) = type_checker.get_base_constraint_of_type(t) {
            return c;
        }
    }
    Arc::clone(t)
}

pub fn get_possible_generic_signatures(
    called: &Arc<Node>,
    type_argument_count: usize,
    c: &mut tsox_checker::checker::Checker,
) -> Vec<Arc<tsox_checker::checker::types::Signature>> {
    let mut type_at_location = c.get_type_at_location(called);
    if let Some(parent) = called.parent() {
        if ast::is_optional_chain(&parent) {
            type_at_location = remove_optionality(
                &type_at_location,
                ast::mig::m3g_2::is_optional_chain_root(&parent),
                true,
                c,
            );
        }
    }
    let signatures = if called
        .parent()
        .map_or(false, |parent| ast::is_new_expression(&parent))
    {
        c.get_signatures_of_type(&type_at_location, tsox_checker::checker::types::SignatureKind::Construct)
    } else {
        c.get_signatures_of_type(&type_at_location, tsox_checker::checker::types::SignatureKind::Call)
    };
    signatures
        .into_iter()
        .filter(|s| s.type_parameters.len() >= type_argument_count)
        .collect()
}

pub fn remove_optionality(
    t: &Arc<tsox_checker::checker::types::Type>,
    is_optional_expression: bool,
    is_optional_chain: bool,
    c: &mut tsox_checker::checker::Checker,
) -> Arc<tsox_checker::checker::types::Type> {
    if is_optional_expression {
        return c.get_non_nullable_type(t);
    } else if is_optional_chain {
        return c.get_non_optional_type(t);
    }
    Arc::clone(t)
}

pub fn find_preceding_matching_token(
    token: &Arc<Node>,
    matching_token_kind: SyntaxKind,
    source_file: &Arc<SourceFile>,
) -> Option<Arc<Node>> {
    let close_token_text = tsox_frontend::scanner::token_to_string(token.kind);
    let matching_token_text = tsox_frontend::scanner::token_to_string(matching_token_kind);
    let text = &source_file.text;
    let best_guess_index = text.rfind(&matching_token_text)?;
    if text
        .rfind(&close_token_text)
        .map_or(false, |idx| idx < best_guess_index)
    {
        let node_at_guess =
            tsox_frontend::astnav::find_preceding_token(&source_file.node, best_guess_index + 1);
        if let Some(node) = node_at_guess {
            if node.kind == matching_token_kind {
                return Some(node);
            }
        }
    }
    let mut token_kind = token.kind;
    let mut remaining_matching_tokens = 0usize;
    let mut current = Arc::clone(token);
    loop {
        let preceding =
            tsox_frontend::astnav::find_preceding_token(&source_file.node, current.pos() as usize)?;
        current = preceding;
        if current.kind == matching_token_kind {
            if remaining_matching_tokens == 0 {
                return Some(current);
            }
            remaining_matching_tokens -= 1;
        } else if current.kind == token_kind {
            remaining_matching_tokens += 1;
        }
    }
}

pub fn get_possible_type_arguments_info(
    token_in: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Option<M5xPossibleTypeArgumentInfo> {
    crate::ls::mig::m5x_7::get_possible_type_arguments_info_worker(token_in, source_file)
}

pub struct M5xPossibleTypeArgumentInfo {
    pub called: Arc<Node>,
    pub n_type_arguments: usize,
}

pub fn get_local_symbol_for_export_specifier(
    reference_location: &Arc<Node>,
    reference_symbol: &Arc<tsox_frontend::ast::Symbol>,
    export_specifier: &Arc<Node>,
    ch: &mut tsox_checker::checker::Checker,
) -> Arc<tsox_frontend::ast::Symbol> {
    if crate::ls::mig::m5x_6::is_export_specifier_alias(reference_location, export_specifier) {
        if let Some(symbol) =
            ch.get_export_specifier_local_target_symbol(export_specifier)
        {
            return symbol;
        }
    }
    Arc::clone(reference_symbol)
}

pub fn to_context_range(
    text_range: Option<TextRange>,
    context_file: &Arc<SourceFile>,
    context: Option<&Arc<Node>>,
) -> Option<TextRange> {
    let text_range = text_range?;
    let Some(context) = context else {
        return Some(text_range);
    };
    let context_range = crate::ls::mig::m5x_7::get_range_of_node(context, context_file, None);
    if context_range.pos != text_range.pos || context_range.end != text_range.end {
        return Some(context_range);
    }
    None
}

pub fn get_reference_at_position_m5x(
    source_file: &Arc<SourceFile>,
    position: usize,
    program: &Arc<tsox_compile::compiler::Program>,
) -> Option<M5xRefInfo> {
    let referenced_files: Vec<Arc<ast::node_source_file::FileReference>> = source_file
        .referenced_files
        .iter()
        .map(|r| Arc::new(r.clone()))
        .collect();
    if let Some(reference_path) =
        super::m5x_3::find_reference_in_position(&referenced_files, position)
    {
        if let Some(file) = program.get_source_file_from_reference(source_file, &reference_path) {
            return Some(M5xRefInfo {
                reference: Some(reference_path),
                file_name: file.file_name.clone(),
                file: Some(file),
                unverified: false,
            });
        }
        return None;
    }
    let type_reference_directives: Vec<Arc<ast::node_source_file::FileReference>> = source_file
        .type_reference_directives
        .iter()
        .map(|r| Arc::new(r.clone()))
        .collect();
    if let Some(type_reference_directive) =
        super::m5x_3::find_reference_in_position(&type_reference_directives, position)
    {
        if let Some(resolved) = program
            .get_resolved_type_reference_directive_from_type_reference_directive(
                &type_reference_directive,
                source_file,
            )
        {
            if let Some(file) = program.get_source_file(&resolved.resolved_file_name) {
                return Some(M5xRefInfo {
                    reference: Some(type_reference_directive),
                    file_name: file.file_name.clone(),
                    file: Some(file),
                    unverified: false,
                });
            }
        }
        return None;
    }
    let lib_reference_directives: Vec<Arc<ast::node_source_file::FileReference>> = source_file
        .lib_reference_directives
        .iter()
        .map(|r| Arc::new(r.clone()))
        .collect();
    if let Some(lib_reference_directive) =
        super::m5x_3::find_reference_in_position(&lib_reference_directives, position)
    {
        if let Some(file) = program.get_lib_file_from_reference(&lib_reference_directive) {
            return Some(M5xRefInfo {
                reference: Some(lib_reference_directive),
                file_name: file.file_name.clone(),
                file: Some(file),
                unverified: false,
            });
        }
        return None;
    }
    if source_file.imports.is_empty() && source_file.module_augmentations.is_empty() {
        return None;
    }
    let node = tsox_frontend::astnav::get_touching_token(&source_file.node, position)?;
    if !crate::ls::mig::m5x_7::is_module_specifier_like(&node)
        || !tsox_core::tspath::is_external_module_name_relative(&ast::node_text(&node))
    {
        return None;
    }
    let resolution = program.get_resolved_module_from_module_specifier(source_file, &node)?;
    let verified_file_name = resolution.resolved_file_name.clone();
    let mut file_name = resolution.resolved_file_name.clone();
    if file_name.is_empty() {
        file_name = tsox_core::tspath::resolve_path(
            &tsox_core::tspath::get_directory_path(&source_file.file_name),
            &[&ast::node_text(&node)],
        );
    }
    Some(M5xRefInfo {
        file: program.get_source_file(&file_name),
        file_name,
        reference: None,
        unverified: !verified_file_name.is_empty(),
    })
}

pub struct M5xRefInfo {
    pub reference: Option<Arc<ast::node_source_file::FileReference>>,
    pub file_name: String,
    pub file: Option<Arc<SourceFile>>,
    pub unverified: bool,
}
