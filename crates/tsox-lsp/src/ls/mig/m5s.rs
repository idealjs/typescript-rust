#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp_basic::{Location, Position, Range};
use crate::lsp::lsproto_util;
use crate::ls::find_all_references::{EntryKind, RefInfo, ReferenceEntry, SymbolAndEntries};
use crate::ls::language_service::LanguageService;
use crate::ls::lsutil_user_preferences::UserPreferences;
use crate::ls::types_highlight::{DocumentHighlight, DocumentHighlightKind, LocationLink, MultiDocumentHighlight};
use tsox_checker::checker::Checker;
use tsox_compile::compiler::Program;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;

#[derive(Debug, Clone, Default)]
pub struct DefinitionResponse {
    pub location: Option<Location>,
    pub locations: Option<Vec<Location>>,
    pub definition_links: Option<Vec<LocationLink>>,
}

#[derive(Debug, Clone, Default)]
pub struct MultiDocumentHighlightsOrNull {
    pub multi_document_highlights: Option<Vec<MultiDocumentHighlight>>,
}

pub static VIRTUAL_CODE_PRODUCED_BY_THE_CONTENT_MAPPER_0_HAS_PROBLEMS_WITH_NO_CORRESPONDING_LOCATION_IN_THIS_FILE: diagnostics::Message = diagnostics::Message {
    code: 100036,
    category: diagnostics::Category::Error,
    key: "Virtual_code_produced_by_the_content_mapper_0_has_problems_with_no_corresponding_location_in_this_file_100036",
    text: "Virtual code produced by the content mapper '{0}' has problems with no corresponding location in this file.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub fn aggregate_synthesized_diagnostics(
    file: &Arc<SourceFile>,
    diags: Vec<&ast::Diagnostic>,
) -> ast::Diagnostic {
    let mut aggregate = ast::Diagnostic::new(
        Some(file.clone()),
        TextRange::new(0, 0),
        VIRTUAL_CODE_PRODUCED_BY_THE_CONTENT_MAPPER_0_HAS_PROBLEMS_WITH_NO_CORRESPONDING_LOCATION_IN_THIS_FILE,
        vec![tsox_compile::mig::m3l_cm_2::content_mapper_source_file_info(&file.file_name)
            .map(|info| info.content_mapper)
            .unwrap_or_default()],
    );
    let owned: Vec<ast::Diagnostic> = diags.into_iter().cloned().collect();
    aggregate.related_information = owned.clone();
    aggregate.category = worst_category(&owned);
    aggregate
}

pub fn is_synthesized_content_mapped_diagnostic(diag: &ast::Diagnostic) -> bool {
    let Some(file) = diag.file.clone() else {
        return false;
    };
    match crate::mig::m5u_conv::source_file_span_map(&file) {
        None => false,
        Some(span_map) => {
            let (_, fidelity) = span_map.virtual_to_original_span(diag.loc);
            fidelity == crate::mig::m5u_conv::SPANMAP_FIDELITY_NONE
        }
    }
}

pub fn worst_category(diags: &[ast::Diagnostic]) -> diagnostics::Category {
    let mut worst = diags[0].category;
    for diag in diags {
        match diag.category {
            diagnostics::Category::Error => return diagnostics::Category::Error,
            diagnostics::Category::Warning => worst = diagnostics::Category::Warning,
            _ => {}
        }
    }
    worst
}

pub struct DisplayPartsWriter {
    builder: String,
    runs: Vec<VSClassifiedTextRun>,
    vs_capability: bool,
    last_written: String,
}

#[derive(Debug, Clone)]
pub struct VSClassifiedTextRun {
    pub classification_type_name: String,
    pub text: String,
}

impl DisplayPartsWriter {
    pub fn new(vs_capability: bool) -> Self {
        DisplayPartsWriter {
            builder: String::new(),
            runs: Vec::new(),
            vs_capability,
            last_written: String::new(),
        }
    }

    fn add_run(&mut self, classification: &str, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.vs_capability {
            self.runs.push(VSClassifiedTextRun {
                classification_type_name: classification.to_string(),
                text: text.to_string(),
            });
        }
        self.last_written = text.to_string();
        self.builder.push_str(text);
    }

    pub fn get_runs(&self) -> &[VSClassifiedTextRun] {
        &self.runs
    }

    pub fn decrease_indent(&self) {}

    pub fn get_column(&self) -> u32 {
        0
    }

    pub fn get_indent(&self) -> usize {
        0
    }

    pub fn get_line(&self) -> usize {
        0
    }

    pub fn get_text_pos(&self) -> usize {
        self.builder.len()
    }

    pub fn has_trailing_comment(&self) -> bool {
        false
    }

    pub fn increase_indent(&self) {}

    pub fn is_at_start_of_line(&self) -> bool {
        false
    }

    pub fn raw_write(&mut self, s: &str) {
        self.add_run("text", s);
    }

    pub fn to_string(&self) -> String {
        self.builder.clone()
    }
}

pub fn lsp_range_contains(outer: &Range, inner: &Range) -> bool {
    lsproto_util::compare_positions(&outer.start, &inner.start) != std::cmp::Ordering::Greater
        && lsproto_util::compare_positions(&inner.end, &outer.end) != std::cmp::Ordering::Greater
}

pub fn create_locations_from_links(links: &[LocationLink]) -> DefinitionResponse {
    let locations = links
        .iter()
        .map(|link| Location {
            uri: link.target_uri.clone(),
            range: link.target_selection_range.clone(),
        })
        .collect();
    DefinitionResponse {
        locations: Some(locations),
        ..Default::default()
    }
}

pub struct FileRange {
    pub file: Arc<SourceFile>,
    pub range: TextRange,
}

impl PartialEq for FileRange {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.file, &other.file)
            && self.range == other.range
    }
}

impl LanguageService {
    pub fn create_definition_locations(
        &self,
        origin_selection_range: Range,
        client_supports_link: bool,
        declarations: Vec<Arc<Node>>,
        reference: Option<&RefInfo>,
        feature: SpanFeature,
    ) -> DefinitionResponse {
        let mut locations: Vec<LocationLink> = Vec::new();
        let mut location_ranges: HashSet<FileRangeKey> = HashSet::new();

        if let Some(reference) = reference {
            let target_range = Range::default();
            locations.push(LocationLink {
                origin_selection_range: Some(origin_selection_range.clone()),
                target_uri: DocumentUri(crate::ls::lsconv_converters::file_name_to_document_uri(
                    &reference.file_name,
                )),
                target_range: target_range.clone(),
                target_selection_range: target_range,
            });
        }

        for decl in declarations {
            let Some(file) = crate::ls::mig::m5t_3::source_file_of_node(&self.get_program(), &decl)
            else {
                continue;
            };
            let name = ast::get_name_of_declaration(&decl).unwrap_or_else(|| decl.clone());
            let name_range = if name.kind == SyntaxKind::EmptyStatement {
                TextRange::new(name.pos(), name.pos())
            } else {
                crate::ls::mig::m5x_3::create_range_from_node(&name, &file)
            };
            let key = FileRangeKey {
                file: file.clone(),
                range: name_range,
            };
            if location_ranges.insert(key) {
                let context_node =
                    crate::ls::mig::m5s_3::get_context_node(&decl).unwrap_or_else(|| decl.clone());
                let mut context_range = crate::ls::mig::m5x_5::to_context_range(
                    Some(name_range),
                    &file,
                    Some(&context_node),
                )
                .unwrap_or(name_range);
                if !name_range.contained_by(&context_range) {                    context_range = TextRange::new(
                        name_range.pos().min(context_range.pos()),
                        name_range.end().max(context_range.end()),
                    );
                }
                let (target_selection_loc, selection_fidelity) = self
                    .source_file_range_to_lsp_location_for_feature(&file, name_range, feature);
                if !selection_fidelity.is_single_segment() {
                    continue;
                }
                let (mut target_loc, context_fidelity) =
                    self.source_file_range_to_lsp_location(&file, context_range);
                if context_fidelity.is_none()
                    || target_loc.uri != target_selection_loc.uri
                    || !lsp_range_contains(&target_loc.range, &target_selection_loc.range)
                {
                    target_loc = target_selection_loc.clone();
                }
                locations.push(LocationLink {
                    origin_selection_range: Some(origin_selection_range.clone()),
                    target_selection_range: target_selection_loc.range,
                    target_uri: target_loc.uri,
                    target_range: target_loc.range,
                });
            }
        }

        if client_supports_link {
            DefinitionResponse {
                definition_links: Some(locations),
                ..Default::default()
            }
        } else {
            create_locations_from_links(&locations)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanFeature {
    Definition,
    TypeDefinition,
    DocumentHighlights,
    References,
    Implementation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanFidelity {
    Exact,
    SingleSegment,
    None_,
}

impl SpanFidelity {
    pub fn is_none(self) -> bool {
        self == SpanFidelity::None_
    }

    pub fn is_single_segment(self) -> bool {
        self == SpanFidelity::Exact || self == SpanFidelity::SingleSegment
    }
}

#[derive(Clone)]
pub struct FileRangeKey {
    file: Arc<SourceFile>,
    range: TextRange,
}

impl PartialEq for FileRangeKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.file, &other.file) && self.range == other.range
    }
}
impl Eq for FileRangeKey {}
impl std::hash::Hash for FileRangeKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.file) as usize).hash(state);
        self.range.hash(state);
    }
}

impl LanguageService {
    pub fn provide_definition_at_position(
        &self,
        program: &Arc<Program>,
        file: &Arc<SourceFile>,
        text_pos: u32,
        client_supports_link: bool,
    ) -> DefinitionResponse {
        let pos = text_pos as usize;
        let Some(node) = astnav::get_touching_property_name(&file.node, pos) else {
            return DefinitionResponse::default();
        };
        let reference = crate::ls::mig::m5w_6::get_reference_at_position(file, pos, program)
            .and_then(|r| {
                r.file.map(|f| RefInfo {
                    file: Some(Arc::clone(&f)),
                    file_name: f.file_name.clone(),
                })
            });

        if node.kind == SyntaxKind::SourceFile {
            return DefinitionResponse::default();
        }

        let (origin_selection_range, _) = self.create_lsp_range_from_node(&node, file);
        if let Some(ref_info) = &reference {
            if ref_info.file.is_some() {
                return self.create_definition_locations(
                    origin_selection_range,
                    client_supports_link,
                    Vec::new(),
                    Some(ref_info),
                    SpanFeature::Definition,
                );
            }
        }

        let mut c = program.get_type_checker_for_file(file);

        if node.kind == SyntaxKind::OverrideKeyword {
            if let Some(sym) = get_symbol_for_overridden_member(&mut c, &node) {
                return self.create_definition_locations(
                    origin_selection_range,
                    client_supports_link,
                    sym.declarations.clone(),
                    None,
                    SpanFeature::Definition,
                );
            }
        }

        if ast::mig::m3g::is_jump_statement_target(&node) {
            if let Some(parent) = node.parent() {
                if let Some(label) =
                    crate::ls::mig::m5x_4::get_target_label(&parent, node.text())
                {
                    return self.create_definition_locations(
                        origin_selection_range,
                        client_supports_link,
                        vec![label],
                        None,
                        SpanFeature::Definition,
                    );
                }
            }
        }

        let parent_is_default_clause = node
            .parent()
            .as_deref()
            .map(|p| p.kind == SyntaxKind::DefaultClause)
            .unwrap_or(false);
        if node.kind == SyntaxKind::CaseKeyword || parent_is_default_clause {
            if let Some(parent) = node.parent() {
                if let Some(stmt) = ast::find_ancestor(&parent, ast::is_switch_statement) {
                    let Some(file) = crate::ls::mig::m5t_3::source_file_of_node(program, &stmt)
                    else {
                        return DefinitionResponse::default();
                    };
                    return self.create_location_from_file_and_range(
                        &file,
                        tsox_frontend::scanner::mig::m4d_2::get_range_of_token_at_position(
                            &file,
                            stmt.pos(),
                        ),
                        SpanFeature::Definition,
                    );
                }
            }
        }

        if node.kind == SyntaxKind::ReturnKeyword
            || node.kind == SyntaxKind::YieldKeyword
            || node.kind == SyntaxKind::AwaitKeyword
        {
            if let Some(fn_node) = ast::find_ancestor(&node, ast::is_function_like_declaration) {
                return self.create_definition_locations(
                    origin_selection_range,
                    client_supports_link,
                    vec![fn_node],
                    None,
                    SpanFeature::Definition,
                );
            }
        }

        let mut declarations = get_declarations_from_location(&mut c, &node);
        let called_declaration = try_get_signature_declaration(&mut c, &node);
        if let Some(called) = called_declaration {
            let jsx_block = node
                .parent()
                .as_deref()
                .map(ast::mig::m3g::is_jsx_opening_like_element)
                .unwrap_or(false)
                && is_jsx_constructor_like(&called);
            if !jsx_block {
                let keyword_name_node = get_declaration_name_for_keyword(&node);
                let symbol = c.get_symbol_at_location(&keyword_name_node);
                let matches = symbol.as_ref().is_some_and(|symbol| {
                    c.get_root_symbols(symbol).iter().any(|root| {
                        symbol_matches_signature(root, &called)
                    })
                });
                if matches {
                    if !ast::is_constructor_declaration(&called) {
                        declarations = Vec::new();
                    } else {
                        declarations = declarations
                            .into_iter()
                            .filter(|n| {
                                !Arc::ptr_eq(n, &called)
                                    && (ast::is_class_declaration(n) || ast::is_class_expression(n))
                            })
                            .collect();
                    }
                } else {
                    declarations = declarations
                        .into_iter()
                        .filter(|n| !Arc::ptr_eq(n, &called))
                        .collect();
                }
                declarations.push(called);
            }
        }
        self.create_definition_locations(
            origin_selection_range,
            client_supports_link,
            declarations,
            reference.as_ref(),
            SpanFeature::Definition,
        )
    }

    pub fn provide_type_definition_at_position(
        &self,
        program: &Arc<Program>,
        file: &Arc<SourceFile>,
        text_pos: u32,
        client_supports_link: bool,
    ) -> DefinitionResponse {
        let pos = text_pos as usize;
        let Some(mut node) = astnav::get_touching_property_name(&file.node, pos) else {
            return DefinitionResponse::default();
        };
        if node.kind == SyntaxKind::SourceFile {
            return DefinitionResponse::default();
        }
        let (origin_selection_range, _) = self.create_lsp_range_from_node(&node, file);

        let mut c = program.get_type_checker_for_file(file);

        node = get_declaration_name_for_keyword(&node);

        if let Some(symbol) = c.get_symbol_at_location(&node) {
            let symbol_type = get_type_of_symbol_at_location(&mut c, &symbol, &node);
            let mut declarations = get_declarations_from_type(&symbol_type);
            if let Some(type_argument) = c.get_first_type_argument_from_known_type(&symbol_type) {
                let mut combined = get_declarations_from_type(&type_argument);
                combined.extend(declarations);
                declarations = combined;
            }
            if !declarations.is_empty() {
                return self.create_definition_locations(
                    origin_selection_range,
                    client_supports_link,
                    declarations,
                    None,
                    SpanFeature::TypeDefinition,
                );
            }
            let flags = symbol.flags;
            if !flags.contains(ast::SymbolFlags::VALUE) && flags.contains(ast::SymbolFlags::TYPE) {
                return self.create_definition_locations(
                    origin_selection_range,
                    client_supports_link,
                    symbol.declarations.clone(),
                    None,
                    SpanFeature::TypeDefinition,
                );
            }
        }

        DefinitionResponse::default()
    }

    pub fn create_location_from_file_and_range(
        &self,
        file: &Arc<SourceFile>,
        text_range: TextRange,
        feature: SpanFeature,
    ) -> DefinitionResponse {
        let (mut mapped_location, fidelity) =
            self.source_file_range_to_lsp_location_for_feature(file, text_range, feature);
        if fidelity.is_none() {
            mapped_location.range = Range::default();
        }
        DefinitionResponse {
            location: Some(mapped_location),
            ..Default::default()
        }
    }
}

fn declaration_name_of_node(node: &Node) -> Option<Arc<Node>> {
    let name = match &node.data {
        ast::NodeData::VariableDeclaration(d) => Some(&d.name),
        ast::NodeData::ParameterDeclaration(d) => Some(&d.name),
        ast::NodeData::BindingElement(d) => d.name.as_ref(),
        ast::NodeData::FunctionDeclaration(d) => d.name.as_ref(),
        ast::NodeData::ClassDeclaration(d) => d.name.as_ref(),
        ast::NodeData::ClassExpression(d) => d.name.as_ref(),
        ast::NodeData::InterfaceDeclaration(d) => Some(&d.name),
        ast::NodeData::TypeAliasDeclaration(d) => Some(&d.name),
        ast::NodeData::EnumMember(d) => Some(&d.name),
        ast::NodeData::EnumDeclaration(d) => Some(&d.name),
        ast::NodeData::ExportSpecifier(d) => Some(&d.name),
        ast::NodeData::GetAccessorDeclaration(d) => Some(&d.name),
        ast::NodeData::SetAccessorDeclaration(d) => Some(&d.name),
        ast::NodeData::MethodSignatureDeclaration(d) => Some(&d.name),
        ast::NodeData::MethodDeclaration(d) => Some(&d.name),
        ast::NodeData::PropertySignatureDeclaration(d) => Some(&d.name),
        ast::NodeData::PropertyDeclaration(d) => Some(&d.name),
        ast::NodeData::FunctionExpression(d) => d.name.as_ref(),
        ast::NodeData::PropertyAssignment(d) => Some(&d.name),
        ast::NodeData::ShorthandPropertyAssignment(d) => Some(&d.name),
        ast::NodeData::NamedTupleMember(d) => Some(&d.name),
        ast::NodeData::ModuleDeclaration(d) => Some(&d.name),
        ast::NodeData::ImportEqualsDeclaration(d) => Some(&d.name),
        ast::NodeData::ImportSpecifier(d) => Some(&d.name),
        ast::NodeData::TypeParameterDeclaration(d) => Some(&d.name),
        _ => None,
    }?;
    Some(Arc::clone(name))
}

pub fn get_declaration_name_for_keyword(node: &Arc<Node>) -> Arc<Node> {
    let kind = node.kind;
    if tsox_frontend::ast::node_data_generated::is_keyword_kind(kind) {
        if let Some(parent) = node.parent() {
            if ast::is_variable_declaration_list(&parent) {
                if let ast::NodeData::VariableDeclarationList(data) = &parent.data {
                    if let Some(decl) = data.declarations.nodes.first() {
                        if let Some(name) = decl.name() {
                            return Arc::clone(name);
                        }
                    }
                }
            } else if let Some(name) = declaration_name_of_node(&parent) {
                if node.pos() < name.pos() {
                    return name;
                }
            }
        }
    }
    node.clone()
}

pub fn get_declarations_from_object_literal_element(
    c: &mut Checker,
    node: &Arc<Node>,
) -> Vec<Arc<Node>> {
    let element = match crate::ls::mig::m5x_4::get_containing_object_literal_element_worker(node)
    {
        Some(element) => element,
        None => return Vec::new(),
    };

    let element_parent = element.parent();
    let contextual_type = match element_parent.as_ref() {
        Some(element_parent) => {
            c.get_contextual_type(
                element_parent,
                tsox_checker::checker::ContextFlags::None,
            )
        }
        None => None,
    };
    let Some(contextual_type) = contextual_type else {
        return Vec::new();
    };

    let mut properties =
        c.get_property_symbols_from_contextual_type(&element, &contextual_type, false);
    if properties.iter().any(|p| {
        p.value_declaration.as_ref().is_some_and(|vd| {
            vd.parent()
                .as_deref()
                .map(ast::is_object_literal_expression)
                .unwrap_or(false)
                && ast::is_object_literal_element(vd)
                && vd.name().is_some_and(|n| Arc::ptr_eq(n, node))
        })
    }) {
        if let Some(element_parent) = element.parent() {
            if let Some(without_node_inferences_type) = c.get_contextual_type(
                &element_parent,
                tsox_checker::checker::ContextFlags::IgnoreNodeInferences,
            ) {
                let without_node_inferences_properties = c.get_property_symbols_from_contextual_type(
                    &element,
                    &without_node_inferences_type,
                    false,
                );
                if !without_node_inferences_properties.is_empty() {
                    properties = without_node_inferences_properties;
                }
            }
        }
    }

    let mut result = Vec::new();
    for prop in &properties {
        result.extend(prop.declarations.iter().cloned());
    }
    result
}

pub fn get_ancestor_call_like_expression(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut target = node.clone();
    loop {
        if !ast::mig::m3g_2::is_right_side_of_property_access(&target) {
            break;
        }
        target = target.parent()?;
    }
    let call_like = target.parent()?;
    if ast::mig::m3f_4::is_call_like_expression(&call_like)
        && Arc::ptr_eq(&ast::mig::x4ast::get_invoked_expression(&call_like), &target)
    {
        Some(call_like)
    } else {
        None
    }
}

pub fn try_get_signature_declaration(type_checker: &mut Checker, node: &Arc<Node>) -> Option<Arc<Node>> {
    let call_like = get_ancestor_call_like_expression(node);
    let signature = call_like.and_then(|cl| type_checker.get_resolved_signature(&cl));
    if let Some(declaration) = signature.and_then(|sig| sig.declaration.clone()) {
        if ast::is_function_like(&declaration) && !ast::is_function_type_node(&declaration) {
            return Some(declaration);
        }
    }
    None
}

pub fn is_jsx_constructor_like(node: &Arc<Node>) -> bool {
    ast::is_constructor_declaration(node)
        || ast::is_constructor_type_node(node)
        || ast::is_call_signature_declaration(node)
        || ast::is_construct_signature_declaration(node)
}

pub fn symbol_matches_signature(symbol: &Arc<Symbol>, called_declaration: &Arc<Node>) -> bool {
    let called_symbol =
        tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(called_declaration);
    if called_symbol
        .as_ref()
        .is_some_and(|cs| Arc::ptr_eq(symbol, cs))
        || called_symbol
            .as_ref()
            .and_then(|cs| cs.parent())
            .is_some_and(|p| Arc::ptr_eq(symbol, &p))
    {
        return true;
    }
    if let Some(parent) = called_declaration.parent() {
        ast::is_assignment_expression(&parent, false)
            || (!ast::mig::m3f_4::is_call_like_expression(&parent)
                && ast::can_have_symbol(&parent)
                && tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(&parent)
                    .is_some_and(|ps| Arc::ptr_eq(symbol, &ps)))
    } else {
        false
    }
}

pub fn get_symbol_for_overridden_member(
    type_checker: &mut Checker,
    node: &Arc<Node>,
) -> Option<Arc<Symbol>> {
    let class_element = ast::find_ancestor(node, ast::is_class_element)?;
    class_element.name()?;
    let base_declaration = ast::find_ancestor(&class_element, ast::is_class_like)?;
    let base_type_node = ast::get_class_extends_heritage_element(&base_declaration)?;
    let expression = base_type_node
        .expression()
        .map(|e| ast::skip_parentheses(e))?;
    let base = if ast::is_class_expression(&expression) {
        tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(&expression)
    } else {
        type_checker.get_symbol_at_location(&expression)
    }?;
    let name = class_element
        .name()
        .map(|n| ast::mig::w5::get_text_of_property_name(n))?;
    if ast::has_static_modifier(&class_element) {
        let base_type = type_checker.get_type_of_symbol(&base);
        type_checker.get_property_of_type(&base_type, &name)
    } else {
        let base_type = type_checker.get_declared_type_of_symbol(&base);
        type_checker.get_property_of_type(&base_type, &name)
    }
}

pub fn get_type_of_symbol_at_location(
    c: &mut Checker,
    symbol: &Arc<Symbol>,
    node: &Arc<Node>,
) -> Arc<tsox_checker::checker::types::Type> {
    let t = c.get_type_of_symbol_at_location(symbol, node);
    let t_symbol_same = t.symbol().is_some_and(|ts| Arc::ptr_eq(ts, symbol));
    let initializer_matches = t
        .symbol()
        .zip(symbol.value_declaration.clone())
        .is_some_and(|(ts, vd)| {
            ast::is_variable_declaration(&vd)
                && vd
                    .initializer()
                    .zip(ts.value_declaration.clone())
                    .is_some_and(|(vi, tvi)| Arc::ptr_eq(&vi, &tvi))
        });
    if t_symbol_same || initializer_matches {
        let sigs = c.get_call_signatures(&t);
        if sigs.len() == 1 {
            if let Some(rt) = c.get_return_type_of_signature(&sigs[0]) {
                return rt;
            }
        }
    }
    t
}

pub fn get_declarations_from_type(
    t: &Arc<tsox_checker::checker::types::Type>,
) -> Vec<Arc<Node>> {
    let mut result: Vec<Arc<Node>> = Vec::new();
    for ty in t.distributed() {
        if let Some(symbol) = ty.symbol() {
            for decl in symbol.declarations.iter() {
                if !result.iter().any(|d| Arc::ptr_eq(d, decl)) {
                    result.push(Arc::clone(decl));
                }
            }
        }
    }
    result
}

fn is_property_name(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::ComputedPropertyName
    )
}

pub fn get_declarations_from_location(c: &mut Checker, node: &Arc<Node>) -> Vec<Arc<Node>> {
    if let Some(parent) = node.parent() {
        if ast::is_identifier(node) && ast::is_shorthand_property_assignment(&parent) {
            let shorthand_symbol = c.get_resolved_symbol(node);
            let mut declarations: Vec<Arc<Node>> = shorthand_symbol
                .map(|s| s.declarations.clone())
                .unwrap_or_default();
            let contextual_declarations = get_declarations_from_object_literal_element(c, node);
            declarations.extend(contextual_declarations);
            return declarations;
        }
    }

    let parent = node.parent();
    if let Some(parent) = &parent {
        if is_property_name(node)
            && ast::is_binding_element(parent)
            && parent
                .parent()
                .as_deref()
                .map(ast::is_object_binding_pattern)
                .unwrap_or(false)
        {
            let grandparent = parent.parent().unwrap();
            if let ast::NodeData::BindingElement(binding_el) = &parent.data {
                if binding_el.dot_dot_dot_token.is_none() {
                    let matches_name = binding_el
                        .property_name
                        .as_ref()
                        .map(|pn| Arc::ptr_eq(pn, node))
                        .unwrap_or(false)
                        || parent
                            .name()
                            .is_some_and(|n| Arc::ptr_eq(n, node));
                    if matches_name {
                        if let Some(name) = ast::mig::m3h::try_get_text_of_property_name(node) {
                            let t = c.get_type_at_location(&grandparent);
                            let types: Vec<Arc<tsox_checker::checker::types::Type>> =
                                if t.is_union() {
                                    t.types().map(|ts| ts.to_vec()).unwrap_or_default()
                                } else {
                                    vec![Arc::clone(&t)]
                                };
                            let mut result = Vec::new();
                            for union_type in &types {
                                if let Some(prop) = c.get_property_of_type(union_type, &name) {
                                    result.extend(prop.declarations.iter().cloned());
                                }
                            }
                            return result;
                        }
                    }
                }
            }
        }
    }

    let node = get_declaration_name_for_keyword(node);
    if let Some(mut symbol) = c.get_symbol_at_location(&node) {
        let flags = symbol.flags;
        if flags.contains(ast::SymbolFlags::Class)
            && !flags.intersects(ast::SymbolFlags::Function | ast::SymbolFlags::VARIABLE)
            && node.kind == SyntaxKind::ConstructorKeyword
        {
            if let Some(constructor) =
                symbol.members.get(ast::INTERNAL_SYMBOL_NAME_CONSTRUCTOR)
            {
                symbol = constructor.clone();
            }
        }
        if flags.contains(ast::SymbolFlags::Alias) {
            if let Some(resolved) = c.follow_alias_resolving(&symbol) {
                symbol = resolved;
            }
        }
        let object_literal_element_declarations =
            get_declarations_from_object_literal_element(c, &node);
        if !object_literal_element_declarations.is_empty() {
            return object_literal_element_declarations;
        }
        let decls = symbol.declarations.clone();
        if !decls.is_empty() {
            return decls;
        }
    }
    let index_infos = c.get_index_signatures_at_location(&node);
    if !index_infos.is_empty() {
        return index_infos;
    }
    Vec::new()
}
